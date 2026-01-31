use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use log::{error, info, warn};
use moka::future::Cache;
use once_cell::sync::Lazy;
use sha2::Sha256;
use tonic::metadata::MetadataMap;
use tonic::{Request, Response, Status};
use tokio::sync::{mpsc, watch, Mutex, RwLock};
use tokio::time;
use tokio_stream::wrappers::ReceiverStream;

use crate::infrastructure::admin_config::AdminConfigStore;
use crate::infrastructure::persistence::admin_node_config;
use sea_orm::EntityTrait;

pub mod nodepanel {
    tonic::include_proto!("nodepanel");
}

pub mod nodecontrol {
    tonic::include_proto!("nodecontrol");
}

pub struct NodePanelService {
    admin_config: AdminConfigStore,
}

pub struct NodeControlService {
    admin_config: AdminConfigStore,
}

static NONCE_CACHE: Lazy<Cache<String, ()>> = Lazy::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(600))
        .max_capacity(200_000)
        .build()
});

struct NodeSession {
    tx: mpsc::Sender<Result<nodecontrol::NodeStreamResponse, Status>>,
    pull_ack: Mutex<HashMap<String, watch::Sender<u64>>>,
}

impl NodeSession {
    fn new(tx: mpsc::Sender<Result<nodecontrol::NodeStreamResponse, Status>>) -> Self {
        Self {
            tx,
            pull_ack: Mutex::new(HashMap::new()),
        }
    }

    async fn notify_pull_ack(&self, kind: &str) {
        let mut map = self.pull_ack.lock().await;
        let sender = map.entry(kind.to_string()).or_insert_with(|| {
            let (tx, _rx) = watch::channel(0u64);
            tx
        });
        let next = *sender.borrow() + 1;
        let _ = sender.send(next);
    }
}

static NODE_SESSIONS: Lazy<RwLock<HashMap<u64, Arc<NodeSession>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn is_node_connected(node_id: u64) -> bool {
    NODE_SESSIONS.read().await.contains_key(&node_id)
}

pub async fn pull_ack_counter(node_id: u64, kind: &str) -> Result<u64, String> {
    let session = NODE_SESSIONS.read().await.get(&node_id).cloned();
    let Some(session) = session else {
        return Err("grpc node not connected".to_string());
    };

    let sender = {
        let mut map = session.pull_ack.lock().await;
        map.entry(kind.to_string())
            .or_insert_with(|| {
                let (tx, _rx) = watch::channel(0u64);
                tx
            })
            .clone()
    };

    Ok(*sender.borrow())
}

pub async fn wait_for_pull_ack_at_least(
    node_id: u64,
    kind: &str,
    min: u64,
    timeout_secs: u64,
) -> Result<(), String> {
    let session = NODE_SESSIONS.read().await.get(&node_id).cloned();
    let Some(session) = session else {
        return Err("grpc node not connected".to_string());
    };

    let sender = {
        let mut map = session.pull_ack.lock().await;
        map.entry(kind.to_string())
            .or_insert_with(|| {
                let (tx, _rx) = watch::channel(0u64);
                tx
            })
            .clone()
    };

    let mut rx = sender.subscribe();
    if *rx.borrow() >= min {
        return Ok(());
    }

    let wait_fut = async move {
        loop {
            rx.changed().await.map_err(|_| "grpc disconnected".to_string())?;
            if *rx.borrow() >= min {
                return Ok(());
            }
        }
    };

    if timeout_secs == 0 {
        return wait_fut.await;
    }

    tokio::time::timeout(Duration::from_secs(timeout_secs), wait_fut)
        .await
        .map_err(|_| "timeout".to_string())?
}

pub async fn send_update_event(node_id: u64, kind: &str) -> Result<(), String> {
    let session = NODE_SESSIONS.read().await.get(&node_id).cloned();
    let Some(session) = session else {
        return Err("grpc node not connected".to_string());
    };

    let msg = nodecontrol::NodeStreamResponse {
        msg: Some(nodecontrol::node_stream_response::Msg::UpdateEvent(
            nodecontrol::UpdateEvent {
                kind: kind.to_string(),
                version: 0,
                ts: chrono::Utc::now().timestamp(),
            },
        )),
    };

    session
        .tx
        .send(Ok(msg))
        .await
        .map_err(|_| "grpc send failed".to_string())
}

