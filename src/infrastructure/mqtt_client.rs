//! MQTT 客户端模块
//! 
//! 用于连接 MQTT broker 并接收节点上报的数据

use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, QoS};
use serde_json::Value;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use log::{info, error, warn, debug};

/// MQTT 客户端管理器
pub struct MqttClientManager {
    client: Arc<AsyncClient>,
    shutdown_flag: Arc<tokio::sync::Notify>,
}

impl MqttClientManager {
    /// 创建并启动 MQTT 客户端
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
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
        
        // 创建客户端和事件循环
        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        let client_arc = Arc::new(client);
        
        // 订阅主题
        let request_topic = "xrayr/node/+/request/+";
        let response_topic = "xrayr/node/+/response/+";
        
        info!("📡 正在连接 MQTT Broker: {}:{}", broker_host, broker_port);
        
        // 等待连接建立
        let mut connected = false;
        let mut retries = 0;
        while !connected && retries < 10 {
            match eventloop.poll().await {
                Ok(Event::Incoming(rumqttc::Packet::ConnAck(_))) => {
                    info!("✅ MQTT 客户端已连接到 {}:{}", broker_host, broker_port);
                    connected = true;
                }
                Ok(Event::Incoming(packet)) => {
                    debug!("收到 MQTT 数据包: {:?}", packet);
                }
                Ok(Event::Outgoing(_)) => {
                    // 连接中
                }
                Err(e) => {
                    warn!("⚠️  MQTT 连接错误: {}，重试中...", e);
                    retries += 1;
                    time::sleep(Duration::from_millis(500)).await;
                }
            }
        }
        
        if !connected {
            return Err("MQTT 客户端连接失败".into());
        }
        
        // 订阅主题
        client_arc.subscribe(request_topic, QoS::AtLeastOnce).await?;
        info!("📡 已订阅 MQTT 主题: {}", request_topic);
        
        client_arc.subscribe(response_topic, QoS::AtLeastOnce).await?;
        info!("📡 已订阅 MQTT 主题: {}", response_topic);
        
        // 创建关闭信号
        let shutdown_flag = Arc::new(tokio::sync::Notify::new());
        
        // 启动事件循环任务
        let client_clone = client_arc.clone();
        let shutdown_clone = shutdown_flag.clone();
        tokio::spawn(async move {
            Self::run_event_loop(eventloop, client_clone, shutdown_clone).await;
        });
        
