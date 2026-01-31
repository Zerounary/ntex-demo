//! 管理服务器模块

pub mod handlers;
pub mod chain_ops;
pub mod routes;

use ntex::web::{self, App};
use std::sync::Arc;

use ntex_files as fs;
use crate::infrastructure::admin_config::AdminConfigStore;
use crate::infrastructure::mqtt_client::MqttClientManager;
use crate::infrastructure::node_transport::{DispatchTransport, GrpcTransport, MqttTransport, NodeTransport};
use sea_orm::DatabaseConnection;

use self::routes::configure;
use handlers::AdminState;

pub async fn serve(port: u16, config: AdminConfigStore, db: DatabaseConnection, mqtt_client: Option<Arc<MqttClientManager>>) -> std::io::Result<()> {
    let node_transport: Option<Arc<dyn NodeTransport>> = mqtt_client.map(|mqtt| {
        let mqtt_transport: Arc<dyn NodeTransport> = Arc::new(MqttTransport::new(mqtt));
        let transport: Arc<dyn NodeTransport> = Arc::new(DispatchTransport::new(
            config.clone(),
            Some(mqtt_transport),
            GrpcTransport::new(),
        ));
        transport
    });
    let state = AdminState {
        config,
        db,
        node_transport,
    };
    
    web::HttpServer::new(move || {
        App::new()
            .state(state.clone())
            .configure(configure)
            .service(
                fs::Files::new("/", "./admin")
                    .show_files_listing()
                    .index_file("index.html"),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}