pub async fn wait_for_pull_ack(node_id: u64, kind: &str, timeout_secs: u64) -> Result<(), String> {
    let cur = pull_ack_counter(node_id, kind).await?;
    wait_for_pull_ack_at_least(node_id, kind, cur + 1, timeout_secs).await
}

impl NodePanelService {
    pub fn new(admin_config: AdminConfigStore) -> Self {
        Self { admin_config }
    }

    async fn authenticate<T>(&self, request: &Request<T>) -> Result<u64, Status> {
        let meta = request.metadata();

        let node_id: u64 = meta
            .get("x-node-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| Status::unauthenticated("missing x-node-id"))?;

        let token = meta.get("x-node-token").and_then(|v| v.to_str().ok());
        let ts: u64 = meta
            .get("x-ts")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| Status::unauthenticated("missing x-ts"))?;
        let nonce = meta
            .get("x-nonce")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("missing x-nonce"))?;
        let sign = meta
            .get("x-sign")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("missing x-sign"))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Status::internal("time error"))?
            .as_secs();
        let skew = now.abs_diff(ts);
        if skew > 300 {
            return Err(Status::unauthenticated("timestamp skew too large"));
        }

        let nonce_key = format!("{}:{}", node_id, nonce);
        if NONCE_CACHE.get(&nonce_key).await.is_some() {
            return Err(Status::unauthenticated("replayed nonce"));
        }
        NONCE_CACHE.insert(nonce_key, ()).await;

        let node = admin_node_config::Entity::find_by_id(node_id)
            .one(self.admin_config.db())
            .await
            .map_err(|e| Status::internal(format!("db error: {}", e)))?
            .ok_or_else(|| Status::unauthenticated("node not found"))?;

        if let Some(expected_token) = node.node_token.as_deref().filter(|s| !s.is_empty()) {
            if token.unwrap_or("") != expected_token {
                return Err(Status::unauthenticated("invalid token"));
            }
        } else {
            warn!("[grpc] node_token missing for node_id={}, allowing request", node_id);
        }

        let Some(shared_secret) = node
            .node_shared_secret
            .as_deref()
            .filter(|s| !s.is_empty())
        else {
            warn!(
                "[grpc] node_shared_secret missing for node_id={}, allowing request",
                node_id
            );
            return Ok(node_id);
        };

        let payload = format!("{}:{}:{}", node_id, ts, nonce);
        let mut mac = Hmac::<Sha256>::new_from_slice(shared_secret.as_bytes())
            .map_err(|_| Status::internal("invalid shared_secret"))?;
        mac.update(payload.as_bytes());
        let expected = hex::encode(mac.finalize().into_bytes());

        if !expected.eq_ignore_ascii_case(sign) {
            return Err(Status::unauthenticated("invalid signature"));
        }

        Ok(node_id)
    }
}

#[tonic::async_trait]
impl nodecontrol::node_control_service_server::NodeControlService for NodeControlService {
    type NodeStreamStream = ReceiverStream<Result<nodecontrol::NodeStreamResponse, Status>>;

