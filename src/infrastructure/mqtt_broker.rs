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
        
        // 加载配置
        let config = load_config_from_toml(port)?;
        
        let separator = "=".repeat(70);
        println!("{}", separator);
        println!("🚀 启动 MQTT Broker (rumqttd)");
        println!("{}", separator);
        println!("   监听地址: 0.0.0.0:{}", port);
        println!("");
        println!("📡 支持的协议:");
        println!("   - MQTT 3.1.1 (端口 {})", port);
        println!("");
        println!("✨ 特性:");
        println!("   - 消息持久化");
        println!("   - QoS 0/1/2 支持");
        println!("   - Retained 消息");
        println!("   - Last Will 遗嘱消息");
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
fn load_config_from_toml(port: u16) -> Result<Config, Box<dyn std::error::Error>> {
    use std::fs;
    
    // 配置文件路径：优先使用项目根目录的 config.toml，如果不存在则使用 mqtt-borcker/config.toml
    let config_paths = vec![
        "config.toml",
        "mqtt-borcker/config.toml",
    ];
    
    // 尝试从现有配置文件加载
    for config_path in &config_paths {
        if let Ok(content) = fs::read_to_string(config_path) {
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
    let config_toml = format!(
        r#"id = 0

[router]
max_connections = 1000
max_outgoing_packet_count = 100
max_segment_size = 104857600
max_segment_count = 10

[v4.v4-1]
name = "v4-1"
listen = "0.0.0.0:{}"
next_connection_delay_ms = 1

[v4.v4-1.connections]
connection_timeout_ms = 60000
max_payload_size = 104857600
max_inflight_count = 100
dynamic_filters = true
"#,
        port
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

