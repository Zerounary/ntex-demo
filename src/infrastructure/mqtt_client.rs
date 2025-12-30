//! MQTT 客户端模块
//! 
//! 用于连接 MQTT broker 并接收节点上报的数据

use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, QoS, Transport};
use serde_json::Value;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::fs;
use log::{info, error, warn, debug};
use std::time::Instant;

use crate::infrastructure::admin_config::AdminConfigStore;
use sea_orm::DatabaseConnection;

/// 响应等待器
type ResponseWaiter = tokio::sync::oneshot::Sender<Value>;

/// MQTT 客户端管理器
pub struct MqttClientManager {
    client: Arc<AsyncClient>,
    shutdown_flag: Arc<tokio::sync::Notify>,
    response_waiters: Arc<Mutex<HashMap<String, ResponseWaiter>>>,
    presence: Arc<Mutex<HashMap<u64, NodePresence>>>,
    admin_config: AdminConfigStore,
    db: DatabaseConnection,
}

pub struct MqttPublisher {
    client: Arc<AsyncClient>,
    shutdown_flag: Arc<tokio::sync::Notify>,
}

#[derive(Clone, Copy, Debug)]
struct NodePresence {
    last_seen: Instant,
    is_online: bool,
}

impl MqttClientManager {
    /// 创建并启动 MQTT 客户端
    pub async fn start(admin_config: AdminConfigStore, db: DatabaseConnection) -> Result<Self, Box<dyn std::error::Error>> {
        match admin_config.set_all_nodes_offline().await {
            Ok(rows) => {
                info!("⚫ [Presence] 启动时已重置所有节点为离线: rows_affected={}", rows);
            }
            Err(e) => {
                warn!("⚠️  [Presence] 启动时重置节点离线失败（将继续运行）: {}", e);
            }
        }

        // 从环境变量获取配置
        let broker_host = env::var("MQTT_BROKER_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        let broker_port = env::var("MQTT_BROKER_PORT")
            .unwrap_or_else(|_| "1883".to_string())
            .parse::<u16>()
            .unwrap_or(1883);
        
        let client_id = format!("xrayr-manager-{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
        
        // 创建 MQTT 选项
        let mut mqttoptions = MqttOptions::new(client_id, broker_host.clone(), broker_port);
        mqttoptions.set_keep_alive(Duration::from_secs(60));
        mqttoptions.set_clean_session(true);
        
        // 设置用户名和密码认证（从环境变量读取，默认使用 manage 账号）
        let mqtt_username = env::var("MQTT_USERNAME")
            .unwrap_or_else(|_| "manage".to_string());
        let mqtt_password = env::var("MQTT_PASSWORD")
            .unwrap_or_else(|_| "manage_password_123".to_string());
        mqttoptions.set_credentials(&mqtt_username, &mqtt_password);
        info!("🔐 [MQTT] 已设置用户名密码认证: username={}", mqtt_username);
        
        // 设置最大包大小（rumqttc 默认可能是 16KB，我们需要支持更大的消息，如 100MB）
        // 注意：rumqttc 0.25.1 使用 set_max_packet_size 方法设置最大包大小
        // 参数：incoming (接收消息的最大包大小), outgoing (发送消息的最大包大小)
        // 这个限制影响接收消息的大小，需要设置为足够大的值以支持日志查询响应
        let max_packet_size = 100 * 1024 * 1024; // 100MB，与 broker 配置一致
        mqttoptions.set_max_packet_size(max_packet_size, max_packet_size);
        info!("✅ [MQTT] 已设置最大包大小: incoming={} bytes, outgoing={} bytes", max_packet_size, max_packet_size);
        
        // 配置 TLS（如果提供了证书路径）
        let tls_ca_path = env::var("MQTT_TLS_CA")
            .or_else(|_| {
                // 尝试从 config.toml 读取 CA 证书路径
                if let Ok(content) = fs::read_to_string("config.toml") {
                    if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                        if let Some(capath) = config
                            .get("v4")
                            .and_then(|v4| v4.get("v4-1"))
                            .and_then(|v4_1| v4_1.get("tls"))
                            .and_then(|tls| tls.get("capath"))
                            .and_then(|v| v.as_str())
                        {
                            return Ok(capath.to_string());
                        }
                    }
                }
                Err(env::VarError::NotPresent)
            })
            .ok();
        
        if let Some(ca_path) = &tls_ca_path {
            info!("🔒 [MQTT] 配置 TLS 连接，CA 证书: {}", ca_path);
            
            // 可选：如果提供了客户端证书和密钥，配置客户端证书认证
            let tls_cert_path = env::var("MQTT_TLS_CERT")
                .or_else(|_| {
                    if let Ok(content) = fs::read_to_string("config.toml") {
                        if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                            if let Some(certpath) = config
                                .get("v4")
                                .and_then(|v4| v4.get("v4-1"))
                                .and_then(|v4_1| v4_1.get("tls"))
                                .and_then(|tls| tls.get("certpath"))
                                .and_then(|v| v.as_str())
                            {
                                return Ok(certpath.to_string());
                            }
                        }
                    }
                    Err(env::VarError::NotPresent)
                })
                .ok();
            
            let tls_key_path = env::var("MQTT_TLS_KEY")
                .or_else(|_| {
                    if let Ok(content) = fs::read_to_string("config.toml") {
                        if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                            if let Some(keypath) = config
                                .get("v4")
                                .and_then(|v4| v4.get("v4-1"))
                                .and_then(|v4_1| v4_1.get("tls"))
                                .and_then(|tls| tls.get("keypath"))
                                .and_then(|v| v.as_str())
                            {
                                return Ok(keypath.to_string());
                            }
                        }
                    }
                    Err(env::VarError::NotPresent)
                })
                .ok();
            
            // 根据 rumqttc 0.25.1 的文档，TlsConfiguration 有两个变体：
            // 1. Simple { ca, alpn, client_auth }
            // 2. Rustls(Arc<ClientConfig>)
            // 我们使用 Simple 变体，它接受 CA 证书的字节
            let ca_cert_bytes = fs::read(ca_path)?;
            
            // 可选：如果提供了客户端证书和密钥，配置客户端证书认证
            let client_auth = if let (Some(cert_path), Some(key_path)) = (&tls_cert_path, &tls_key_path) {
                let cert_bytes = fs::read(cert_path)?;
                let key_bytes = fs::read(key_path)?;
                Some((cert_bytes, key_bytes))
            } else {
                None
            };
            
            // 创建 TlsConfiguration::Simple
            let has_client_auth = client_auth.is_some();
            let tls_config = rumqttc::TlsConfiguration::Simple {
                ca: ca_cert_bytes,
                alpn: None,
                client_auth,
            };
            
            mqttoptions.set_transport(Transport::tls_with_config(tls_config));
            if has_client_auth {
                info!("🔒 [MQTT] TLS 已启用（包含客户端证书认证）");
            } else {
                info!("🔒 [MQTT] TLS 已启用（仅 CA 证书验证）");
            }
        } else {
            info!("📡 [MQTT] 使用普通 TCP 连接（未启用 TLS）");
        }
        
