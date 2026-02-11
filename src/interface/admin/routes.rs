//! 管理服务器路由配置

use ntex::web;

use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // 查询接口
    cfg.service(handlers::list_nodes);
    cfg.service(handlers::create_node);
    cfg.service(handlers::update_node_meta);
    cfg.service(handlers::query_handler);

    // 游戏配置（Game Config）
    cfg.service(handlers::list_games);
    cfg.service(handlers::get_game);
    cfg.service(handlers::create_game);
    cfg.service(handlers::update_game);
    cfg.service(handlers::delete_game);
    cfg.service(handlers::list_accelerator_nodes);
    cfg.service(handlers::list_game_bindings);
    cfg.service(handlers::upsert_game_binding);
    cfg.service(handlers::delete_game_binding);

    // 链路（Chain）配置管理
    cfg.service(handlers::apply_chain);
    cfg.service(handlers::get_chains);
    cfg.service(handlers::upsert_chain);
    cfg.service(handlers::delete_chain);

    cfg.service(handlers::refresh_node_network_interfaces);

    cfg.service(handlers::firewall_list_rules);
    cfg.service(handlers::firewall_upsert_port_rule);
    cfg.service(handlers::firewall_delete_rule);
    
    // 用户管理
    cfg.service(handlers::add_user);
    cfg.service(handlers::update_user);
    cfg.service(handlers::delete_user);

    cfg.service(handlers::add_inbound);
    cfg.service(handlers::update_inbound);
    cfg.service(handlers::delete_inbound);
    
    // 上游代理管理
    cfg.service(handlers::add_outbound);
    cfg.service(handlers::update_outbound);
    cfg.service(handlers::delete_outbound);
    cfg.service(handlers::query_udp_latency);
    
    // 路由配置管理
    cfg.service(handlers::update_routing);
    cfg.service(handlers::add_routing_rule);
    cfg.service(handlers::update_routing_rule);
    cfg.service(handlers::delete_routing_rule);
    
    // 用户映射管理
    cfg.service(handlers::add_mapping);
    cfg.service(handlers::update_mapping);
    cfg.service(handlers::delete_mapping);
    
    // 维护模式管理
    cfg.service(handlers::get_maintenance_mode);
    cfg.service(handlers::set_maintenance_mode);

    // accelerator_activity 活动配置
    cfg.service(handlers::get_accelerator_activity);
    cfg.service(handlers::set_accelerator_activity);
    
}


