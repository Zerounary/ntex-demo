use ntex::web;

use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(handlers::accelerator_bootstrap)
            .service(handlers::sync_profiles)
            .service(handlers::dashboard)
            .service(handlers::library)
            .service(handlers::settings)
            .service(handlers::navigation)
            .service(handlers::create_wechat_ticket)
            .service(handlers::wechat_status)
            .service(handlers::account_login),
    );
}
