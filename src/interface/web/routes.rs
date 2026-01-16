use ntex::web;

use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(handlers::accelerator_bootstrap)
            .service(handlers::accelerator_games)
            .service(handlers::dashboard_announcements)
            .service(handlers::library_games)
            .service(handlers::accelerator_search)
            .service(handlers::sync_profiles)
            .service(handlers::session_start)
            .service(handlers::session_stop)
            .service(handlers::accelerator_user_register)
            .service(handlers::accelerator_user_login)
            .service(handlers::accelerator_user_me)
            .service(handlers::accelerator_user_update_profile)
            .service(handlers::accelerator_user_change_password)
            .service(handlers::accelerator_user_logout)
            .service(handlers::list_game_bindings)
            .service(handlers::list_game_nodes)
            .service(handlers::set_game_nodes)
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