        // 创建客户端和事件循环
        // 第二个参数是 channel capacity，增加它有助于处理更多的并发消息
        let (client, eventloop) = AsyncClient::new(mqttoptions, 100);
        let client_arc = Arc::new(client);
        
        // 订阅主题
        let request_topic = "xrayr/node/+/request/+";
        let response_topic = "xrayr/node/+/response/+";
        
        info!("📡 正在连接 MQTT Broker: {}:{}", broker_host, broker_port);
        
        // 创建关闭信号
        let shutdown_flag = Arc::new(tokio::sync::Notify::new());
        
        // 创建响应等待器映射
        let response_waiters = Arc::new(Mutex::new(HashMap::<String, ResponseWaiter>::new()));

        let presence = Arc::new(Mutex::new(HashMap::<u64, NodePresence>::new()));
        
        // 启动事件循环任务（必须在订阅之前启动，以便处理连接和订阅确认）
        let client_clone = client_arc.clone();
        let shutdown_clone = shutdown_flag.clone();
        let response_waiters_clone = response_waiters.clone();
        let admin_config_clone = admin_config.clone();
        let db_clone = db.clone();
        let presence_clone = presence.clone();
        tokio::spawn(async move {
            Self::run_event_loop(eventloop, client_clone, shutdown_clone, response_waiters_clone, presence_clone, admin_config_clone, db_clone).await;
        });

        let shutdown_clone = shutdown_flag.clone();
        let presence_clone = presence.clone();
        let admin_config_clone = admin_config.clone();
        tokio::spawn(async move {
            Self::run_presence_watchdog(shutdown_clone, presence_clone, admin_config_clone).await;
        });
        
        // 等待连接建立（给事件循环一些时间处理连接）
        time::sleep(Duration::from_millis(500)).await;
        
        // 订阅主题
        info!("📡 正在订阅 MQTT 主题: {}", request_topic);
        client_arc.subscribe(request_topic, QoS::AtLeastOnce).await?;
        info!("✅ 已发送订阅请求: {}", request_topic);
        
        info!("📡 正在订阅 MQTT 主题: {}", response_topic);
        client_arc.subscribe(response_topic, QoS::AtLeastOnce).await?;
        info!("✅ 已发送订阅请求: {}", response_topic);
        
        // 等待订阅确认（给 broker 一些时间处理订阅）
        time::sleep(Duration::from_millis(500)).await;
        info!("✅ 订阅完成，等待订阅确认...");
        
