//! 管理服务器模块

pub mod handlers;
pub mod routes;

use ntex::web::{self, App};
use std::sync::Arc;

use crate::infrastructure::admin_config::AdminConfigStore;
use crate::infrastructure::mqtt_client::MqttClientManager;

use self::routes::configure;
use handlers::AdminState;

pub async fn serve(port: u16, config: AdminConfigStore, mqtt_client: Option<Arc<MqttClientManager>>) -> std::io::Result<()> {
    let state = AdminState {
        config,
        mqtt_client,
    };
    
    web::HttpServer::new(move || {
        App::new()
            .state(state.clone())
            .configure(configure)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

