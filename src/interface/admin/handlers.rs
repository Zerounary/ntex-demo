//! 管理服务器处理器

use ntex::http::StatusCode;
use ntex::web::types::{Json, Query, State};
use ntex::web::{self, HttpResponse};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::infrastructure::admin_config::{AdminConfigStore, OutboundConfig, RoutingRule, NodeConfig, User};
use crate::infrastructure::mqtt_client::MqttClientManager;
use crate::infrastructure::persistence::{
    admin_node_config, admin_user, admin_outbound, admin_routing, admin_user_mapping,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};

#[derive(Clone)]
pub struct AdminState {
    pub config: AdminConfigStore,
    pub db: DatabaseConnection,
    pub mqtt_client: Option<std::sync::Arc<MqttClientManager>>,
}

// ========== 查询接口 ==========

#[web::get("/")]
pub async fn query_handler(
    state: State<AdminState>,
    Query(params): Query<HashMap<String, String>>,
) -> HttpResponse {
    let act = params.get("act").map(|s| s.as_str()).unwrap_or("");
    
    // 获取 node_id，优先从查询参数获取，否则从配置获取
    let node_id = if let Some(node_id_str) = params.get("node_id") {
        node_id_str.parse::<u64>().unwrap_or_else(|_| {
            // 从配置获取默认 node_id（同步方式，因为我们在异步上下文中）
            let node_config = state.config.get_node_config();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    node_config.await.node_id
                })
            })
        })
    } else {
        // 从配置获取默认 node_id
        let node_config = state.config.get_node_config().await;
        node_config.node_id
    };
    
    match act {
        "user" => {
            // 先检查 node_config 是否存在
            let node_config_exists = admin_node_config::Entity::find_by_id(node_id)
                .one(&state.db)
                .await
                .unwrap_or(None)
                .is_some();
            
            if !node_config_exists {
                return HttpResponse::Ok().json(&serde_json::json!({
                    "msg": "ok",
                    "data": []
                }));
            }
            
            // 查询用户数据
            let users = admin_user::Entity::find()
                .filter(admin_user::Column::NodeId.eq(node_id))
                .all(&state.db)
                .await
                .unwrap_or_default();
            
            let users_data: Vec<User> = users.into_iter().map(|u| User {
                id: u.id,
                uuid: u.uuid,
                st: u.st,
                dt: u.dt,
            }).collect();
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": users_data
            }))
        }
        "config" => {
            // 查询或创建 node_config
            let node_config_result = admin_node_config::Entity::find_by_id(node_id)
                .one(&state.db)
                .await;
            
            match node_config_result {
                Ok(Some(config_model)) => {
                    // 配置已存在，直接返回
                    let node_config = NodeConfig {
                        node_id: config_model.node_id,
                        node_type: config_model.node_type,
                        node_speed_limit: config_model.node_speed_limit,
                        traffic_rate: config_model.traffic_rate,
                        sort: config_model.sort,
                        inbounds: serde_json::from_value(config_model.inbounds.clone()).unwrap_or_default(),
                    };
                    HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": node_config
                    }))
                }
                Ok(None) => {
                    // 配置不存在，从内存配置获取并创建
                    let mem_config = state.config.get_node_config().await;
                    let inbounds_json = serde_json::to_value(mem_config.inbounds.clone()).unwrap_or(serde_json::json!([]));
                    
                    let node_type = mem_config.node_type.clone();
                    let node_speed_limit = mem_config.node_speed_limit;
                    let traffic_rate = mem_config.traffic_rate;
                    let sort = mem_config.sort;
                    
                    let new_config = admin_node_config::ActiveModel {
                        node_id: Set(node_id),
                        node_type: Set(node_type),
                        node_speed_limit: Set(node_speed_limit),
                        traffic_rate: Set(traffic_rate),
                        sort: Set(sort),
                        inbounds: Set(inbounds_json),
                        ..Default::default()
                    };
                    
                    if let Err(e) = new_config.insert(&state.db).await {
                        return HttpResponse::InternalServerError().json(&serde_json::json!({
                            "msg": "error",
                            "error": format!("创建节点配置失败: {}", e)
                        }));
                    }
                    
                    HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": mem_config
                    }))
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(&serde_json::json!({
                        "msg": "error",
                        "error": format!("查询节点配置失败: {}", e)
                    }))
                }
            }
        }
        "outbound" => {
            // 先检查 node_config 是否存在
            let node_config_exists = admin_node_config::Entity::find_by_id(node_id)
                .one(&state.db)
                .await
                .unwrap_or(None)
                .is_some();
            
            if !node_config_exists {
                return HttpResponse::Ok().json(&serde_json::json!({
                    "msg": "ok",
                    "data": {
                        "outbounds": [],
                        "user_mapping": {}
                    }
                }));
            }
            
            // 查询 outbound 数据
            let outbounds = admin_outbound::Entity::find()
                .filter(admin_outbound::Column::NodeId.eq(node_id))
                .all(&state.db)
                .await
                .unwrap_or_default();
            
            let outbounds_data: Vec<OutboundConfig> = outbounds.into_iter().map(|o| OutboundConfig {
                tag: o.tag,
                protocol: o.protocol,
                settings: o.settings,
                stream_settings: o.stream_settings,
            }).collect();
            
            // 查询用户映射
            let mappings = admin_user_mapping::Entity::find()
                .filter(admin_user_mapping::Column::NodeId.eq(node_id))
                .all(&state.db)
                .await
                .unwrap_or_default();
            
            let mut user_mapping = HashMap::new();
            for m in mappings {
                user_mapping.insert(m.uuid, m.outbound_tag);
            }
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": {
                    "outbounds": outbounds_data,
                    "user_mapping": user_mapping
                }
            }))
        }
        "routing" => {
            // 先检查 node_config 是否存在
            let node_config_exists = admin_node_config::Entity::find_by_id(node_id)
                .one(&state.db)
                .await
                .unwrap_or(None)
                .is_some();
            
            if !node_config_exists {
                return HttpResponse::Ok().json(&serde_json::json!({
                    "msg": "ok",
                    "data": {
                        "domainStrategy": "AsIs",
                        "rules": []
                    }
                }));
            }
            
            // 查询路由配置
            let routing = admin_routing::Entity::find_by_id(node_id)
                .one(&state.db)
                .await
                .unwrap_or(None);
            
            if let Some(r) = routing {
                let rules: Vec<RoutingRule> = serde_json::from_value(r.rules.clone()).unwrap_or_default();
                HttpResponse::Ok().json(&serde_json::json!({
                    "msg": "ok",
                    "data": {
                        "domainStrategy": r.domain_strategy,
                        "rules": rules
                    }
                }))
            } else {
                // 路由配置不存在，返回默认值
                HttpResponse::Ok().json(&serde_json::json!({
                    "msg": "ok",
                    "data": {
                        "domainStrategy": "AsIs",
                        "rules": []
                    }
                }))
            }
        }
        "maintenance" => {
            let mode = state.config.get_maintenance_mode().await;
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": {
                    "maintenance_mode": mode,
                    "description": "维护模式：开启时跳过新用户添加，仅允许已存在用户"
                }
            }))
        }
        "user_logs" => {
            let uid = params.get("uid").and_then(|s| s.parse::<u64>().ok());
            if uid.is_none() {
                return HttpResponse::BadRequest().json(&serde_json::json!({
                    "msg": "error",
                    "error": "uid 参数是必需的"
                }));
            }
            
            let days = params.get("days").and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let limit = params.get("limit").and_then(|s| s.parse::<u64>().ok()).unwrap_or(100);
            let event = params.get("event").cloned();
            
            let query_params = serde_json::json!({
                "uid": uid.unwrap(),
                "days": days,
                "limit": limit,
                "event": event
            });
            
            if let Some(ref mqtt_client) = state.mqtt_client {
                let result = mqtt_client.query_node_logs(node_id, query_params, 15).await;
                
                if let Some(result) = result {
                    HttpResponse::Ok().json(&result)
                } else {
                    HttpResponse::GatewayTimeout().json(&serde_json::json!({
                        "msg": "error",
                        "error": "查询日志超时或失败"
                    }))
                }
            } else {
                HttpResponse::ServiceUnavailable().json(&serde_json::json!({
                    "msg": "error",
                    "error": "MQTT 未连接，无法查询日志"
                }))
            }
        }
        _ => {
            HttpResponse::NotFound().json(&serde_json::json!({
                "msg": "error",
                "error": "未知的操作"
            }))
        }
    }
}

