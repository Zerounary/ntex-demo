//! MQTT Broker 管理模块
//! 
//! 负责启动和关闭 MQTT broker

use rumqttd::{Broker, Config};
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use log::{info, error, warn};

/// MQTT Broker 管理器
pub struct MqttBrokerManager {
    shutdown_flag: Arc<AtomicBool>,
    broker_thread: Option<thread::JoinHandle<()>>,
}

impl MqttBrokerManager {
    /// 创建并启动 MQTT Broker
    pub fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let shutdown_flag_clone = shutdown_flag.clone();
        
        // 从环境变量获取端口，默认 1883
        let port = env::var("MQTT_PORT")
            .unwrap_or_else(|_| "1883".to_string())
            .parse::<u16>()
            .unwrap_or(1883);
        
        // 从环境变量获取 TLS 证书路径（可选）
        let mut tls_cert_path = env::var("MQTT_TLS_CERT").ok();
        let mut tls_key_path = env::var("MQTT_TLS_KEY").ok();
        let mut tls_ca_path = env::var("MQTT_TLS_CA").ok();
        
        // 如果环境变量中没有，尝试从 config.toml 读取
        if tls_cert_path.is_none() || tls_key_path.is_none() || tls_ca_path.is_none() {
            use std::fs;
            if let Ok(content) = fs::read_to_string("config.toml") {
                if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                    if let Some(tls_config) = config
                        .get("v4")
                        .and_then(|v4| v4.get("1"))
                        .and_then(|v4_1| v4_1.get("tls"))
                    {
                        if tls_cert_path.is_none() {
                            tls_cert_path = tls_config.get("certpath")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                        }
                        if tls_key_path.is_none() {
                            tls_key_path = tls_config.get("keypath")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                        }
                        if tls_ca_path.is_none() {
                            tls_ca_path = tls_config.get("capath")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                        }
                    }
                }
            }
        }
        
        // 加载配置
        let config = load_config_from_toml(port, &tls_cert_path, &tls_key_path, &tls_ca_path)?;
        
        let separator = "=".repeat(70);
        println!("{}", separator);
        println!("🚀 启动 MQTT Broker (rumqttd)");
        println!("{}", separator);
        println!("   监听地址: 0.0.0.0:{}", port);
        println!("");
        println!("📡 支持的协议:");
        if tls_cert_path.is_some() && tls_key_path.is_some() {
            println!("   - MQTT 3.1.1 over TLS (端口 {}) 🔒", port);
        } else {
            println!("   - MQTT 3.1.1 (端口 {})", port);
        }
        println!("");
        println!("✨ 特性:");
        println!("   - 消息持久化");
        println!("   - QoS 0/1/2 支持");
        println!("   - Retained 消息");
        println!("   - Last Will 遗嘱消息");
        if tls_cert_path.is_some() && tls_key_path.is_some() {
            println!("   - TLS/SSL 加密 🔒");
            if let Some(ref cert) = tls_cert_path {
                println!("   - 证书: {}", cert);
            }
            if let Some(ref key) = tls_key_path {
                println!("   - 密钥: {}", key);
            }
            if let Some(ref ca) = tls_ca_path {
                println!("   - CA 证书: {}", ca);
            }
        }
        println!("{}", separator);
        
        // 在单独的线程中启动 broker
        let broker_thread = thread::Builder::new()
            .name("mqtt-broker".to_string())
            .spawn(move || {
                let mut broker = Broker::new(config);
                
                info!("✅ 正在启动 MQTT Broker...");
                match broker.start() {
                    Ok(_) => {
                        info!("✅ MQTT Broker 已启动，等待连接...");
                        // 等待关闭信号
                        while !shutdown_flag_clone.load(Ordering::Relaxed) {
                            thread::sleep(std::time::Duration::from_millis(100));
                        }
                        info!("🛑 正在关闭 MQTT Broker...");
                    }
                    Err(e) => {
                        error!("❌ Broker 启动失败: {}", e);
                        error!("\n提示：");
                        error!("   1. 检查配置文件是否存在");
                        error!("   2. 确保至少配置了 v4、v5 或 ws 中的一个");
                        error!("   3. 检查端口 {} 是否被占用", port);
                    }
                }
            })?;
        
        // 注意：rumqttd 的 Broker::start() 是阻塞的，无法优雅关闭
        // 将线程标记为非守护线程，但主程序退出时会强制结束
        
        // 等待一小段时间确保 broker 启动
        thread::sleep(std::time::Duration::from_millis(500));
        
        Ok(Self {
            shutdown_flag,
            broker_thread: Some(broker_thread),
        })
    }
    
    /// 关闭 MQTT Broker
    pub fn shutdown(&mut self) {
        info!("🛑 正在关闭 MQTT Broker...");
        self.shutdown_flag.store(true, Ordering::Relaxed);
        
        // 注意：rumqttd 的 Broker::start() 是阻塞的，没有提供停止方法
        // 我们只能设置关闭标志，但无法强制停止 broker
        // 当主程序退出时，broker 线程会随着进程一起结束
        
        if let Some(thread) = self.broker_thread.take() {
            // 等待一小段时间，看线程是否能响应关闭信号
            let timeout = std::time::Duration::from_secs(1);
            let start = std::time::Instant::now();
            
            while !thread.is_finished() && start.elapsed() < timeout {
                thread::sleep(std::time::Duration::from_millis(50));
            }
            
            if !thread.is_finished() {
                warn!("⚠️  MQTT Broker 线程未能在超时时间内结束");
                warn!("⚠️  注意：rumqttd 的 Broker::start() 是阻塞的，无法优雅关闭");
                warn!("⚠️  程序退出时 broker 线程会随进程一起结束");
            } else {
                info!("✅ MQTT Broker 已关闭");
            }
        }
    }
}

