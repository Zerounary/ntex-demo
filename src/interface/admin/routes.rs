//! 管理服务器路由配置

use ntex::web;

use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // 查询接口
    cfg.service(handlers::list_nodes);
    cfg.service(handlers::query_handler);
    
    // 用户管理
    cfg.service(handlers::add_user);
    cfg.service(handlers::update_user);
    cfg.service(handlers::delete_user);
    
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
    
}


