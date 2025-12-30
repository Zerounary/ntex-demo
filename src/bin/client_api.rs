#[path = "../application/mod.rs"]
mod application;
#[path = "../config.rs"]
mod config;
#[path = "../domain/mod.rs"]
mod domain;
#[path = "../infrastructure/mod.rs"]
mod infrastructure;
#[path = "../interface/mod.rs"]
mod interface;

use dotenv::dotenv;
use env_logger::Env;
use infrastructure::{admin_config, database, seed};
use interface::web;
use interface::web::AppState;
use log::{error, info, warn};
use std::sync::Arc;

use crate::config::AppConfig;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let default_filter = "info,rumqttd::router::routing=off,rumqttd=off";
    env_logger::Builder::from_env(Env::default().default_filter_or(default_filter)).init();

    let config = AppConfig::from_env();

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

    info!("正在初始化数据库表结构...");
    if let Err(e) = database::init(&db).await {
        error!("数据库初始化失败: {}", e);
        error!("请检查数据库权限和配置，程序将退出");
        std::process::exit(1);
    }
    info!("数据库表结构初始化成功");

    info!("正在执行数据库种子数据...");
    if let Err(e) = seed::seed(&db).await {
        error!("数据库种子数据执行失败: {}", e);
        error!("程序将退出");
        std::process::exit(1);
    }
    info!("数据库种子数据执行成功");

    let admin_config = admin_config::AdminConfigStore::new(db.clone());

    let mqtt_publisher = match infrastructure::mqtt_client::MqttPublisher::start().await {
        Ok(p) => Some(Arc::new(p)),
        Err(e) => {
            warn!("MQTT publisher 启动失败（将继续运行，但节点配置刷新可能有延迟）: {}", e);
            None
        }
    };

    let state = AppState::new(db.clone(), admin_config, mqtt_publisher);

    info!("正在启动客户端 Web 服务器 (端口 {})...", config.port);

    tokio::select! {
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

    Ok(())
}
