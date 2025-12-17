mod application;
mod config;
mod domain;
mod infrastructure;
mod interface;

use dotenv::dotenv;
use env_logger::Env;
use infrastructure::{admin_config, database, mqtt_broker, mqtt_client, seed};
use interface::{admin, web};
use interface::web::AppState;
use log::{error, info};

use crate::config::AppConfig;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
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
        }
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

    // 启动 MQTT Broker
    info!("正在启动 MQTT Broker...");
    let mut mqtt_broker = mqtt_broker::MqttBrokerManager::start()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("MQTT Broker 启动失败: {}", e)))?;
    info!("MQTT Broker 启动成功");
    
    // 等待一小段时间确保 broker 完全启动
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // 启动 MQTT 客户端用于接收节点上报数据
    info!("正在启动 MQTT 客户端...");
    let mqtt_client = mqtt_client::MqttClientManager::start()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("MQTT 客户端启动失败: {}", e)))?;
    info!("MQTT 客户端启动成功");

    let state = AppState::new(db);
    
    // 创建管理配置存储
    let admin_config = admin_config::AdminConfigStore::new();
    
    // 创建 MQTT 客户端 Arc 引用
    let mqtt_client_arc = std::sync::Arc::new(mqtt_client);
    
    // 启动管理服务器（端口 667）和 web 服务器并行运行
    let admin_config_clone = admin_config.clone();
    let mqtt_client_clone = mqtt_client_arc.clone();
    
    info!("正在启动管理服务器 (端口 667)...");
    info!("正在启动 Web 服务器 (端口 {})...", config.port);
    
    // 使用 tokio::select! 并行运行两个服务器，并监听关闭信号
    tokio::select! {
        result = admin::serve(667, admin_config_clone, Some(mqtt_client_clone)) => {
            if let Err(e) = result {
                eprintln!("管理服务器错误: {:?}", e);
            } else {
                info!("管理服务器已关闭");
            }
        }
        result = web::serve(config.port, state) => {
            if let Err(e) = result {
                eprintln!("Web 服务器错误: {:?}", e);
            } else {
                info!("Web 服务器已关闭");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("收到 Ctrl+C 信号，正在关闭服务器...");
        }
    }
    
    // 当服务器关闭时，关闭 MQTT 客户端和 Broker
    info!("正在关闭 MQTT 客户端...");
    mqtt_client_arc.shutdown().await;
    
    info!("正在关闭 MQTT Broker...");
    mqtt_broker.shutdown();
    
    Ok(())
}

fn to_io_error(err: impl std::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
}
