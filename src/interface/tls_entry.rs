use std::fs;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use log::{error, info, warn};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio_rustls::TlsAcceptor;

fn load_certs(path: &str) -> io::Result<Vec<CertificateDer<'static>>> {
    let pem = fs::read(path)?;
    let mut reader = io::BufReader::new(io::Cursor::new(pem));
    rustls_pemfile::certs(&mut reader).collect()
}

fn load_private_key(path: &str) -> io::Result<PrivateKeyDer<'static>> {
    let pem = fs::read(path)?;
    let mut reader = io::BufReader::new(io::Cursor::new(pem));
    let key = rustls_pemfile::private_key(&mut reader)?;
    key.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no private key found"))
}

fn port_from_env(vars: &[&str], default_port: u16) -> u16 {
    for var in vars {
        if let Ok(v) = std::env::var(var) {
            if let Ok(port) = v.parse::<u16>() {
                return port;
            }
        }
    }
    default_port
}

fn local_backend_from_listen_addr(var: &str, default_port: u16) -> String {
    let port = std::env::var(var)
        .ok()
        .and_then(|v| v.parse::<SocketAddr>().ok())
        .map(|a| a.port())
        .unwrap_or(default_port);
    format!("127.0.0.1:{}", port)
}

fn u64_from_env(var: &str, default_value: u64) -> u64 {
    std::env::var(var)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default_value)
}

static CONN_SEQ: AtomicU64 = AtomicU64::new(1);

pub async fn serve_tls_entry(addr: std::net::SocketAddr) -> io::Result<()> {
    let cert_dir = std::env::var("TLS_CERT_DIR").unwrap_or_default();
    if cert_dir.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "TLS_CERT_DIR is required for TLS entry",
        ));
    }

    let max_conns = u64_from_env("TLS_MAX_CONNS", 1024) as usize;
    let handshake_timeout_ms = u64_from_env("TLS_HANDSHAKE_TIMEOUT_MS", 8_000);
    let backend_connect_timeout_ms = u64_from_env("TLS_BACKEND_CONNECT_TIMEOUT_MS", 3_000);
    let session_timeout_secs = u64_from_env("TLS_SESSION_TIMEOUT_SECS", 0);
    let semaphore = Arc::new(Semaphore::new(max_conns));

    let grpc_backend = std::env::var("TLS_GRPC_BACKEND_ADDR")
        .unwrap_or_else(|_| local_backend_from_listen_addr("GRPC_LISTEN_ADDR", 50051));

    let admin_port = port_from_env(&["ADMIN_PORT"], 667);
    let admin_backend = std::env::var("TLS_ADMIN_BACKEND_ADDR")
        .unwrap_or_else(|_| format!("127.0.0.1:{}", admin_port));

    let api_port = port_from_env(&["CLIENT_API_PORT", "PORT"], 8080);
    let api_backend = std::env::var("TLS_API_BACKEND_ADDR")
        .unwrap_or_else(|_| format!("127.0.0.1:{}", api_port));

    let cert_path = format!("{}/fullchain.pem", cert_dir.trim_end_matches(['/', '\\']));
    let key_path = format!("{}/privkey.pem", cert_dir.trim_end_matches(['/', '\\']));

    let certs = load_certs(&cert_path)?;
    let key = load_private_key(&key_path)?;

    let mut tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    let acceptor = TlsAcceptor::from(Arc::new(tls_config));
    let listener = TcpListener::bind(addr).await?;

    info!("[tls-entry] listening on {}", addr);

    loop {
        let (tcp, peer) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let grpc_backend = grpc_backend.clone();
        let admin_backend = admin_backend.clone();
        let api_backend = api_backend.clone();
        let semaphore = semaphore.clone();

        let permit = match semaphore.try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                warn!("[tls-entry] reject peer={} reason=too_many_connections", peer);
                drop(tcp);
                continue;
            }
        };

        tokio::spawn(async move {
            let _permit = permit;
            let conn_id = CONN_SEQ.fetch_add(1, Ordering::Relaxed);
            let t0 = Instant::now();

            let handshake_res = tokio::time::timeout(
                tokio::time::Duration::from_millis(handshake_timeout_ms),
                acceptor.accept(tcp),
            )
            .await;

            match handshake_res {
                Ok(Ok(mut tls_stream)) => {
                    let sni = tls_stream
                        .get_ref()
                        .1
                        .server_name()
                        .unwrap_or("")
                        .to_string();

                    let alpn = tls_stream
                        .get_ref()
                        .1
                        .alpn_protocol()
                        .map(|p| String::from_utf8_lossy(p).to_string())
                        .unwrap_or_default();

                    let backend = if sni.starts_with("grpc.") {
                        grpc_backend
                    } else if sni.starts_with("admin.") {
                        admin_backend
                    } else if sni.starts_with("api.") {
                        api_backend
                    } else {
                        warn!("[tls-entry] unknown sni peer={} sni={} (fallback admin)", peer, sni);
                        admin_backend
                    };

                    info!(
                        "[tls-entry] accepted conn_id={} peer={} sni={} alpn={} backend={} elapsed_ms={} ",
                        conn_id,
                        peer,
                        sni,
                        alpn,
                        backend,
                        t0.elapsed().as_millis()
                    );

                    let connect_res = tokio::time::timeout(
                        tokio::time::Duration::from_millis(backend_connect_timeout_ms),
                        TcpStream::connect(&backend),
                    )
                    .await;

                    match connect_res {
                        Ok(Ok(mut upstream)) => {
                            let io_fut = tokio::io::copy_bidirectional(&mut tls_stream, &mut upstream);

                            let io_res = if session_timeout_secs > 0 {
                                tokio::time::timeout(
                                    tokio::time::Duration::from_secs(session_timeout_secs),
                                    io_fut,
                                )
                                .await
                                .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "session timeout"))
                                .and_then(|r| r)
                            } else {
                                io_fut.await
                            };

                            match io_res {
                                Ok((c2s, s2c)) => {
                                    info!(
                                        "[tls-entry] done conn_id={} peer={} sni={} backend={} c2s_bytes={} s2c_bytes={} elapsed_ms={}",
                                        conn_id,
                                        peer,
                                        sni,
                                        backend,
                                        c2s,
                                        s2c,
                                        t0.elapsed().as_millis()
                                    );
                                }
                                Err(e) => {
                                    error!(
                                        "[tls-entry] proxy error conn_id={} peer={} sni={} backend={} err={} elapsed_ms={}",
                                        conn_id,
                                        peer,
                                        sni,
                                        backend,
                                        e,
                                        t0.elapsed().as_millis()
                                    );
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            error!(
                                "[tls-entry] connect backend error conn_id={} peer={} sni={} backend={} err={} elapsed_ms={}",
                                conn_id,
                                peer,
                                sni,
                                backend,
                                e,
                                t0.elapsed().as_millis()
                            );
                        }
                        Err(_) => {
                            error!(
                                "[tls-entry] connect backend timeout conn_id={} peer={} sni={} backend={} elapsed_ms={}",
                                conn_id,
                                peer,
                                sni,
                                backend,
                                t0.elapsed().as_millis()
                            );
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!(
                        "[tls-entry] tls handshake failed conn_id={} peer={} err={} elapsed_ms={}",
                        conn_id,
                        peer,
                        e,
                        t0.elapsed().as_millis()
                    );
                }
                Err(_) => {
                    error!(
                        "[tls-entry] tls handshake timeout conn_id={} peer={} elapsed_ms={}",
                        conn_id,
                        peer,
                        t0.elapsed().as_millis()
                    );
                }
            }
        });
    }
}