// ========== 用户管理 ==========

#[derive(Deserialize)]
pub struct AddUserRequest {
    pub uuid: String,
    #[serde(default = "default_st")]
    pub st: u64,
    #[serde(default = "default_dt")]
    pub dt: u64,
}

fn default_st() -> u64 { 1 }
fn default_dt() -> u64 { 0 }

#[web::post("/api/admin/user")]
pub async fn add_user(
    state: State<AdminState>,
    Json(body): Json<AddUserRequest>,
) -> HttpResponse {
    // 检查维护模式
    let maintenance_mode = state.config.get_maintenance_mode().await;
    if maintenance_mode {
        // 检查用户是否已存在
        let users = state.config.get_users().await;
        if !users.iter().any(|u| u.uuid == body.uuid) {
            return HttpResponse::build(StatusCode::SERVICE_UNAVAILABLE).json(&serde_json::json!({
                "msg": "error",
                "error": "维护模式已启用，无法添加新用户。仅允许已存在用户连接。",
                "maintenance_mode": true
            }));
        }
    }
    
    match state.config.add_user(body.uuid.clone(), body.st, body.dt).await {
            Ok(user) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let node_config = state.config.get_node_config().await;
                let _ = mqtt_client.publish_update_notification(node_config.node_id, "user").await;
            }
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": user,
                "message": "用户添加成功"
            }))
        }
        Err(_e) => {
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": state.config.get_users().await.iter().find(|u| u.uuid == body.uuid).cloned(),
                "message": "用户已存在，未添加"
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub uuid: Option<String>,
    pub st: Option<u64>,
    pub dt: Option<u64>,
}

