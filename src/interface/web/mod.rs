pub mod dto;
pub mod errors;
pub mod handlers;
pub mod routes;

use ntex::web::{self, App};
use ntex_files as fs;
use sea_orm::DatabaseConnection;
 use std::sync::Arc;

use crate::infrastructure::admin_config::AdminConfigStore;
 use crate::infrastructure::mqtt_client::MqttPublisher;

use self::routes::configure;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub admin_config: AdminConfigStore,
    pub mqtt_publisher: Option<Arc<MqttPublisher>>,
}

impl AppState {
    pub fn new(
        db: DatabaseConnection,
        admin_config: AdminConfigStore,
        mqtt_publisher: Option<Arc<MqttPublisher>>,
    ) -> Self {
        Self {
            db,
            admin_config,
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
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
