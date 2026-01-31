pub mod dto;
pub mod errors;
pub mod auth;
pub mod handlers;
pub mod routes;

use ntex::web::{self, App};
use ntex_files as fs;
use sea_orm::DatabaseConnection;
 use std::sync::Arc;
use tonic::transport::Channel;

use crate::infrastructure::admin_config::AdminConfigStore;
 use crate::infrastructure::mqtt_client::MqttPublisher;

use self::routes::configure;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub admin_config: AdminConfigStore,
    pub main_grpc: Option<Channel>,
    pub mqtt_publisher: Option<Arc<MqttPublisher>>,
}

impl AppState {
    pub fn new(
        db: DatabaseConnection,
        admin_config: AdminConfigStore,
        main_grpc: Option<Channel>,
        mqtt_publisher: Option<Arc<MqttPublisher>>,
    ) -> Self {
        Self {
            db,
            admin_config,
            main_grpc,
            mqtt_publisher,
        }
    }
}

pub async fn serve(port: u16, state: AppState) -> std::io::Result<()> {
    web::HttpServer::new(move || {
        App::new()
            .state(state.clone())
            .configure(configure)
            .service(
                fs::Files::new("/", "./static")
                    .show_files_listing()
                    .index_file("index.html"),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