#[web::put("/api/admin/user/{id}")]
pub async fn update_user(
    state: State<AdminState>,
    path: web::types::Path<u64>,
    Json(body): Json<UpdateUserRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    
    match state.config.update_user(id, body.uuid, body.st, body.dt).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let node_config = state.config.get_node_config().await;
                let _ = mqtt_client.publish_update_notification(node_config.node_id, "user").await;
            }
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok"
            }))
        }
        Err(e) => {
            HttpResponse::NotFound().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

#[web::delete("/api/admin/user/{id}")]
pub async fn delete_user(
    state: State<AdminState>,
    path: web::types::Path<u64>,
) -> HttpResponse {
    let id = path.into_inner();
    
    match state.config.delete_user(id).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let node_config = state.config.get_node_config().await;
                let _ = mqtt_client.publish_update_notification(node_config.node_id, "user").await;
                let _ = mqtt_client.publish_update_notification(node_config.node_id, "outbound").await;
            }
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok"
            }))
        }
        Err(e) => {
            HttpResponse::NotFound().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

// ========== 上游代理管理 ==========

#[derive(Deserialize)]
pub struct AddOutboundRequest {
    pub tag: String,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default)]
    pub settings: Value,
}

fn default_protocol() -> String { "shadowsocks".to_string() }

#[web::post("/api/admin/outbound")]
pub async fn add_outbound(
    state: State<AdminState>,
    Json(body): Json<AddOutboundRequest>,
) -> HttpResponse {
    let outbound = OutboundConfig {
        tag: body.tag.clone(),
        protocol: body.protocol,
        settings: body.settings,
        stream_settings: None,
    };
    
    match state.config.add_outbound(outbound.clone()).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let node_config = state.config.get_node_config().await;
                let _ = mqtt_client.publish_update_notification(node_config.node_id, "outbound").await;
            }
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": outbound
            }))
        }
        Err(e) => {
            HttpResponse::BadRequest().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateOutboundRequest {
    pub protocol: Option<String>,
    pub settings: Option<Value>,
}

#[web::put("/api/admin/outbound/{tag}")]
pub async fn update_outbound(
    state: State<AdminState>,
    path: web::types::Path<String>,
    Json(body): Json<UpdateOutboundRequest>,
) -> HttpResponse {
    let tag = path.into_inner();
    
    match state.config.update_outbound(&tag, body.protocol, body.settings).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let node_config = state.config.get_node_config().await;
                let _ = mqtt_client.publish_update_notification(node_config.node_id, "outbound").await;
            }
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok"
            }))
        }
        Err(e) => {
            HttpResponse::NotFound().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

#[web::delete("/api/admin/outbound/{tag}")]
pub async fn delete_outbound(
    state: State<AdminState>,
    path: web::types::Path<String>,
) -> HttpResponse {
    let tag = path.into_inner();
    
    match state.config.delete_outbound(&tag).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let node_config = state.config.get_node_config().await;
                let _ = mqtt_client.publish_update_notification(node_config.node_id, "outbound").await;
            }
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok"
            }))
        }
        Err(e) => {
            HttpResponse::NotFound().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

#[web::get("/api/admin/outbound/udp_latency")]
pub async fn query_udp_latency(
    state: State<AdminState>,
    Query(params): Query<HashMap<String, String>>,
) -> HttpResponse {
    let outbound_tag = params.get("outbound_tag");
    if outbound_tag.is_none() {
        return HttpResponse::BadRequest().json(&serde_json::json!({
            "msg": "error",
            "error": "outbound_tag 参数是必需的"
        }));
    }
    
    let node_id = if let Some(node_id_str) = params.get("node_id") {
        node_id_str.parse::<u64>().ok().unwrap_or_else(|| {
            // 从配置获取默认 node_id（同步方式）
            let node_config = state.config.get_node_config();
            // 使用 tokio::task::block_in_place 在异步上下文中执行同步操作
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    node_config.await.node_id
                })
            })
        })
    } else {
        // 从配置获取默认 node_id
        let node_config = state.config.get_node_config().await;
        node_config.node_id
    };
    
    if let Some(ref mqtt_client) = state.mqtt_client {
        let result = mqtt_client.query_udp_latency(
            node_id,
            outbound_tag.unwrap(),
            8,  // timeout 8秒
        ).await;
        
        if let Some(result) = result {
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": result
            }))
        } else {
            HttpResponse::GatewayTimeout().json(&serde_json::json!({
                "msg": "error",
                "error": "查询 UDP 延迟超时或失败"
            }))
        }
    } else {
        HttpResponse::ServiceUnavailable().json(&serde_json::json!({
            "msg": "error",
            "error": "MQTT 未连接，无法查询 UDP 延迟"
        }))
    }
}

