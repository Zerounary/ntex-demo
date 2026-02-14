mod application;
mod cdk;
mod config;
mod domain;
mod email;
mod infrastructure;
mod interface;

use dotenv::dotenv;
use env_logger::Env;
use infrastructure::{admin_config, database, seed};
use interface::{admin, grpc_server, tls_entry};
use log::{error, info, warn};
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Once};

use crate::infrastructure::node_transport::{GrpcTransport, NodeTransport};

use crate::config::AppConfig;

use crate::infrastructure::persistence::{
    acceleration_session, node_online_user_log, node_traffic_log, user_wallet,
};

#[ntex::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    ensure_rustls_crypto_provider();
    // 配置日志过滤器：降低 rumqttd 路由模块的日志级别，避免正常路由信息被记录为 ERROR
    // 如果环境变量 RUST_LOG 未设置，使用默认值：全局 info，rumqttd 相关模块为 warn
    let default_filter = "info,rumqttd::router::routing=off,rumqttd=off";
    env_logger::Builder::from_env(Env::default().default_filter_or(default_filter)).init();
    let config = AppConfig::from_env();

    // 首先检测数据库连接
    info!("正在检测数据库连接...");
    let db = match database::connect(&config.database_url).await {
        Ok(db) => {
            info!("数据库连接成功");
            db
        },
        Err(e) => {
            error!("数据库连接失败: {}", e);
            error!("数据库 URL: {}", config.database_url);
            error!("请检查数据库配置和连接信息，程序将退出");
            std::process::exit(1);
        }
    };

    // 初始化数据库表结构
    info!("正在初始化数据库表结构...");
    if let Err(e) = database::init(&db).await {
        error!("数据库初始化失败: {}", e);
        error!("请检查数据库权限和配置，程序将退出");
        std::process::exit(1);
    }
    info!("数据库表结构初始化成功");

    // 执行数据库种子数据
    info!("正在执行数据库种子数据...");
    if let Err(e) = seed::seed(&db).await {
        error!("数据库种子数据执行失败: {}", e);
        error!("程序将退出");
        std::process::exit(1);
    }
    info!("数据库种子数据执行成功");
    
    // 创建管理配置存储
    let admin_config = admin_config::AdminConfigStore::new(db.clone());

    let node_transport: Arc<dyn NodeTransport> = Arc::new(GrpcTransport::new());

    // 启动 gRPC 服务端骨架（仅本机监听，供未来 443 TLS 入口转发）
    {
        let grpc_addr: SocketAddr = env::var("GRPC_LISTEN_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:50051".to_string())
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("invalid GRPC_LISTEN_ADDR: {}", e)))?;
        let admin_config_clone = admin_config.clone();
        tokio::spawn(async move {
            info!("[grpc] starting grpc server on {}", grpc_addr);
            if let Err(e) = grpc_server::serve_grpc(grpc_addr, admin_config_clone).await {
                error!("[grpc] server error: {}", e);
            }
        });
    }

    // 可选启动 443 TLS 入口（默认关闭，避免绑定 443 失败）
    if env::var("ENABLE_TLS_ENTRY").ok().as_deref() == Some("1") {
        let tls_addr: SocketAddr = env::var("TLS_ENTRY_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:443".to_string())
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("invalid TLS_ENTRY_ADDR: {}", e)))?;
        tokio::spawn(async move {
            if let Err(e) = tls_entry::serve_tls_entry(tls_addr).await {
                error!("[tls-entry] server error: {}", e);
            }
        });
    }

    // 启动分钟计费后台任务（不依赖客户端心跳）
    {
        let db_clone = db.clone();
        let admin_config_clone = admin_config.clone();
        let transport_clone = node_transport.clone();
        tokio::spawn(async move {
            run_minute_billing_daemon(db_clone, admin_config_clone, transport_clone).await;
        });
    }
    
    // 启动管理服务器
    let admin_config_clone = admin_config.clone();
    let db_clone = db.clone();

    let admin_port: u16 = env::var("ADMIN_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(667);

    info!("正在启动管理服务器 (端口 {})...", admin_port);
    
    // 使用 tokio::select! 运行管理服务器，并监听关闭信号
    tokio::select! {
        result = admin::serve(admin_port, admin_config_clone, db_clone) => {
            if let Err(e) = result {
                eprintln!("管理服务器错误: {:?}", e);
            } else {
                info!("管理服务器已关闭");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("收到 Ctrl+C 信号，正在关闭服务器...");
        }
    }

    Ok(())
}

fn to_io_error(err: impl std::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
}

fn ensure_rustls_crypto_provider() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("failed to install rustls ring crypto provider");
    });
}

