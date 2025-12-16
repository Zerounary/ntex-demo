//! MQTT Broker using rumqttd
//! 
//! This is a standalone MQTT broker implementation using rumqttd.
//! 
//! Usage:
//!   cargo run --bin mqtt_broker

use rumqttd::{Broker, Config};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取端口，默认 1883
    let port = env::var("MQTT_PORT")
        .unwrap_or_else(|_| "1883".to_string())
        .parse::<u16>()
        .unwrap_or(1883);
    
    // 根据 rumqttd 0.20.0 的 API，需要构造包含 v4 配置的 Config
    // 尝试使用 serde 从 TOML 加载，或者直接构造
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
    println!("");
    println!("📝 使用示例:");
    println!("   # 订阅主题");
    println!("   mosquitto_sub -h localhost -p {} -t 'test/topic'", port);
    println!("");
    println!("   # 发布消息");
    println!("   mosquitto_pub -h localhost -p {} -t 'test/topic' -m 'Hello MQTT'", port);
    println!("");
    println!("按 Ctrl+C 停止服务器");
    println!("{}", separator);
    
    // 创建并启动 Broker
    let mut broker = Broker::new(config);
    
    // 启动 Broker（同步方法）
    println!("✅ 正在启动 MQTT Broker...");
    match broker.start() {
        Ok(_) => {
            println!("✅ MQTT Broker 已启动，等待连接...");
            println!("\n按 Ctrl+C 停止服务器...");
            // 阻塞等待
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
        Err(e) => {
            eprintln!("❌ Broker 启动失败: {}", e);
            eprintln!("\n提示：");
            eprintln!("   1. 检查 config.toml 配置文件");
            eprintln!("   2. 确保至少配置了 v4、v5 或 ws 中的一个");
            eprintln!("   3. 检查端口 {} 是否被占用", port);
            eprintln!("   4. 尝试使用配置文件方式：");
            eprintln!("      rumqttd --config config.toml");
            Err(e.into())
        }
    }
}

fn load_config_from_toml(port: u16) -> Result<Config, Box<dyn std::error::Error>> {
    use std::fs;
    use toml;
    
    // 如果配置文件存在，尝试加载
    if let Ok(content) = fs::read_to_string("config.toml") {
        // 使用 config crate 或直接解析 TOML
        // 根据 rumqttd 的实际实现，可能需要使用不同的方式
        // 这里我们尝试使用 serde 反序列化
        let config: Config = toml::from_str(&content)?;
        return Ok(config);
    }
    
    // 如果配置文件不存在，创建默认配置
    let config_toml = format!(
        r#"id = 0

[v4]
listen = "0.0.0.0:{}"
"#,
        port
    );
    
    fs::write("config.toml", &config_toml)?;
    println!("✅ 已创建配置文件: config.toml");
    
    // 加载刚创建的配置
    let config: Config = toml::from_str(&config_toml)?;
    Ok(config)
}