// ========== 路由配置管理 ==========

#[derive(Deserialize)]
pub struct UpdateRoutingRequest {
    pub domain_strategy: Option<String>,
    pub rules: Option<Vec<RoutingRule>>,
    pub routing: Option<Value>,  // 完整的路由配置
}

#[web::post("/api/admin/routing")]
pub async fn update_routing(
    state: State<AdminState>,
    Json(body): Json<UpdateRoutingRequest>,
) -> HttpResponse {
    // 如果提供了完整的 routing 配置，直接替换
    if let Some(routing) = body.routing {
        if let Ok(routing_config) = serde_json::from_value::<serde_json::Map<String, Value>>(routing) {
            let domain_strategy = routing_config.get("domainStrategy")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "AsIs".to_string());
            
            let rules = routing_config.get("rules")
                .and_then(|v| v.as_array())
                .and_then(|arr| {
                    arr.iter()
                        .map(|v| serde_json::from_value::<RoutingRule>(v.clone()))
                        .collect::<Result<Vec<_>, _>>()
                        .ok()
                })
                .unwrap_or_default();
            
            state.config.update_routing(Some(domain_strategy), Some(rules)).await;
        }
    } else {
        state.config.update_routing(body.domain_strategy, body.rules).await;
    }
    
    // 推送更新通知
    if let Some(ref mqtt_client) = state.mqtt_client {
        let node_config = state.config.get_node_config().await;
        let _ = mqtt_client.publish_update_notification(node_config.node_id, "routing").await;
    }
    
    let routing = state.config.get_routing().await;
    HttpResponse::Ok().json(&serde_json::json!({
        "msg": "ok",
        "data": {
            "domainStrategy": routing.domain_strategy,
            "rules": routing.rules
        }
    }))
}

#[web::post("/api/admin/routing/rule")]
pub async fn add_routing_rule(
    state: State<AdminState>,
    Json(body): Json<RoutingRule>,
) -> HttpResponse {
    state.config.add_routing_rule(body).await;
    
    // 推送更新通知
    if let Some(ref mqtt_client) = state.mqtt_client {
        let node_config = state.config.get_node_config().await;
        let _ = mqtt_client.publish_update_notification(node_config.node_id, "routing").await;
    }
    
    let routing = state.config.get_routing().await;
    HttpResponse::Ok().json(&serde_json::json!({
        "msg": "ok",
        "data": {
            "domainStrategy": routing.domain_strategy,
            "rules": routing.rules
        }
    }))
}