    async fn node_stream(
        &self,
        request: Request<tonic::Streaming<nodecontrol::NodeStreamRequest>>,
    ) -> Result<Response<Self::NodeStreamStream>, Status> {
        let node_id = self
            .authenticate_meta(request.metadata().clone())
            .await?;
        info!("[grpc] node_stream connected node_id={}", node_id);

        let mut inbound = request.into_inner();

        let (tx, rx) = mpsc::channel::<Result<nodecontrol::NodeStreamResponse, Status>>(64);
        let session = Arc::new(NodeSession::new(tx.clone()));
        {
            let mut sessions = NODE_SESSIONS.write().await;
            sessions.insert(node_id, session.clone());
        }
        let mut tx_heartbeat = tx.clone();

        tokio::spawn(async move {
            let mut ticker = time::interval(Duration::from_secs(30));
            loop {
                ticker.tick().await;
                let msg = nodecontrol::NodeStreamResponse {
                    msg: Some(nodecontrol::node_stream_response::Msg::Heartbeat(
                        nodecontrol::Heartbeat { ts: chrono::Utc::now().timestamp() },
                    )),
                };

                if tx_heartbeat.send(Ok(msg)).await.is_err() {
                    break;
                }
            }
        });

        let session_for_inbound = session.clone();
        tokio::spawn(async move {
            loop {
                match inbound.message().await {
                    Ok(Some(frame)) => match frame.msg {
                        Some(nodecontrol::node_stream_request::Msg::Heartbeat(hb)) => {
                            info!("[grpc] node_stream heartbeat node_id={} ts={}", node_id, hb.ts);
                        }
                        Some(nodecontrol::node_stream_request::Msg::PullAck(ack)) => {
                            info!(
                                "[grpc] node_stream pull_ack node_id={} kind={} version={} ts={}",
                                node_id, ack.kind, ack.version, ack.ts
                            );
                            session_for_inbound.notify_pull_ack(&ack.kind).await;
                        }
                        Some(nodecontrol::node_stream_request::Msg::Report(r)) => {
                            info!(
                                "[grpc] node_stream report node_id={} kind={} bytes={} ts={}",
                                node_id,
                                r.kind,
                                r.payload.len(),
                                r.ts
                            );
                        }
                        Some(nodecontrol::node_stream_request::Msg::CommandResponse(rsp)) => {
                            info!(
                                "[grpc] node_stream command_response node_id={} request_id={} ok={} bytes={}",
                                node_id,
                                rsp.request_id,
                                rsp.ok,
                                rsp.payload.len()
                            );
                        }
                        None => {
                            warn!("[grpc] node_stream empty frame node_id={}", node_id);
                        }
                    },
                    Ok(None) => {
                        info!("[grpc] node_stream closed node_id={}", node_id);
                        break;
                    }
                    Err(e) => {
                        warn!("[grpc] node_stream recv error node_id={} err={}", node_id, e);
                        break;
                    }
                }
            }

            {
                let mut sessions = NODE_SESSIONS.write().await;
                if let Some(cur) = sessions.get(&node_id) {
                    if Arc::ptr_eq(cur, &session_for_inbound) {
                        sessions.remove(&node_id);
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn get_users(
        &self,
        request: Request<nodecontrol::GetUsersRequest>,
    ) -> Result<Response<nodecontrol::GetUsersReply>, Status> {
        let _node_id = self
            .authenticate_meta(request.metadata().clone())
            .await?;
        Ok(Response::new(nodecontrol::GetUsersReply {
            payload: Vec::new(),
            version: 0,
        }))
    }

    async fn get_config(
        &self,
        request: Request<nodecontrol::GetConfigRequest>,
    ) -> Result<Response<nodecontrol::GetConfigReply>, Status> {
        let _node_id = self
            .authenticate_meta(request.metadata().clone())
            .await?;
        Ok(Response::new(nodecontrol::GetConfigReply {
            payload: Vec::new(),
            version: 0,
        }))
    }

    async fn get_inbounds(
        &self,
        request: Request<nodecontrol::GetInboundsRequest>,
    ) -> Result<Response<nodecontrol::GetInboundsReply>, Status> {
        let _node_id = self
            .authenticate_meta(request.metadata().clone())
            .await?;
        Ok(Response::new(nodecontrol::GetInboundsReply {
            payload: Vec::new(),
            version: 0,
        }))
    }

    async fn get_outbounds(
        &self,
        request: Request<nodecontrol::GetOutboundsRequest>,
    ) -> Result<Response<nodecontrol::GetOutboundsReply>, Status> {
        let _node_id = self
            .authenticate_meta(request.metadata().clone())
            .await?;
        Ok(Response::new(nodecontrol::GetOutboundsReply {
            payload: Vec::new(),
            version: 0,
        }))
    }

    async fn get_routing(
        &self,
        request: Request<nodecontrol::GetRoutingRequest>,
    ) -> Result<Response<nodecontrol::GetRoutingReply>, Status> {
        let _node_id = self
            .authenticate_meta(request.metadata().clone())
            .await?;
        Ok(Response::new(nodecontrol::GetRoutingReply {
            payload: Vec::new(),
            version: 0,
        }))
    }
}

impl NodeControlService {
    pub fn new(admin_config: AdminConfigStore) -> Self {
        Self { admin_config }
    }

    async fn authenticate_meta(&self, meta: MetadataMap) -> Result<u64, Status> {
        let node_id: u64 = meta
            .get("x-node-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| Status::unauthenticated("missing x-node-id"))?;

        let token = meta
            .get("x-node-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let ts: u64 = meta
            .get("x-ts")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| Status::unauthenticated("missing x-ts"))?;
        let nonce = meta
            .get("x-nonce")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("missing x-nonce"))?;
        let sign = meta
            .get("x-sign")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("missing x-sign"))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Status::internal("time error"))?
            .as_secs();
        let skew = now.abs_diff(ts);
        if skew > 300 {
            return Err(Status::unauthenticated("timestamp skew too large"));
        }

        let nonce_key = format!("{}:{}", node_id, nonce);
        if NONCE_CACHE.get(&nonce_key).await.is_some() {
            return Err(Status::unauthenticated("replayed nonce"));
        }
        NONCE_CACHE.insert(nonce_key, ()).await;

        let node = admin_node_config::Entity::find_by_id(node_id)
            .one(self.admin_config.db())
            .await
            .map_err(|e| Status::internal(format!("db error: {}", e)))?
            .ok_or_else(|| Status::unauthenticated("node not found"))?;

        if let Some(expected_token) = node.node_token.as_deref().filter(|s| !s.is_empty()) {
            if token != expected_token {
                return Err(Status::unauthenticated("invalid token"));
            }
        }

        let Some(shared_secret) = node
            .node_shared_secret
            .as_deref()
            .filter(|s| !s.is_empty())
        else {
            return Ok(node_id);
        };

        let base_string = format!(
            "{}\n{}\n{}\n{}",
            node_id,
            token,
            ts,
            nonce
        );
        let mut mac = Hmac::<Sha256>::new_from_slice(shared_secret.as_bytes())
            .map_err(|_| Status::internal("invalid shared_secret"))?;
        mac.update(base_string.as_bytes());
        let expected = hex::encode(mac.finalize().into_bytes());

        if !expected.eq_ignore_ascii_case(sign) {
            return Err(Status::unauthenticated("invalid signature"));
        }

        Ok(node_id)
    }
}

#[tonic::async_trait]
impl nodepanel::node_panel_server::NodePanel for NodePanelService {
    async fn ping(
        &self,
        request: Request<nodepanel::PingRequest>,
    ) -> Result<Response<nodepanel::PingReply>, Status> {
        let node_id = self.authenticate(&request).await?;
        let req = request.into_inner();
        info!("[grpc] ping authed node_id={} req.node_id={}", node_id, req.node_id);
        let msg = format!("pong: node_id={} message={}", req.node_id, req.message);
        Ok(Response::new(nodepanel::PingReply { message: msg }))
    }
}

pub async fn serve_grpc(
    addr: std::net::SocketAddr,
    admin_config: AdminConfigStore,
) -> Result<(), Box<dyn std::error::Error>> {
    let admin_config_for_panel = admin_config.clone();
    let admin_config_for_control = admin_config.clone();

    tonic::transport::Server::builder()
        .add_service(nodepanel::node_panel_server::NodePanelServer::new(
            NodePanelService::new(admin_config_for_panel),
        ))
        .add_service(nodecontrol::node_control_service_server::NodeControlServiceServer::new(
            NodeControlService::new(admin_config_for_control),
        ))
        .serve(addr)
        .await?;
    Ok(())
}