        Ok(Self {
            client: client_arc,
            shutdown_flag,
            response_waiters,
            presence,
            admin_config,
            db,
        })
    }
    
    /// 运行事件循环，处理消息
    async fn run_event_loop(
        mut eventloop: EventLoop,
        _client: Arc<AsyncClient>,
        shutdown_flag: Arc<tokio::sync::Notify>,
        response_waiters: Arc<Mutex<HashMap<String, ResponseWaiter>>>,
        presence: Arc<Mutex<HashMap<u64, NodePresence>>>,
        admin_config: AdminConfigStore,
        db: DatabaseConnection,
    ) {
        info!("🚀 MQTT 客户端事件循环已启动");
        
        loop {
            tokio::select! {
                // 处理 MQTT 事件
                event = eventloop.poll() => {
                    match event {
                        Ok(Event::Incoming(rumqttc::Packet::Publish(publish))) => {
                            info!("📨 [MQTT EventLoop] 收到发布消息: topic=\"{}\", payload_len={}", publish.topic, publish.payload.len());
                            // 检查是否是响应消息
                            if publish.topic.contains("/response/") {
                                info!("🔔 [MQTT EventLoop] 这是响应消息，准备处理...");
                            }
                            let client_clone = _client.clone();
                            let response_waiters_clone = response_waiters.clone();
                            let presence_clone = presence.clone();
                            // 检查 payload 大小（最大 100MB，与 broker 配置一致）
                            if publish.payload.len() > 104857600 {
                                warn!("⚠️  MQTT 消息 payload 大小超限: topic={}, size={} bytes (最大: 104857600 bytes)", 
                                      publish.topic, publish.payload.len());
                                // 尝试从 topic 解析信息并发送错误响应
                                Self::handle_oversized_message(client_clone, publish.topic.clone(), publish.payload.len()).await;
                            } else {
                                let admin_config_clone = admin_config.clone();
                                let db_clone = db.clone();
                                Self::handle_message(client_clone, response_waiters_clone, presence_clone, admin_config_clone, db_clone, publish.topic, publish.payload).await;
                            }
                        }
                        Ok(Event::Incoming(rumqttc::Packet::ConnAck(_))) => {
                            info!("✅ MQTT 连接已确认");
                        }
                        Ok(Event::Incoming(rumqttc::Packet::Disconnect)) => {
                            warn!("⚠️  MQTT 连接已断开");
                        }
                        Ok(Event::Incoming(rumqttc::Packet::SubAck(suback))) => {
                            info!("✅ [MQTT EventLoop] 订阅确认: {:?}", suback);
                        }
                        Ok(Event::Incoming(rumqttc::Packet::PubAck(_))) => {
                            debug!("✅ MQTT 发布确认");
                        }
                        Ok(Event::Incoming(packet)) => {
                            info!("📦 [MQTT EventLoop] 收到其他数据包: {:?}", packet);
                        }
                        Ok(Event::Outgoing(_)) => {
                            // 忽略出站消息
                        }
                        Err(e) => {
                            let error_str = e.to_string();
                            // 检查是否是 payload 大小超限错误
                            if error_str.contains("payload size limit exceeded") 
                                || error_str.contains("serialization/deserialization error") {
                                error!("❌ [MQTT EventLoop] MQTT 消息 payload 大小超限，消息被丢弃: {}", error_str);
                                
                                // 尝试从错误信息中提取 payload 大小
                                let payload_size = if let Some(size_str) = error_str.split("exceeded: ").nth(1) {
                                    size_str.trim().parse::<usize>().ok()
                                } else {
                                    None
                                };
                                
                                if let Some(size) = payload_size {
                                    error!("❌ [MQTT EventLoop] 检测到 payload 大小: {} bytes，超过了 rumqttc 的默认限制（约 16KB）", size);
                                    error!("❌ [MQTT EventLoop] 这是导致日志查询超时的根本原因！");
                                    error!("❌ [MQTT EventLoop] 解决方案：需要增加 rumqttc 的 max packet size 配置");
                                    
                                    // 由于 rumqttc 在解析阶段就报错了，我们无法获取到完整的消息和 topic
                                    // 但是，我们可以尝试从最近发送的请求中推断这可能是一个响应消息
                                    // 通知所有等待的请求可能超时了（由于 payload 大小限制）
                                    let waiters = response_waiters.lock().await;
                                    if !waiters.is_empty() {
                                        let waiting_ids: Vec<String> = waiters.keys().cloned().collect();
                                        error!("⚠️  [MQTT EventLoop] 当前有 {} 个等待中的请求可能受到影响: {:?}", 
                                               waiting_ids.len(), waiting_ids);
                                        // 注意：我们不能直接通知等待器，因为我们需要完整的响应数据
                                        // 但是我们可以记录这个错误，帮助用户诊断问题
                                    }
                                }
                                
                                warn!("⚠️  客户端连接状态正常，可以继续发送消息");
                                // 对于 payload 大小错误，直接继续，不等待
                                // 注意：rumqttc 的 AsyncClient 和 EventLoop 是分离的，
                                // 即使事件循环出错，客户端仍可以发送消息
                            } else {
                                error!("❌ MQTT 事件循环错误: {}", e);
                                error!("⚠️  等待 1 秒后继续，客户端连接状态可能受影响");
                                // 对于其他错误，等待一段时间后继续
                                time::sleep(Duration::from_secs(1)).await;
                            }
                            // 继续循环，不中断事件循环
                        }
                    }
                }
                // 检查关闭信号
                _ = shutdown_flag.notified() => {
                    info!("🛑 收到关闭信号，停止 MQTT 客户端");
                    break;
                }
            }
        }
        
        info!("✅ MQTT 客户端事件循环已停止");
    }

    async fn run_presence_watchdog(
        shutdown_flag: Arc<tokio::sync::Notify>,
        presence: Arc<Mutex<HashMap<u64, NodePresence>>>,
        admin_config: AdminConfigStore,
    ) {
        let offline_after = Duration::from_secs(
            env::var("NODE_OFFLINE_AFTER_SECONDS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60),
        );

        let mut ticker = time::interval(Duration::from_secs(10));
        loop {
            tokio::select! {
                _ = shutdown_flag.notified() => {
                    break;
                }
                _ = ticker.tick() => {
                    let mut to_mark_offline: Vec<u64> = Vec::new();
                    {
                        let now = Instant::now();
                        let mut map = presence.lock().await;
                        for (node_id, p) in map.iter_mut() {
                            if p.is_online && now.duration_since(p.last_seen) > offline_after {
                                p.is_online = false;
                                to_mark_offline.push(*node_id);
                            }
                        }
                    }

                    for node_id in to_mark_offline {
                        if let Err(e) = admin_config.set_node_online_status(node_id, false).await {
                            warn!("⚠️  [Presence] 标记节点离线失败: node_id={}, error={}", node_id, e);
                        } else {
                            info!("⚫ [Presence] 节点已离线: node_id={}", node_id);
                        }
                    }
                }
            }
        }
    }
    
    /// 处理超大消息（payload 大小超限）
    async fn handle_oversized_message(
        client: Arc<AsyncClient>,
        topic: String,
        payload_size: usize,
    ) {
        // 解析主题: xrayr/node/{node_id}/request/{request_id} 或 xrayr/node/{node_id}/response/{request_id}
        let topic_parts: Vec<&str> = topic.split('/').collect();
        if topic_parts.len() != 5 {
            warn!("⚠️  超大消息，但主题格式无效: {}", topic);
            return;
        }
        
        let node_id = topic_parts[2];
        let topic_type = topic_parts[3]; // 'request' 或 'response'
        let request_id = topic_parts[4];
        
        // 只处理请求消息，响应消息不需要发送响应
        if topic_type != "request" {
            warn!("⚠️  超大消息是响应类型，无需处理: topic={}", topic);
            return;
        }
        
        // 检查是否是管理端自己发送的请求
        if request_id.starts_with("log_query_")
            || request_id.starts_with("udp_probe_")
            || request_id.starts_with("query_network_")
        {
            warn!("⚠️  超大消息是管理端发送的请求，无需响应: topic={}", topic);
            return;
        }
        
        // 发送错误响应
        let response_topic = format!("xrayr/node/{}/response/{}", node_id, request_id);
        let error_response = serde_json::json!({
            "msg": "error",
            "error": format!("消息 payload 大小超限: {} bytes，最大允许: 104857600 bytes", payload_size)
        });
        
        let response_json = serde_json::to_string(&error_response)
            .unwrap_or_else(|_| r#"{"msg":"error","error":"消息 payload 大小超限"}"#.to_string());
        
        match client.publish(&response_topic, QoS::AtLeastOnce, false, response_json.as_bytes()).await {
            Ok(_) => {
                warn!("✅ [MQTT] 已发送 payload 超限错误响应: {}", response_topic);
            }
            Err(e) => {
                error!("❌ [MQTT] 发送 payload 超限错误响应失败: {}, 错误: {}", response_topic, e);
            }
        }
    }
    
    /// 处理收到的消息
    async fn handle_message(
        client: Arc<AsyncClient>,
        response_waiters: Arc<Mutex<HashMap<String, ResponseWaiter>>>,
        presence: Arc<Mutex<HashMap<u64, NodePresence>>>,
        admin_config: AdminConfigStore,
        _db: DatabaseConnection,
        topic: String,
        payload: bytes::Bytes,
    ) {
        // 解析主题: xrayr/node/{node_id}/request/{request_id} 或 xrayr/node/{node_id}/response/{request_id}
        let topic_parts: Vec<&str> = topic.split('/').collect();
        if topic_parts.len() != 5 {
            warn!("⚠️  无效的 MQTT 主题格式: {}", topic);
            return;
        }
        
        let node_id = topic_parts[2];
        let topic_type = topic_parts[3]; // 'request' 或 'response'
        let request_id = topic_parts[4];
        
        let node_id_u64_from_topic = node_id.parse::<u64>().ok();
        if let Some(node_id_u64) = node_id_u64_from_topic {
            let mut should_set_online = false;
            {
                let mut map = presence.lock().await;
                let entry = map.entry(node_id_u64).or_insert(NodePresence {
                    last_seen: Instant::now(),
                    is_online: false,
                });
                entry.last_seen = Instant::now();
                if !entry.is_online {
                    entry.is_online = true;
                    should_set_online = true;
                }
            }

            if should_set_online {
                if let Err(e) = admin_config.set_node_online_status(node_id_u64, true).await {
                    warn!("⚠️  [Presence] 标记节点在线失败: node_id={}, error={}", node_id_u64, e);
                } else {
                    info!("🟢 [Presence] 节点已在线: node_id={}", node_id_u64);
                }
            }
        }

        // 解析 JSON 负载
        let payload_str = match String::from_utf8(payload.to_vec()) {
            Ok(s) => s,
            Err(e) => {
                error!("❌ 无法解析消息负载为 UTF-8: {}", e);
                return;
            }
        };
        
        let data: Value = match serde_json::from_str(&payload_str) {
            Ok(d) => d,
            Err(e) => {
                error!("❌ 无法解析消息负载为 JSON: {}", e);
                return;
            }
        };
        
        // 如果是响应消息
        if topic_type == "response" {
            info!("📥 [MQTT Response] 收到响应消息: topic={}, node_id={}, request_id={}", topic, node_id, request_id);
            info!("📥 [MQTT Response] payload 长度: {} bytes", payload.len());
            debug!("📥 [MQTT Response] 完整消息内容: {}", serde_json::to_string_pretty(&data).unwrap_or_default());
            
            // 检查是否有等待此响应的等待器
            let mut waiters = response_waiters.lock().await;
            
            // 调试：打印所有等待的 request_id
            if waiters.is_empty() {
                warn!("⚠️  [MQTT Response] 没有等待中的请求 (request_id={})", request_id);
            } else {
                let waiting_ids: Vec<String> = waiters.keys().cloned().collect();
                info!("🔍 [MQTT Response] 当前等待的 request_id 列表: {:?}, 收到的 request_id: \"{}\"", waiting_ids, request_id);
                
                // 详细比较 request_id
                for waiting_id in &waiting_ids {
                    if waiting_id == request_id {
                        info!("✅ [MQTT Response] 找到匹配的 request_id: \"{}\"", waiting_id);
                    } else {
                        debug!("🔍 [MQTT Response] 不匹配: 等待的=\"{}\" (len={}), 收到的=\"{}\" (len={})", 
                               waiting_id, waiting_id.len(), request_id, request_id.len());
                    }
                }
            }
            
            // 将 request_id 转换为 String 以确保类型匹配
            let request_id_str = request_id.to_string();
            
            // 尝试精确匹配
            if let Some(waiter) = waiters.remove(&request_id_str) {
                info!("✅ [MQTT Response] 找到等待器，发送响应: request_id=\"{}\"", request_id);
                if waiter.send(data.clone()).is_err() {
                    warn!("⚠️  [MQTT Response] 发送响应到等待器失败: request_id=\"{}\"", request_id);
                } else {
                    info!("✅ [MQTT Response] 响应已成功发送到等待器: request_id=\"{}\"", request_id);
                }
            } else {
                // 没有等待器，可能是节点主动发送的响应，打印完整内容
                warn!("⚠️  [MQTT Response] 未找到对应的等待器: request_id=\"{}\"", request_id);
                warn!("⚠️  [MQTT Response] 可能的原因：1) 请求已超时 2) request_id 不匹配 3) 响应到达太晚");
                // 尝试打印更详细的调试信息
                let waiting_ids: Vec<String> = waiters.keys().cloned().collect();
                warn!("🔍 [MQTT Response] 当前等待的 request_id 列表: {:?}", waiting_ids);
                warn!("🔍 [MQTT Response] 收到的 request_id (字符串): \"{}\"", request_id);
                warn!("🔍 [MQTT Response] 收到的 request_id (长度): {}", request_id.len());
                warn!("🔍 [MQTT Response] 收到的 request_id (字节): {:?}", request_id.as_bytes());
                if !waiting_ids.is_empty() {
                    warn!("🔍 [MQTT Response] 第一个等待的 request_id: \"{}\" (长度: {})", waiting_ids[0], waiting_ids[0].len());
                    warn!("🔍 [MQTT Response] 第一个等待的 request_id (字节): {:?}", waiting_ids[0].as_bytes());
                    warn!("🔍 [MQTT Response] request_id 是否相等: {}", request_id == waiting_ids[0]);
                    // 尝试逐个字符比较
                    let received_chars: Vec<char> = request_id.chars().collect();
                    let waiting_chars: Vec<char> = waiting_ids[0].chars().collect();
                    if received_chars.len() == waiting_chars.len() {
                        for (i, (rc, wc)) in received_chars.iter().zip(waiting_chars.iter()).enumerate() {
                            if rc != wc {
                                warn!("🔍 [MQTT Response] 第 {} 个字符不同: 收到的='{}' (U+{:04X}), 等待的='{}' (U+{:04X})", 
                                      i, rc, *rc as u32, wc, *wc as u32);
                            }
                        }
                    }
                }
                println!("📥 [MQTT Response] 完整消息内容:");
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
            return;
        }
        
        // 处理请求消息
        let action = data.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        // 检查是否是管理端自己发送的请求（通过 request_id 前缀判断）
        // 管理端发送的请求使用特定前缀：log_query_, udp_probe_, query_network_
        if request_id.starts_with("log_query_")
            || request_id.starts_with("udp_probe_")
            || request_id.starts_with("query_network_")
        {
            // 这是管理端自己发送的请求，应该由节点处理，管理端忽略
            debug!("⚠️  [MQTT] 收到管理端自己发送的请求，忽略: action={}, request_id={}", action, request_id);
            return;
        }
        
        info!("📨 [MQTT Request] node_id={}, action={}, request_id={}", node_id, action, request_id);
        
        // 解析 node_id 为 u64
        let node_id_u64 = match node_id.parse::<u64>() {
            Ok(id) => id,
            Err(e) => {
                error!("❌ [MQTT Request] 无效的 node_id: {}, 错误: {}", node_id, e);
                // 发送错误响应
                let response_topic = format!("xrayr/node/{}/response/{}", node_id, request_id);
                let error_response = serde_json::json!({
                    "msg": "error",
                    "error": format!("无效的 node_id: {}", node_id)
                });
                let response_json = serde_json::to_string(&error_response)
                    .unwrap_or_else(|_| r#"{"msg":"error","error":"序列化响应失败"}"#.to_string());
                if let Err(e) = client.publish(&response_topic, QoS::AtLeastOnce, false, response_json.as_bytes()).await {
                    error!("❌ [MQTT] 发送错误响应失败: {}, 错误: {}", response_topic, e);
                }
                return;
            }
        };
        
        // 生成响应（异步方法）
        // 对于 config 请求，需要检查是否包含硬件信息
        if action == "config" {
            // 检查请求数据中是否包含硬件信息
            if let Some(request_data) = data.get("data") {
                if let Err(e) = admin_config.update_node_hardware_info(node_id_u64, request_data).await {
                    warn!("⚠️  [MQTT Request] 更新节点硬件信息失败: {}", e);
                }
            }
        }
        
        let response = Self::handle_request(action, node_id_u64, &admin_config).await;
        
        // 如果返回 None，表示应该由节点端处理，管理端不发送响应
        if response.is_none() {
            info!("⚠️  [MQTT] 请求 {} 应该由节点端处理，管理端忽略", action);
            return;
        }
        
        // 发送响应
        let response_topic = format!("xrayr/node/{}/response/{}", node_id, request_id);
        let response_json = serde_json::to_string(&response.unwrap())
            .unwrap_or_else(|_| r#"{"msg":"error","error":"序列化响应失败"}"#.to_string());
        
        match client.publish(&response_topic, QoS::AtLeastOnce, false, response_json.as_bytes()).await {
            Ok(_) => {
                info!("✅ [MQTT] 已发送响应: {}", response_topic);
            }
            Err(e) => {
                error!("❌ [MQTT] 发送响应失败: {}, 错误: {}", response_topic, e);
            }
        }
        
        // 根据 action 类型打印不同的信息
        match action {
            "user" | "config" | "inbound" | "outbound" | "routing" => {
                info!("📋 [配置请求] action={}, node_id={}", action, node_id);
                println!("📋 [配置请求] 完整消息内容:");
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
            "submit" => {
                // 流量上报
                if let Some(data_array) = data.get("data").and_then(|v| v.as_array()) {
                    info!("📈 [流量上报] node_id={}, 记录数={}", node_id, data_array.len());
                    if let Err(e) = admin_config.handle_traffic_report(node_id_u64, data_array).await {
                        error!("❌ [流量上报] 处理失败: {}", e);
                    }
                }
            }
            "nodestatus" => {
                // 节点状态上报
                if let Some(status_data) = data.get("data") {
                    if let Err(e) = admin_config.handle_node_status_report(node_id_u64, status_data).await {
                        error!("❌ [节点状态] 处理失败: {}", e);
                    } else {
                        let cpu = status_data.get("cpu").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let mem = status_data.get("mem").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let disk = status_data.get("disk").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let uptime = status_data.get("uptime").and_then(|v| v.as_u64()).unwrap_or(0);
                        
                        // 检查是否有网络接口信息
                        let network_count = status_data.get("network")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.len())
                            .unwrap_or(0);
                        
                        if network_count > 0 {
                            info!("✅ [节点状态] 已更新: node_id={}, CPU={:.1}%, 内存={:.1}%, 磁盘={:.1}%, 运行时间={}秒, 网络接口数={}", 
                                  node_id, cpu * 100.0, mem * 100.0, disk * 100.0, uptime, network_count);
                        } else {
                            info!("✅ [节点状态] 已更新: node_id={}, CPU={:.1}%, 内存={:.1}%, 磁盘={:.1}%, 运行时间={}秒", 
                                  node_id, cpu * 100.0, mem * 100.0, disk * 100.0, uptime);
                        }
                    }
                }
            }
            "onlineusers" => {
                // 在线用户上报
                if let Some(users_array) = data.get("data").and_then(|v| v.as_array()) {
                    info!("👥 [在线用户上报] node_id={}, 在线用户数={}", node_id, users_array.len());
                    if let Err(e) = admin_config.handle_online_users_report(node_id_u64, users_array).await {
                        error!("❌ [在线用户] 处理失败: {}", e);
                    }
                }
            }
            "illegal" => {
                // 非法行为上报
                if let Some(illegal_array) = data.get("data").and_then(|v| v.as_array()) {
                    info!("⚠️  [非法行为上报] node_id={}, 记录数={}", node_id, illegal_array.len());
                    if let Err(e) = admin_config.handle_illegal_report(node_id_u64, illegal_array).await {
                        error!("❌ [非法行为] 处理失败: {}", e);
                    }
                }
            }
            "outbound_failure" => {
                // Outbound 连接失败上报
                if let Some(failure_data) = data.get("data") {
                    if let Err(e) = admin_config.handle_outbound_event(node_id_u64, "failure", failure_data).await {
                        error!("❌ [Outbound失败] 处理失败: {}", e);
                    } else {
                        let outbound_tag = failure_data.get("outbound_tag")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A");
                        error!("🔴 [Outbound 连接失败] node_id={}, outbound_tag={}", node_id, outbound_tag);
                    }
                }
            }
            "outbound_recovery" => {
                // Outbound 连接恢复上报
                if let Some(recovery_data) = data.get("data") {
                    if let Err(e) = admin_config.handle_outbound_event(node_id_u64, "recovery", recovery_data).await {
                        error!("❌ [Outbound恢复] 处理失败: {}", e);
                    } else {
                        let outbound_tag = recovery_data.get("outbound_tag")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A");
                        info!("🟢 [Outbound 连接恢复] node_id={}, outbound_tag={}", node_id, outbound_tag);
                    }
                }
            }
            "outbound_latency" => {
                // 单个 Outbound 延迟上报
                if let Some(latency_data) = data.get("data") {
                    if let Err(e) = admin_config.handle_outbound_latency(node_id_u64, latency_data, "tcp").await {
                        error!("❌ [Outbound延迟] 处理失败: {}", e);
                    }
                }
            }
            "outbound_latencies" => {
                // 批量 Outbound 延迟上报
                if let Some(latencies_data) = data.get("data") {
                    if let Some(latencies_array) = latencies_data.get("latencies").and_then(|v| v.as_array()) {
                        info!("📊 [批量 Outbound 延迟] node_id={}, 数量={}", 
                              node_id, latencies_array.len());
                        for latency_item in latencies_array {
                            if let Err(e) = admin_config.handle_outbound_latency(node_id_u64, latency_item, "tcp").await {
                                error!("❌ [Outbound延迟] 处理失败: {}", e);
                            }
                        }
                    }
                }
            }
            "outbound_udp_latency" => {
                // UDP Outbound 延迟上报
                if let Some(latency_data) = data.get("data") {
                    if let Err(e) = admin_config.handle_outbound_latency(node_id_u64, latency_data, "udp").await {
                        error!("❌ [Outbound延迟] 处理失败: {}", e);
                    }
                }
            }
            "outbound_udp_latencies" => {
                // 批量 UDP Outbound 延迟上报
                if let Some(latencies_data) = data.get("data") {
                    if let Some(latencies_array) = latencies_data.get("latencies").and_then(|v| v.as_array()) {
                        info!("📊 [批量 UDP Outbound 延迟] node_id={}, 数量={}", 
                              node_id, latencies_array.len());
                        for latency_item in latencies_array {
                            if let Err(e) = admin_config.handle_outbound_latency(node_id_u64, latency_item, "udp").await {
                                error!("❌ [Outbound延迟] 处理失败: {}", e);
                            }
                        }
                    }
                }
            }
            "query_logs" => {
                // 日志查询请求
                info!("📝 [日志查询请求] node_id={}", node_id);
                println!("📝 [日志查询请求] 完整消息内容:");
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
            "udp_probe" => {
                // UDP 探测请求
                info!("🔍 [UDP 探测请求] node_id={}", node_id);
                println!("🔍 [UDP 探测请求] 完整消息内容:");
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
            _ => {
                warn!("⚠️  [未知操作] node_id={}, action={}", node_id, action);
                println!("⚠️  [未知操作] 完整消息内容:");
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
        }
    }
    
    /// 处理 MQTT 请求，返回响应数据
    async fn handle_request(action: &str, node_id: u64, admin_config: &AdminConfigStore) -> Option<Value> {
        match action {
            "user" => {
                // 从数据库获取用户列表
                match admin_config.get_users(node_id).await {
                    Ok(users) => {
                        Some(serde_json::json!({
                            "msg": "ok",
                            "data": users
                        }))
                    }
                    Err(e) => {
                        error!("❌ [MQTT Request] 获取用户列表失败: {}", e);
                        Some(serde_json::json!({
                            "msg": "error",
                            "error": e
                        }))
                    }
                }
            }
            "config" => {
                // 从数据库获取节点配置
                match admin_config.get_node_config(node_id).await {
                    Ok(node_config) => {
                        // 检查请求数据中是否包含硬件信息
                        // 注意：这里需要从原始请求数据中获取，但 handle_request 只接收 action 和 node_id
                        // 我们需要修改函数签名来接收完整的请求数据
                        Some(serde_json::json!({
                            "msg": "ok",
                            "data": node_config
                        }))
                    }
                    Err(e) => {
                        error!("❌ [MQTT Request] 获取节点配置失败: {}", e);
                        Some(serde_json::json!({
                            "msg": "error",
                            "error": e
                        }))
                    }
                }
            }
            "inbound" => {
                match admin_config.get_inbounds(node_id).await {
                    Ok(inbounds) => {
                        Some(serde_json::json!({
                            "msg": "ok",
                            "data": {
                                "inbounds": inbounds
                            }
                        }))
                    }
                    Err(e) => {
                        error!("❌ [MQTT Request] 获取入站配置失败: {}", e);
                        Some(serde_json::json!({
                            "msg": "error",
                            "error": e
                        }))
                    }
                }
            }
            "outbound" => {
                // 从数据库获取上游代理配置
                match admin_config.get_outbounds(node_id).await {
                    Ok((outbounds, user_mapping)) => {
                        Some(serde_json::json!({
                            "msg": "ok",
                            "data": {
                                "outbounds": outbounds,
                                "user_mapping": user_mapping
                            }
                        }))
                    }
                    Err(e) => {
                        error!("❌ [MQTT Request] 获取上游代理配置失败: {}", e);
                        Some(serde_json::json!({
                            "msg": "error",
                            "error": e
                        }))
                    }
                }
            }
            "routing" => {
                // 从数据库获取路由配置
                match admin_config.get_routing(node_id).await {
                    Ok(routing) => {
                        Some(serde_json::json!({
                            "msg": "ok",
                            "data": {
                                "domainStrategy": routing.domain_strategy,
                                "rules": routing.rules
                            }
                        }))
                    }
                    Err(e) => {
                        error!("❌ [MQTT Request] 获取路由配置失败: {}", e);
                        Some(serde_json::json!({
                            "msg": "error",
                            "error": e
                        }))
                    }
                }
            }
            "submit" | "nodestatus" | "onlineusers" | "illegal" 
            | "outbound_failure" | "outbound_recovery" 
            | "outbound_latency" | "outbound_latencies"
            | "outbound_udp_latency" | "outbound_udp_latencies" => {
                // 对于上报操作，返回成功响应
                Some(serde_json::json!({
                    "msg": "ok"
                }))
            }
            "query_logs" => {
                // 日志查询请求应该由节点处理
                // 注意：如果这个请求是管理端自己发送的，应该在 handle_message 中已经被过滤掉了
                // 这里不应该收到管理端发送的 query_logs 请求
                None  // 返回 None，让节点端处理
            }
            "udp_probe" => {
                // UDP 探测请求应该由节点处理，管理端不发送响应
                None
            }
            _ => {
                // 未知操作
                Some(serde_json::json!({
                    "msg": "error",
                    "error": format!("未知的 action: {}", action)
                }))
            }
        }
    }
    
    /// 推送配置更新通知到节点
    /// 
    /// # 参数
    /// - `node_id`: 节点 ID
    /// - `update_type`: 更新类型 ('user', 'outbound', 'config', 'routing')
    pub async fn publish_update_notification(
        &self,
        node_id: u64,
        update_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let topic = format!("xrayr/node/{}/update/{}", node_id, update_type);
        let message = serde_json::json!({
            "type": update_type,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            "action": "update"
        });
        
        let message_json = serde_json::to_string(&message)?;
        
        match self.client.publish(&topic, QoS::AtLeastOnce, false, message_json.as_bytes()).await {
            Ok(_) => {
                info!("📢 [MQTT] 已推送更新通知: {} (type={})", topic, update_type);
                Ok(())
            }
            Err(e) => {
                error!("❌ [MQTT] 推送更新通知失败: {}, 错误: {}", topic, e);
                Err(format!("推送更新通知失败: {}", e).into())
            }
        }
    }
    
    /// 通过 MQTT 查询节点日志
    /// 
    /// # 参数
    /// - `node_id`: 节点 ID
    /// - `query_params`: 查询参数，包含 uid, days, event, limit 等
    /// - `timeout`: 超时时间（秒）
    pub async fn query_node_logs(
        &self,
        node_id: u64,
        query_params: Value,
        timeout: u64,
    ) -> Option<Value> {
        info!("📤 [MQTT Log Query] 开始查询日志: node_id={}, query_params={:?}, timeout={}s", 
              node_id, query_params, timeout);
        
        // 生成请求 ID（与 Python 实现保持一致：log_query_{timestamp_ms}_{node_id}）
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let request_id = format!("log_query_{}_{}", timestamp_ms, node_id);
        
        // 准备请求数据
        let request_data = serde_json::json!({
            "node_id": node_id,
            "token": "123",  // 使用配置的 token
            "action": "query_logs",
            "data": query_params
        });
        
        // 创建响应等待器
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        // 注册等待器
        {
            let mut waiters = self.response_waiters.lock().await;
            waiters.insert(request_id.clone(), tx);
            info!("📝 [MQTT Log Query] 已注册响应等待器: request_id={}", request_id);
            // 打印所有等待的 request_id（用于调试）
            let waiting_ids: Vec<String> = waiters.keys().cloned().collect();
            debug!("🔍 [MQTT Log Query] 当前所有等待的 request_id: {:?}", waiting_ids);
        }
        
        // 发布请求
        let request_topic = format!("xrayr/node/{}/request/{}", node_id, request_id);
        let request_json = serde_json::to_string(&request_data)
            .unwrap_or_else(|_| r#"{"msg":"error","error":"序列化请求失败"}"#.to_string());
        
        info!("📤 [MQTT Log Query] 准备发送日志查询请求: topic={}, request_id={}", request_topic, request_id);
        debug!("📤 [MQTT Log Query] 请求内容: {}", request_json);
        
        match self.client.publish(&request_topic, QoS::AtLeastOnce, false, request_json.as_bytes()).await {
            Ok(_) => {
                info!("✅ [MQTT Log Query] 请求发布成功，等待响应 (timeout={}s)...", timeout);
            }
            Err(e) => {
                error!("❌ [MQTT Log Query] 发送日志查询请求失败: {}, 错误: {}", request_topic, e);
                // 清理等待器
                let mut waiters = self.response_waiters.lock().await;
                waiters.remove(&request_id);
                return None;
            }
        }
        
        // 等待响应（带超时）
        let start_time = std::time::Instant::now();
        match time::timeout(Duration::from_secs(timeout), rx).await {
            Ok(Ok(response)) => {
                let elapsed = start_time.elapsed();
                info!("✅ [MQTT Log Query] 收到日志查询响应: request_id={}, 耗时 {:.2}s", 
                      request_id, elapsed.as_secs_f64());
                debug!("✅ [MQTT Log Query] 响应内容: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
                Some(response)
            }
            Ok(Err(_)) => {
                warn!("⚠️  [MQTT Log Query] 日志查询响应通道已关闭: request_id={}", request_id);
                // 清理等待器
                let mut waiters = self.response_waiters.lock().await;
                waiters.remove(&request_id);
                None
            }
            Err(_) => {
                let elapsed = start_time.elapsed();
                warn!("⚠️  [MQTT Log Query] 日志查询超时: request_id={}, timeout={}s, 已等待 {:.2}s", 
                      request_id, timeout, elapsed.as_secs_f64());
                // 打印当前所有等待的 request_id（用于调试）
                let mut waiters = self.response_waiters.lock().await;
                let waiting_ids: Vec<String> = waiters.keys().cloned().collect();
                warn!("🔍 [MQTT Log Query] 超时时，当前所有等待的 request_id: {:?}", waiting_ids);
                // 清理等待器
                if waiters.remove(&request_id).is_some() {
                    warn!("⚠️  [MQTT Log Query] 已清理超时的等待器: request_id={}", request_id);
                } else {
                    warn!("⚠️  [MQTT Log Query] 等待器不存在（可能已被清理）: request_id={}", request_id);
                }
                None
            }
        }
    }

    pub async fn query_node_network_interfaces(
        &self,
        node_id: u64,
        timeout: u64,
    ) -> Option<Value> {
        let request_id = format!(
            "query_network_{}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            node_id
        );

        let request_data = serde_json::json!({
            "node_id": node_id,
            "token": "123",
            "action": "query_network"
        });

        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut waiters = self.response_waiters.lock().await;
            waiters.insert(request_id.clone(), tx);
        }

        let request_topic = format!("xrayr/node/{}/request/{}", node_id, request_id);
        let request_json = serde_json::to_string(&request_data)
            .unwrap_or_else(|_| r#"{\"msg\":\"error\",\"error\":\"序列化请求失败\"}"#.to_string());

        if let Err(e) = self
            .client
            .publish(&request_topic, QoS::AtLeastOnce, false, request_json.as_bytes())
            .await
        {
            error!(
                "❌ [MQTT Network Query] 发送网络接口查询请求失败: {}, 错误: {}",
                request_topic, e
            );
            let mut waiters = self.response_waiters.lock().await;
            waiters.remove(&request_id);
            return None;
        }

        match time::timeout(Duration::from_secs(timeout), rx).await {
            Ok(Ok(response)) => Some(response),
            Ok(Err(_)) => {
                let mut waiters = self.response_waiters.lock().await;
                waiters.remove(&request_id);
                None
            }
            Err(_) => {
                let mut waiters = self.response_waiters.lock().await;
                waiters.remove(&request_id);
                None
            }
        }
    }
    
    /// 主动查询指定 outbound 的 UDP 延迟
    /// 
    /// # 参数
    /// - `node_id`: 节点 ID
    /// - `outbound_tag`: Outbound 标签
    /// - `timeout`: 超时时间（秒）
    pub async fn query_udp_latency(
        &self,
        node_id: u64,
        outbound_tag: &str,
        timeout: u64,
    ) -> Option<Value> {
        info!("🔍 [UDP Probe API] 开始查询 UDP 延迟: node_id={}, outbound_tag={}, timeout={}s", 
              node_id, outbound_tag, timeout);
        
        // 生成请求 ID
        let request_id = format!(
            "udp_probe_{}_{}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            node_id,
            outbound_tag
        );
        
        // 准备请求数据
        let request_data = serde_json::json!({
            "node_id": node_id,
            "token": "123",
            "action": "udp_probe",
            "data": {
                "outbound_tag": outbound_tag
            }
        });
        
        debug!("📤 [UDP Probe API] 准备发送 MQTT 请求: request_id={}, data={}", 
               request_id, serde_json::to_string(&request_data).unwrap_or_default());
        
        // 创建响应等待器
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        // 注册等待器
        {
            let mut waiters = self.response_waiters.lock().await;
            waiters.insert(request_id.clone(), tx);
        }
        
        // 发布请求
        let request_topic = format!("xrayr/node/{}/request/{}", node_id, request_id);
        let request_json = serde_json::to_string(&request_data)
            .unwrap_or_else(|_| r#"{"msg":"error","error":"序列化请求失败"}"#.to_string());
        
        debug!("📡 [UDP Probe API] 发布 MQTT 消息到主题: {}", request_topic);
        
        match self.client.publish(&request_topic, QoS::AtLeastOnce, false, request_json.as_bytes()).await {
            Ok(_) => {
                info!("✅ [UDP Probe API] MQTT 消息发布成功，等待响应 (timeout={}s)...", timeout);
            }
            Err(e) => {
                error!("❌ [UDP Probe API] 发布 UDP 探测请求失败: {}, 错误: {}", request_topic, e);
                // 清理等待器
                let mut waiters = self.response_waiters.lock().await;
                waiters.remove(&request_id);
                return None;
            }
        }
        
        // 等待响应（带超时）
        let start_time = std::time::Instant::now();
        match time::timeout(Duration::from_secs(timeout), rx).await {
            Ok(Ok(response)) => {
                let elapsed = start_time.elapsed();
                info!("✅ [UDP Probe API] 收到响应 (耗时 {:.2}s): {}", 
                      elapsed.as_secs_f64(), 
                      serde_json::to_string(&response).unwrap_or_default());
                Some(response)
            }
            Ok(Err(_)) => {
                warn!("⚠️  [UDP Probe API] UDP 探测响应通道已关闭: request_id={}", request_id);
                None
            }
            Err(_) => {
                let elapsed = start_time.elapsed();
                warn!("⚠️  [UDP Probe API] UDP 探测超时 (等待了 {:.2}s)，未收到响应: request_id={}", 
                      elapsed.as_secs_f64(), request_id);
                // 清理等待器
                let mut waiters = self.response_waiters.lock().await;
                waiters.remove(&request_id);
                None
            }
        }
    }
    
    /// 关闭 MQTT 客户端
    pub async fn shutdown(&self) {
        info!("🛑 正在关闭 MQTT 客户端...");
        self.shutdown_flag.notify_one();
        let _ = self.client.disconnect().await;
        info!("✅ MQTT 客户端已关闭");
    }
}

impl MqttPublisher {
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let broker_host = env::var("MQTT_BROKER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let broker_port = env::var("MQTT_BROKER_PORT")
            .unwrap_or_else(|_| "1883".to_string())
            .parse::<u16>()
            .unwrap_or(1883);

        let client_id = format!(
            "xrayr-client-api-publisher-{}",
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        );

        let mut mqttoptions = MqttOptions::new(client_id, broker_host.clone(), broker_port);
        mqttoptions.set_keep_alive(Duration::from_secs(60));
        mqttoptions.set_clean_session(true);

        let mqtt_username = env::var("MQTT_USERNAME").unwrap_or_else(|_| "manage".to_string());
        let mqtt_password = env::var("MQTT_PASSWORD").unwrap_or_else(|_| "manage_password_123".to_string());
        mqttoptions.set_credentials(&mqtt_username, &mqtt_password);

        let max_packet_size = 100 * 1024 * 1024;
        mqttoptions.set_max_packet_size(max_packet_size, max_packet_size);

        let tls_ca_path = env::var("MQTT_TLS_CA")
            .or_else(|_| {
                if let Ok(content) = fs::read_to_string("config.toml") {
                    if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                        if let Some(capath) = config
                            .get("v4")
                            .and_then(|v4| v4.get("v4-1"))
                            .and_then(|v4_1| v4_1.get("tls"))
                            .and_then(|tls| tls.get("capath"))
                            .and_then(|v| v.as_str())
                        {
                            return Ok(capath.to_string());
                        }
                    }
                }
                Err(env::VarError::NotPresent)
            })
            .ok();

        if let Some(ca_path) = &tls_ca_path {
            let tls_cert_path = env::var("MQTT_TLS_CERT")
                .or_else(|_| {
                    if let Ok(content) = fs::read_to_string("config.toml") {
                        if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                            if let Some(certpath) = config
                                .get("v4")
                                .and_then(|v4| v4.get("v4-1"))
                                .and_then(|v4_1| v4_1.get("tls"))
                                .and_then(|tls| tls.get("certpath"))
                                .and_then(|v| v.as_str())
                            {
                                return Ok(certpath.to_string());
                            }
                        }
                    }
                    Err(env::VarError::NotPresent)
                })
                .ok();

            let tls_key_path = env::var("MQTT_TLS_KEY")
                .or_else(|_| {
                    if let Ok(content) = fs::read_to_string("config.toml") {
                        if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                            if let Some(keypath) = config
                                .get("v4")
                                .and_then(|v4| v4.get("v4-1"))
                                .and_then(|v4_1| v4_1.get("tls"))
                                .and_then(|tls| tls.get("keypath"))
                                .and_then(|v| v.as_str())
                            {
                                return Ok(keypath.to_string());
                            }
                        }
                    }
                    Err(env::VarError::NotPresent)
                })
                .ok();

            let ca_cert_bytes = fs::read(ca_path)?;
            let client_auth = if let (Some(cert_path), Some(key_path)) = (&tls_cert_path, &tls_key_path) {
                let cert_bytes = fs::read(cert_path)?;
                let key_bytes = fs::read(key_path)?;
                Some((cert_bytes, key_bytes))
            } else {
                None
            };

            let tls_config = rumqttc::TlsConfiguration::Simple {
                ca: ca_cert_bytes,
                alpn: None,
                client_auth,
            };
            mqttoptions.set_transport(Transport::tls_with_config(tls_config));
        }

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 32);
        let client = Arc::new(client);
        let shutdown_flag = Arc::new(tokio::sync::Notify::new());

        let shutdown_clone = shutdown_flag.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_clone.notified() => {
                        break;
                    }
                    ev = eventloop.poll() => {
                        match ev {
                            Ok(Event::Incoming(rumqttc::Packet::ConnAck(_))) => {
                                info!("✅ [MQTT Publisher] 连接已确认");
                            }
                            Ok(_) => {}
                            Err(e) => {
                                warn!("⚠️  [MQTT Publisher] eventloop 错误: {}", e);
                                time::sleep(Duration::from_millis(200)).await;
                            }
                        }
                    }
                }
            }
        });

        time::sleep(Duration::from_millis(200)).await;
        info!("📡 [MQTT Publisher] 已启动: {}:{}", broker_host, broker_port);

        Ok(Self {
            client,
            shutdown_flag,
        })
    }

    pub async fn publish_update_notification(
        &self,
        node_id: u64,
        update_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let topic = format!("xrayr/node/{}/update/{}", node_id, update_type);
        let message = serde_json::json!({
            "type": update_type,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            "action": "update"
        });

        let message_json = serde_json::to_string(&message)?;
        match self
            .client
            .publish(&topic, QoS::AtLeastOnce, false, message_json.as_bytes())
            .await
        {
            Ok(_) => {
                info!("📢 [MQTT Publisher] 已推送更新通知: {} (type={})", topic, update_type);
                Ok(())
            }
            Err(e) => {
                error!("❌ [MQTT Publisher] 推送更新通知失败: {}, 错误: {}", topic, e);
                Err(format!("推送更新通知失败: {}", e).into())
            }
        }
    }

    pub async fn shutdown(&self) {
        self.shutdown_flag.notify_one();
        let _ = self.client.disconnect().await;
    }
}