#[web::put("/api/admin/routing/rule/{index}")]
pub async fn update_routing_rule(
    state: State<AdminState>,
    path: web::types::Path<usize>,
    Json(body): Json<RoutingRule>,
) -> HttpResponse {
    let index = path.into_inner();
    
    match state.config.update_routing_rule(index, body.clone()).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let node_config = state.config.get_node_config().await;
                let _ = mqtt_client.publish_update_notification(node_config.node_id, "routing").await;
            }
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": body
            }))
        }
        Err(e) => {
            HttpResponse::NotFound().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

#[web::delete("/api/admin/routing/rule/{index}")]
pub async fn delete_routing_rule(
    state: State<AdminState>,
    path: web::types::Path<usize>,
) -> HttpResponse {
    let index = path.into_inner();
    
    match state.config.delete_routing_rule(index).await {
        Ok(deleted_rule) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let node_config = state.config.get_node_config().await;
                let _ = mqtt_client.publish_update_notification(node_config.node_id, "routing").await;
            }
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": {
                    "deleted_rule": deleted_rule
                }
            }))
        }
        Err(e) => {
            HttpResponse::NotFound().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

// ========== 用户映射管理 ==========

#[derive(Deserialize)]
pub struct AddMappingRequest {
    pub uuid: String,
    pub outbound_tag: String,
}

#[web::post("/api/admin/mapping")]
pub async fn add_mapping(
    state: State<AdminState>,
    Json(body): Json<AddMappingRequest>,
) -> HttpResponse {
    state.config.add_mapping(body.uuid.clone(), body.outbound_tag.clone()).await;
    
    // 推送更新通知
    if let Some(ref mqtt_client) = state.mqtt_client {
        let node_config = state.config.get_node_config().await;
        let _ = mqtt_client.publish_update_notification(node_config.node_id, "outbound").await;
    }
    
    let mut data = serde_json::Map::new();
    data.insert(body.uuid.clone(), serde_json::Value::String(body.outbound_tag.clone()));
    
    HttpResponse::Ok().json(&serde_json::json!({
        "msg": "ok",
        "data": data
    }))
}

#[derive(Deserialize)]
pub struct UpdateMappingRequest {
    pub outbound_tag: String,
}

#[web::put("/api/admin/mapping/{uuid}")]
pub async fn update_mapping(
    state: State<AdminState>,
    path: web::types::Path<String>,
    Json(body): Json<UpdateMappingRequest>,
) -> HttpResponse {
    let uuid = path.into_inner();
    
    match state.config.update_mapping(&uuid, body.outbound_tag.clone()).await {
        Ok(old_tag) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let node_config = state.config.get_node_config().await;
                let _ = mqtt_client.publish_update_notification(node_config.node_id, "outbound").await;
            }
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": {
                    "uuid": uuid,
                    "old_outbound_tag": old_tag,
                    "new_outbound_tag": body.outbound_tag
                }
            }))
        }
        Err(e) => {
            HttpResponse::NotFound().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

#[web::delete("/api/admin/mapping/{uuid}")]
pub async fn delete_mapping(
    state: State<AdminState>,
    path: web::types::Path<String>,
) -> HttpResponse {
    let uuid = path.into_inner();
    
    match state.config.delete_mapping(&uuid).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let node_config = state.config.get_node_config().await;
                let _ = mqtt_client.publish_update_notification(node_config.node_id, "outbound").await;
            }
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok"
            }))
        }
        Err(e) => {
            HttpResponse::NotFound().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

// ========== 维护模式管理 ==========

#[web::get("/api/admin/maintenance")]
pub async fn get_maintenance_mode(
    state: State<AdminState>,
) -> HttpResponse {
    let mode = state.config.get_maintenance_mode().await;
    HttpResponse::Ok().json(&serde_json::json!({
        "msg": "ok",
        "data": {
            "maintenance_mode": mode,
            "description": "维护模式：开启时跳过新用户添加，仅允许已存在用户"
        }
    }))
}

#[derive(Deserialize)]
pub struct SetMaintenanceRequest {
    pub enabled: bool,
}

#[web::post("/api/admin/maintenance")]
pub async fn set_maintenance_mode(
    state: State<AdminState>,
    Json(body): Json<SetMaintenanceRequest>,
) -> HttpResponse {
    let old_mode = state.config.set_maintenance_mode(body.enabled).await;
    let mode_str = if body.enabled { "启用" } else { "禁用" };
    
    HttpResponse::Ok().json(&serde_json::json!({
        "msg": "ok",
        "data": {
            "maintenance_mode": body.enabled,
            "previous_mode": old_mode,
            "message": format!("维护模式已{}", mode_str)
        }
    }))
}