        Ok(Self {
            client: client_arc,
            shutdown_flag,
        })
    }
    
    /// 运行事件循环，处理消息
    async fn run_event_loop(
        mut eventloop: EventLoop,
        _client: Arc<AsyncClient>,
        shutdown_flag: Arc<tokio::sync::Notify>,
    ) {
        info!("🚀 MQTT 客户端事件循环已启动");
        
        loop {
            tokio::select! {
                // 处理 MQTT 事件
                event = eventloop.poll() => {
                    match event {
                        Ok(Event::Incoming(rumqttc::Packet::Publish(publish))) => {
                            let client_clone = _client.clone();
                            Self::handle_message(client_clone, publish.topic, publish.payload).await;
                        }
                        Ok(Event::Incoming(rumqttc::Packet::ConnAck(_))) => {
                            info!("✅ MQTT 连接已确认");
                        }
                        Ok(Event::Incoming(rumqttc::Packet::Disconnect)) => {
                            warn!("⚠️  MQTT 连接已断开");
                        }
                        Ok(Event::Incoming(packet)) => {
                            debug!("收到 MQTT 数据包: {:?}", packet);
                        }
                        Ok(Event::Outgoing(_)) => {
                            // 忽略出站消息
                        }
                        Err(e) => {
                            error!("❌ MQTT 事件循环错误: {}", e);
                            // 等待一段时间后继续
                            time::sleep(Duration::from_secs(1)).await;
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
    
    /// 处理收到的消息
    async fn handle_message(client: Arc<AsyncClient>, topic: String, payload: bytes::Bytes) {
        // 解析主题: xrayr/node/{node_id}/request/{request_id} 或 xrayr/node/{node_id}/response/{request_id}
        let topic_parts: Vec<&str> = topic.split('/').collect();
        if topic_parts.len() != 5 {
            warn!("⚠️  无效的 MQTT 主题格式: {}", topic);
            return;
        }
        
        let node_id = topic_parts[2];
        let topic_type = topic_parts[3]; // 'request' 或 'response'
        let request_id = topic_parts[4];
        
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
            info!("📥 [MQTT Response] node_id={}, request_id={}", node_id, request_id);
            println!("📥 [MQTT Response] 完整消息内容:");
            println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            return;
        }
        
        // 处理请求消息
        let action = data.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        info!("📨 [MQTT Request] node_id={}, action={}, request_id={}", node_id, action, request_id);
        
        // 生成响应
        let response = Self::handle_request(action, &data);
        
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
            "user" | "config" | "outbound" | "routing" => {
                info!("📋 [配置请求] action={}, node_id={}", action, node_id);
                println!("📋 [配置请求] 完整消息内容:");
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
            "submit" => {
                // 流量上报
                if let Some(data_array) = data.get("data").and_then(|v| v.as_array()) {
                    info!("📈 [流量上报] node_id={}, 记录数={}", node_id, data_array.len());
                    println!("📈 [流量上报] 完整消息内容:");
                    println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
                }
            }
            "nodestatus" => {
                // 节点状态上报
                if let Some(status_data) = data.get("data") {
                    let cpu = status_data.get("cpu").and_then(|v| v.as_str()).unwrap_or("N/A");
                    let mem = status_data.get("mem").and_then(|v| v.as_str()).unwrap_or("N/A");
                    let disk = status_data.get("disk").and_then(|v| v.as_str()).unwrap_or("N/A");
                    let uptime = status_data.get("uptime").and_then(|v| v.as_u64()).unwrap_or(0);
                    info!("💻 [节点状态上报] node_id={}, CPU={}, 内存={}, 磁盘={}, 运行时间={}秒", 
                          node_id, cpu, mem, disk, uptime);
                }
                println!("💻 [节点状态上报] 完整消息内容:");
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
            "onlineusers" => {
                // 在线用户上报
                if let Some(users_array) = data.get("data").and_then(|v| v.as_array()) {
                    info!("👥 [在线用户上报] node_id={}, 在线用户数={}", node_id, users_array.len());
                    println!("👥 [在线用户上报] 完整消息内容:");
                    println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
                }
            }
            "illegal" => {
                // 非法行为上报
                if let Some(illegal_array) = data.get("data").and_then(|v| v.as_array()) {
                    info!("⚠️  [非法行为上报] node_id={}, 记录数={}", node_id, illegal_array.len());
                    println!("⚠️  [非法行为上报] 完整消息内容:");
                    println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
                }
            }
            "outbound_failure" => {
                // Outbound 连接失败上报
                if let Some(failure_data) = data.get("data") {
                    let outbound_tag = failure_data.get("outbound_tag")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A");
                    let error_msg = failure_data.get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A");
                    error!("🔴 [Outbound 连接失败] node_id={}, outbound_tag={}, error={}", 
                           node_id, outbound_tag, error_msg);
                }
                println!("🔴 [Outbound 连接失败] 完整消息内容:");
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
            "outbound_recovery" => {
                // Outbound 连接恢复上报
                if let Some(recovery_data) = data.get("data") {
                    let outbound_tag = recovery_data.get("outbound_tag")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A");
                    info!("🟢 [Outbound 连接恢复] node_id={}, outbound_tag={}", 
                          node_id, outbound_tag);
                }
                println!("🟢 [Outbound 连接恢复] 完整消息内容:");
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
            "outbound_latency" => {
                // 单个 Outbound 延迟上报
                if let Some(latency_data) = data.get("data") {
                    let outbound_tag = latency_data.get("outbound_tag")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A");
                    let latency_ms = latency_data.get("latency_ms")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    info!("⏱️  [Outbound 延迟] node_id={}, outbound_tag={}, latency={:.2}ms", 
                          node_id, outbound_tag, latency_ms);
                }
                println!("⏱️  [Outbound 延迟] 完整消息内容:");
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
            "outbound_latencies" => {
                // 批量 Outbound 延迟上报
                if let Some(latencies_data) = data.get("data") {
                    if let Some(latencies_array) = latencies_data.get("latencies").and_then(|v| v.as_array()) {
                        info!("📊 [批量 Outbound 延迟] node_id={}, 数量={}", 
                              node_id, latencies_array.len());
                        println!("📊 [批量 Outbound 延迟] 完整消息内容:");
                        println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
                    }
                }
            }
            "outbound_udp_latency" => {
                // UDP Outbound 延迟上报
                if let Some(latency_data) = data.get("data") {
                    let outbound_tag = latency_data.get("outbound_tag")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A");
                    let latency_ms = latency_data.get("latency_ms")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    info!("📡 [UDP Outbound 延迟] node_id={}, outbound_tag={}, latency={:.2}ms", 
                          node_id, outbound_tag, latency_ms);
                }
                println!("📡 [UDP Outbound 延迟] 完整消息内容:");
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
            "outbound_udp_latencies" => {
                // 批量 UDP Outbound 延迟上报
                if let Some(latencies_data) = data.get("data") {
                    if let Some(latencies_array) = latencies_data.get("latencies").and_then(|v| v.as_array()) {
                        info!("📊 [批量 UDP Outbound 延迟] node_id={}, 数量={}", 
                              node_id, latencies_array.len());
                        println!("📊 [批量 UDP Outbound 延迟] 完整消息内容:");
                        println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
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
    fn handle_request(action: &str, _request_data: &Value) -> Option<Value> {
        match action {
            "user" => {
                // 返回用户列表（使用 Python 脚本中的配置数据）
                Some(serde_json::json!({
                    "msg": "ok",
                    "data": [
                        {
                            "id": 1,
                            "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                            "st": 5,
                            "dt": 0
                        },
                        {
                            "id": 2,
                            "uuid": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
                            "st": 1,
                            "dt": 0
                        },
                        {
                            "id": 3,
                            "uuid": "c3d4e5f6-a7b8-9012-cdef-123456789012",
                            "st": 1,
                            "dt": 0
                        },
                        {
                            "id": 4,
                            "uuid": "d4e5f6a7-b8c9-0123-def0-234567890123",
                            "st": 1,
                            "dt": 0
                        },
                        {
                            "id": 5,
                            "uuid": "e5f6a7b8-c9d0-1234-ef01-345678901234",
                            "st": 1,
                            "dt": 0
                        },
                        {
                            "id": 6,
                            "uuid": "f6a7b8c9-d0e1-2345-f012-456789012345",
                            "st": 1,
                            "dt": 0
                        }
                    ]
                }))
            }
            "config" => {
                // 返回节点配置（使用 Python 脚本中的配置数据）
                Some(serde_json::json!({
                    "msg": "ok",
                    "data": {
                        "node_id": 41,
                        "node_type": "Vmess",
                        "node_speed_limit": 0,
                        "traffic_rate": 1.0,
                        "sort": 1,
                        "inbounds": [
                            {
                                "port": 10086,
                                "protocol": "vmess",
                                "settings": {},
                                "streamSettings": {
                                    "network": "tcp"
                                }
                            }
                        ]
                    }
                }))
            }
            "outbound" => {
                // 返回上游代理配置（使用 Python 脚本中的配置数据）
                Some(serde_json::json!({
                    "msg": "ok",
                    "data": {
                        "outbounds": [
                            {
                                "tag": "block",
                                "protocol": "blackhole",
                                "settings": {
                                    "response": {
                                        "type": "http"
                                    }
                                }
                            },
                            {
                                "tag": "direct",
                                "protocol": "freedom",
                                "settings": {}
                            },
                            {
                                "tag": "ss_1",
                                "protocol": "shadowsocks",
                                "settings": {
                                    "servers": [
                                        {
                                            "address": "67.209.176.181",
                                            "port": 19166,
                                            "method": "aes-256-gcm",
                                            "password": "bxaeWJ4Kf9ZL59R3"
                                        }
                                    ]
                                }
                            },
                            {
                                "tag": "ss_2",
                                "protocol": "shadowsocks",
                                "settings": {
                                    "servers": [
                                        {
                                            "address": "65.49.212.165",
                                            "port": 19166,
                                            "method": "aes-256-gcm",
                                            "password": "bxaeWJ4Kf9ZL59R3"
                                        }
                                    ]
                                }
                            },
                            {
                                "tag": "ss_3",
                                "protocol": "shadowsocks",
                                "settings": {
                                    "servers": [
                                        {
                                            "address": "65.49.212.165",
                                            "port": 19166,
                                            "method": "aes-256-gcm",
                                            "password": "bxaeWJ4Kf9ZL59R3"
                                        }
                                    ]
                                }
                            },
                            {
                                "tag": "vmess_loopback",
                                "protocol": "vmess",
                                "settings": {
                                    "vnext": [
                                        {
                                            "address": "127.0.0.1",
                                            "port": 10086,
                                            "users": [
                                                {
                                                    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                                                    "alterId": 0,
                                                    "email": "t@t.tt",
                                                    "security": "auto"
                                                }
                                            ]
                                        }
                                    ]
                                },
                                "streamSettings": {
                                    "network": "tcp"
                                }
                            }
                        ],
                        "user_mapping": {
                            "a1b2c3d4-e5f6-7890-abcd-ef1234567890": "ss_1",
                            "b2c3d4e5-f6a7-8901-bcde-f12345678901": "ss_2",
                            "c3d4e5f6-a7b8-9012-cdef-123456789012": "ss_3",
                            "d4e5f6a7-b8c9-0123-def0-234567890123": "ss_1",
                            "e5f6a7b8-c9d0-1234-ef01-345678901234": "ss_2"
                        }
                    }
                }))
            }
            "routing" => {
                // 返回路由配置（使用 Python 脚本中的配置数据）
                Some(serde_json::json!({
                    "msg": "ok",
                    "data": {
                        "domainStrategy": "AsIs",
                        "rules": []
                    }
                }))
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
                Some(serde_json::json!({
                    "msg": "error",
                    "error": "query_logs 应该由节点处理"
                }))
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
    
    /// 关闭 MQTT 客户端
    pub async fn shutdown(&self) {
        info!("🛑 正在关闭 MQTT 客户端...");
        self.shutdown_flag.notify_one();
        let _ = self.client.disconnect().await;
        info!("✅ MQTT 客户端已关闭");
    }
}
