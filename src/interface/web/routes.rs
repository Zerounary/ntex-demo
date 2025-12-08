use ntex::web;

use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(handlers::accelerator_bootstrap)
            .service(handlers::sync_profiles)
            .service(handlers::dashboard)
            .service(handlers::library)
            .service(handlers::settings)
            .service(handlers::navigation)
            .service(handlers::create_wechat_ticket)
            .service(handlers::wechat_status)
            .service(handlers::account_login)
            .service(handlers::register_node)
            .service(handlers::unregister_node)
            .service(handlers::node_heartbeat)
            .service(handlers::list_nodes)
            .service(handlers::list_active_nodes)
            .service(handlers::generate_cdks)
            .service(handlers::redeem_cdk)
            .service(handlers::list_cdks)
            .service(handlers::validate_account)
            .service(handlers::start_acceleration),
    );
}
