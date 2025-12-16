mod application;
mod config;
mod domain;
mod infrastructure;
mod interface;

use dotenv::dotenv;
use env_logger::Env;
use infrastructure::{database, mqtt_broker, mqtt_client, seed};
use interface::web::{self, AppState};
use log::info;

use crate::config::AppConfig;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let config = AppConfig::from_env();

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

    // 初始化数据库
    let db = database::connect(&config.database_url)
        .await
        .map_err(to_io_error)?;
    database::init(&db).await.map_err(to_io_error)?;
    seed::seed(&db).await.map_err(to_io_error)?;

    let state = AppState::new(db);
    
    // 启动 web 服务器
    info!("正在启动 Web 服务器...");
    let server_result = web::serve(config.port, state).await;
    
    // 当 web 服务器关闭时，关闭 MQTT 客户端和 Broker
    info!("正在关闭 MQTT 客户端...");
    mqtt_client.shutdown().await;
    
    info!("正在关闭 MQTT Broker...");
    mqtt_broker.shutdown();
    
    server_result
}

fn to_io_error(err: impl std::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
}