async fn run_minute_billing_daemon(
    db: sea_orm::DatabaseConnection,
    admin_config: admin_config::AdminConfigStore,
    node_transport: Arc<dyn NodeTransport>,
) {
    async fn revoke_session_users(
        admin_config: &admin_config::AdminConfigStore,
        node_transport: &dyn NodeTransport,
        session: &acceleration_session::Model,
    ) {
        let mut targets: Vec<(u64, u64)> = Vec::new();
        if session.admin_user_id > 0 {
            targets.push((session.node_id, session.admin_user_id));
        }
        if let (Some(node_id), Some(admin_user_id)) = (session.tcp_node_id, session.tcp_admin_user_id)
        {
            if admin_user_id > 0 {
                targets.push((node_id, admin_user_id));
            }
        }
        if let (Some(node_id), Some(admin_user_id)) = (session.udp_node_id, session.udp_admin_user_id)
        {
            if admin_user_id > 0 {
                targets.push((node_id, admin_user_id));
            }
        }

        targets.sort();
        targets.dedup();

        for (node_id, admin_user_id) in targets {
            let _ = admin_config.delete_user(node_id, admin_user_id).await;
            let _ = node_transport
                .publish_update_notification(node_id, "user")
                .await;
            let _ = node_transport
                .publish_update_notification(node_id, "outbound")
                .await;
            let _ = node_transport
                .publish_update_notification(node_id, "inbound")
                .await;
            let _ = node_transport
                .publish_update_notification(node_id, "config")
                .await;
        }
    }

    let tick_seconds: u64 = env::var("BILLING_TICK_SECONDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);

    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(tick_seconds));

    loop {
        ticker.tick().await;

        let now = chrono::Utc::now();

        let sessions = match acceleration_session::Entity::find()
            .filter(acceleration_session::Column::Status.eq("active"))
            .filter(acceleration_session::Column::BillType.eq("minute"))
            .order_by_asc(acceleration_session::Column::LastAccountedAt)
            .all(&db)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                error!("[billing] query active minute sessions failed: {}", e);
                continue;
            }
        };

        for session in sessions {
            let session_id = session.session_id.clone();
            let node_id = session.node_id;
            let admin_user_id = session.admin_user_id;
            let user_id = session.user_id.clone();
            let last_accounted_at: chrono::DateTime<chrono::Utc> = session.last_accounted_at.into();
            let last_activity_at: chrono::DateTime<chrono::Utc> = session.last_activity_at.into();
            let started_at: chrono::DateTime<chrono::Utc> = session.started_at.into();

            // 基于节点 onlineusers 上报判断是否仍在线（不依赖客户端上报）
            let last_seen: Option<chrono::DateTime<chrono::Utc>> = match node_online_user_log::Entity::find()
                .filter(node_online_user_log::Column::NodeId.eq(node_id))
                .filter(node_online_user_log::Column::UserId.eq(admin_user_id))
                .order_by_desc(node_online_user_log::Column::CreatedAt)
                .one(&db)
                .await
            {
                Ok(v) => v.map(|m| m.created_at.into()),
                Err(e) => {
                    error!(
                        "[billing] query online user log failed: session_id={}, err={}",
                        session_id, e
                    );
                    None
                }
            };

            let started_at_db: sea_orm::prelude::DateTimeUtc = started_at.into();
            let has_traffic = match node_traffic_log::Entity::find()
                .filter(node_traffic_log::Column::NodeId.eq(node_id))
                .filter(node_traffic_log::Column::UserId.eq(admin_user_id))
                .filter(node_traffic_log::Column::CreatedAt.gte(started_at_db))
                .order_by_desc(node_traffic_log::Column::CreatedAt)
                .one(&db)
                .await
            {
                Ok(Some(m)) => m.upload > 0 || m.download > 0,
                Ok(None) => false,
                Err(e) => {
                    warn!(
                        "[billing] query traffic log failed: session_id={}, err={}",
                        session_id, e
                    );
                    false
                }
            };

            let last_activity_ts = match last_seen.clone() {
                Some(last_seen_ts) => {
                    if last_seen_ts > last_activity_at {
                        let mut s = acceleration_session::ActiveModel {
                            session_id: Set(session_id.clone()),
                            ..Default::default()
                        };
                        s.last_activity_at = Set(last_seen_ts.into());
                        s.updated_at = Set(now.into());
                        let _ = s.update(&db).await;
                        last_seen_ts
                    } else {
                        last_activity_at
                    }
                }
                None => last_activity_at,
            };

            let elapsed = now.signed_duration_since(last_accounted_at);
            let minutes_to_charge: i64 = elapsed.num_minutes();
            if minutes_to_charge <= 0 {
                continue;
            }

            let txn = match db.begin().await {
                Ok(t) => t,
                Err(e) => {
                    error!("[billing] begin txn failed: session_id={}, err={}", session_id, e);
                    continue;
                }
            };

            // 幂等/并发保护：在事务内重新读取 session，确认仍为 active 且 last_accounted_at 未变化
            let session_in_txn = match acceleration_session::Entity::find_by_id(session_id.clone())
                .one(&txn)
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    error!("[billing] session recheck failed: session_id={}, err={}", session_id, e);
                    let _ = txn.rollback().await;
                    continue;
                }
            };

            let Some(session_in_txn) = session_in_txn else {
                let _ = txn.rollback().await;
                continue;
            };

            if session_in_txn.status != "active" {
                let _ = txn.rollback().await;
                continue;
            }

            if session_in_txn.last_accounted_at != session.last_accounted_at {
                let _ = txn.rollback().await;
                continue;
            }

            let wallet = match user_wallet::Entity::find_by_id(user_id.clone()).one(&txn).await {
                Ok(v) => v,
                Err(e) => {
                    error!("[billing] wallet query failed: user_id={}, err={}", user_id, e);
                    let _ = txn.rollback().await;
                    continue;
                }
            };

            let mut should_revoke = false;
            let mut status_after: Option<String> = None;

            let Some(wallet) = wallet else {
                let last_activity_ts = last_seen.unwrap_or(now);
                let updated = match acceleration_session::Entity::update_many()
                    .col_expr(
                        acceleration_session::Column::Status,
                        Expr::value("insufficient_balance"),
                    )
                    .col_expr(acceleration_session::Column::EndedAt, Expr::value(now))
                    .col_expr(
                        acceleration_session::Column::LastActivityAt,
                        Expr::value(last_activity_ts),
                    )
                    .col_expr(acceleration_session::Column::UpdatedAt, Expr::value(now))
                    .filter(acceleration_session::Column::SessionId.eq(session_id.clone()))
                    .filter(acceleration_session::Column::Status.eq("active"))
                    .filter(acceleration_session::Column::LastAccountedAt.eq(session.last_accounted_at))
                    .exec(&txn)
                    .await
                {
                    Ok(r) => r.rows_affected,
                    Err(e) => {
                        error!(
                            "[billing] session stop(insufficient, no wallet) update failed: session_id={}, err={}",
                            session_id, e
                        );
                        0
                    }
                };

                if updated == 0 {
                    let _ = txn.rollback().await;
                    continue;
                }

                if txn.commit().await.is_err() {
                    continue;
                }

                should_revoke = true;
                status_after = Some("insufficient_balance".to_string());
                revoke_session_users(&admin_config, node_transport.as_ref(), &session).await;
                info!(
                    "[billing] session stopped: session_id={}, status={}",
                    session_id,
                    status_after.unwrap()
                );
                continue;
            };

            let remaining = wallet.remaining_minutes;
            if remaining <= 0 {
                let last_activity_ts = last_seen.unwrap_or(now);
                let updated = match acceleration_session::Entity::update_many()
                    .col_expr(
                        acceleration_session::Column::Status,
                        Expr::value("insufficient_balance"),
                    )
                    .col_expr(acceleration_session::Column::EndedAt, Expr::value(now))
                    .col_expr(
                        acceleration_session::Column::LastActivityAt,
                        Expr::value(last_activity_ts),
                    )
                    .col_expr(acceleration_session::Column::UpdatedAt, Expr::value(now))
                    .filter(acceleration_session::Column::SessionId.eq(session_id.clone()))
                    .filter(acceleration_session::Column::Status.eq("active"))
                    .filter(acceleration_session::Column::LastAccountedAt.eq(session.last_accounted_at))
                    .exec(&txn)
                    .await
                {
                    Ok(r) => r.rows_affected,
                    Err(e) => {
                        error!(
                            "[billing] session stop(insufficient, empty wallet) update failed: session_id={}, err={}",
                            session_id, e
                        );
                        0
                    }
                };

                if updated == 0 {
                    let _ = txn.rollback().await;
                    continue;
                }

                if txn.commit().await.is_err() {
                    continue;
                }

                revoke_session_users(&admin_config, node_transport.as_ref(), &session).await;
                continue;
            }

            let charge = minutes_to_charge.min(remaining);

            let mut w: user_wallet::ActiveModel = wallet.into();
            w.remaining_minutes = Set(remaining - charge);
            w.updated_at = Set(now.into());
            if w.update(&txn).await.is_err() {
                let _ = txn.rollback().await;
                continue;
            }

            let last_activity_ts = last_seen.unwrap_or(now);
            let new_last_accounted_at = last_accounted_at + chrono::Duration::minutes(charge);
            let new_billed = session_in_txn.billed_minutes + charge;

            if charge < minutes_to_charge {
                should_revoke = true;
                status_after = Some("insufficient_balance".to_string());
            }

            let mut updater = acceleration_session::Entity::update_many()
                .col_expr(acceleration_session::Column::BilledMinutes, Expr::value(new_billed))
                .col_expr(
                    acceleration_session::Column::LastAccountedAt,
                    Expr::value(new_last_accounted_at),
                )
                .col_expr(
                    acceleration_session::Column::LastActivityAt,
                    Expr::value(last_activity_ts),
                )
                .col_expr(acceleration_session::Column::UpdatedAt, Expr::value(now));

            if should_revoke {
                updater = updater
                    .col_expr(
                        acceleration_session::Column::Status,
                        Expr::value("insufficient_balance"),
                    )
                    .col_expr(acceleration_session::Column::EndedAt, Expr::value(now));
            }

            let updated = match updater
                .filter(acceleration_session::Column::SessionId.eq(session_id.clone()))
                .filter(acceleration_session::Column::Status.eq("active"))
                .filter(acceleration_session::Column::LastAccountedAt.eq(session.last_accounted_at))
                .exec(&txn)
                .await
            {
                Ok(r) => r.rows_affected,
                Err(e) => {
                    error!("[billing] session billing update failed: session_id={}, err={}", session_id, e);
                    0
                }
            };

            if updated == 0 {
                let _ = txn.rollback().await;
                continue;
            }

            if txn.commit().await.is_err() {
                continue;
            }

            if should_revoke {
                revoke_session_users(&admin_config, node_transport.as_ref(), &session).await;
                info!(
                    "[billing] session stopped: session_id={}, status=insufficient_balance",
                    session_id
                );
            } else {
                info!(
                    "[billing] session billed: session_id={}, minutes_charged={}, remaining_minutes={}",
                    session_id,
                    minutes_to_charge,
                    remaining - charge
                );
            }
        }
    }
}