impl Drop for MqttBrokerManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// 从 TOML 文件加载配置，如果文件不存在则创建默认配置
fn load_config_from_toml(
    port: u16,
    tls_cert_path: &Option<String>,
    tls_key_path: &Option<String>,
    tls_ca_path: &Option<String>,
) -> Result<Config, Box<dyn std::error::Error>> {
    use std::fs;
    
    let max_connections: usize = env::var("MQTT_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10000);
    
    // 配置文件路径：优先使用项目根目录的 config.toml，如果不存在则使用 mqtt-borcker/config.toml
    let config_paths = vec![
        "config.toml",
        "mqtt-borcker/config.toml",
    ];
    
    // 尝试从现有配置文件加载
    for config_path in &config_paths {
        if let Ok(mut content) = fs::read_to_string(config_path) {
            // 检查配置文件中是否已有 TLS 配置
            let has_tls_config = content.contains("[v4.1.tls]");
            
            // 如果配置文件中没有 TLS 配置，但用户提供了 TLS 环境变量，则添加 TLS 配置
            if !has_tls_config && tls_cert_path.is_some() && tls_key_path.is_some() {
                // 在 [v4.v4-1.connections] 之前插入 TLS 配置
                if let Some(connections_pos) = content.find("[v4.1.connections]") {
                    let mut tls_config = String::new();
                    tls_config.push_str("\n[v4.1.tls]\n");
                    if let Some(cert_path) = tls_cert_path {
                        tls_config.push_str(&format!("certpath = \"{}\"\n", cert_path));
                    }
                    if let Some(key_path) = tls_key_path {
                        tls_config.push_str(&format!("keypath = \"{}\"\n", key_path));
                    }
                    if let Some(ca_path) = tls_ca_path {
                        tls_config.push_str(&format!("capath = \"{}\"\n", ca_path));
                    }
                    content.insert_str(connections_pos, &tls_config);
                    
                    // 尝试更新配置文件
                    if let Err(e) = fs::write(config_path, &content) {
                        warn!("⚠️  无法更新配置文件以添加 TLS 配置: {}", e);
                    } else {
                        info!("✅ 已在配置文件中添加 TLS 配置");
                    }
                }
            }
            
            match toml::from_str::<Config>(&content) {
                Ok(config) => {
                    info!("✅ 从 {} 加载 MQTT Broker 配置", config_path);
                    return Ok(config);
                }
                Err(e) => {
                    warn!("⚠️  配置文件 {} 解析失败: {}，将使用默认配置", config_path, e);
                }
            }
        }
    }
    
    // 如果配置文件不存在或解析失败，创建默认配置
    warn!("⚠️  未找到有效的 MQTT Broker 配置文件，使用默认配置");
    
    // 构建 TLS 配置部分（如果提供了证书路径）
    let mut tls_section = String::new();
    if let (Some(cert_path), Some(key_path)) = (tls_cert_path, tls_key_path) {
        tls_section.push_str("\n[v4.1.tls]\n");
        tls_section.push_str(&format!("certpath = \"{}\"\n", cert_path));
        tls_section.push_str(&format!("keypath = \"{}\"\n", key_path));
        if let Some(ca_path) = tls_ca_path {
            tls_section.push_str(&format!("capath = \"{}\"\n", ca_path));
        }
    }
    
    let config_toml = format!(
        r#"id = 0

[router]
max_connections = {}
max_outgoing_packet_count = 100
max_segment_size = 104857600
max_segment_count = 10

[v4.1]
name = "v4-1"
listen = "0.0.0.0:{}"
next_connection_delay_ms = 1
{}[v4.1.connections]
connection_timeout_ms = 60000
max_payload_size = 104857600
max_inflight_count = 100
dynamic_filters = true
"#,
        max_connections, port, tls_section
    );
    
    // 尝试写入默认配置文件
    if let Err(e) = fs::write("config.toml", &config_toml) {
        warn!("⚠️  无法创建默认配置文件: {}", e);
    } else {
        info!("✅ 已创建默认配置文件: config.toml");
    }
    
    // 加载刚创建的配置
    let config: Config = toml::from_str(&config_toml)?;
    Ok(config)
}

