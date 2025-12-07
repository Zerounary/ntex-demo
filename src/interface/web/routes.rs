use ntex::web;

use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/accelerator")
                    .route("/bootstrap", web::get().to(handlers::accelerator_bootstrap))
                    .route("/profiles", web::post().to(handlers::sync_profiles)),
            )
            .route("/dashboard", web::get().to(handlers::dashboard))
            .route("/library", web::get().to(handlers::library))
            .route("/settings/meta", web::get().to(handlers::settings))
            .route("/navigation", web::get().to(handlers::navigation))
            .service(
                web::scope("/auth")
                    .route(
                        "/wechat/ticket",
                        web::post().to(handlers::create_wechat_ticket),
                    )
                    .route("/wechat/status", web::get().to(handlers::wechat_status))
                    .route("/account", web::post().to(handlers::account_login)),
            ),
    );
}
