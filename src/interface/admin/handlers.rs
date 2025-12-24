//! 管理服务器处理器

use ntex::http::StatusCode;
use ntex::web::types::{Json, Query, State};
use ntex::web::{self, HttpResponse};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::infrastructure::admin_config::{AdminConfigStore, ChainDefinition, InboundConfig, OutboundConfig, RoutingRule};
use crate::infrastructure::mqtt_client::MqttClientManager;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AdminState {
    pub config: AdminConfigStore,
    pub db: DatabaseConnection,
    pub mqtt_client: Option<std::sync::Arc<MqttClientManager>>,
}

// ========== 链路（Chain）配置管理 ==========

#[web::get("/api/admin/chains")]
pub async fn get_chains(
    state: State<AdminState>,
    Query(params): Query<HashMap<String, String>>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    match state.config.get_chains(node_id).await {
        Ok(chains) => HttpResponse::Ok().json(&serde_json::json!({
            "msg": "ok",
            "data": chains
        })),
        Err(e) => HttpResponse::InternalServerError().json(&serde_json::json!({
            "msg": "error",
            "error": e
        })),
    }
}

#[web::post("/api/admin/chains")]
pub async fn update_chains(
    state: State<AdminState>,
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<Vec<ChainDefinition>>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    if let Err(e) = state.config.update_chains(node_id, body).await {
        return HttpResponse::InternalServerError().json(&serde_json::json!({
            "msg": "error",
            "error": e
        }));
    }

    if let Some(ref mqtt_client) = state.mqtt_client {
        let _ = mqtt_client.publish_update_notification(node_id, "chains").await;
        let _ = mqtt_client.publish_update_notification(node_id, "config").await;
    }

    match state.config.get_chains(node_id).await {
        Ok(chains) => HttpResponse::Ok().json(&serde_json::json!({
            "msg": "ok",
            "data": chains
        })),
        Err(e) => HttpResponse::InternalServerError().json(&serde_json::json!({
            "msg": "error",
            "error": e
        })),
    }
}

#[derive(Deserialize)]
pub struct AddInboundRequest {
    pub tag: String,
    pub protocol: String,
    pub port: i32,
    #[serde(default)]
    pub listen: Option<String>,
    #[serde(default)]
    pub settings: Value,
    #[serde(default)]
    pub stream_settings: Option<Value>,
    #[serde(default)]
    pub sniffing: Option<Value>,
}

#[web::post("/api/admin/inbound")]
pub async fn add_inbound(
    state: State<AdminState>,
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<AddInboundRequest>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let inbound = InboundConfig {
        tag: body.tag,
        protocol: body.protocol,
        port: body.port,
        listen: body.listen,
        settings: body.settings,
        stream_settings: body.stream_settings,
        sniffing: body.sniffing,
    };

    match state.config.add_inbound(node_id, inbound.clone()).await {
        Ok(_) => {
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "inbound").await;
                let _ = mqtt_client.publish_update_notification(node_id, "config").await;
            }
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": inbound
            }))
        }
        Err(e) => HttpResponse::BadRequest().json(&serde_json::json!({
            "msg": "error",
            "error": e
        })),
    }
}

#[derive(Deserialize)]
pub struct UpdateInboundRequest {
    pub protocol: Option<String>,
    pub port: Option<i32>,
    pub listen: Option<Option<String>>,
    pub settings: Option<Value>,
    pub stream_settings: Option<Option<Value>>,
    pub sniffing: Option<Option<Value>>,
}

#[web::put("/api/admin/inbound/{tag}")]
pub async fn update_inbound(
    state: State<AdminState>,
    Query(params): Query<HashMap<String, String>>,
    path: web::types::Path<String>,
    Json(body): Json<UpdateInboundRequest>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let tag = path.into_inner();

    match state
        .config
        .update_inbound(
            node_id,
            &tag,
            body.protocol,
            body.port,
            body.listen,
            body.settings,
            body.stream_settings,
            body.sniffing,
        )
        .await
    {
        Ok(_) => {
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "inbound").await;
                let _ = mqtt_client.publish_update_notification(node_id, "config").await;
            }
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok"
            }))
        }
        Err(e) => HttpResponse::NotFound().json(&serde_json::json!({
            "msg": "error",
            "error": e
        })),
    }
}

#[web::delete("/api/admin/inbound/{tag}")]
pub async fn delete_inbound(
    state: State<AdminState>,
    Query(params): Query<HashMap<String, String>>,
    path: web::types::Path<String>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let tag = path.into_inner();

    match state.config.delete_inbound(node_id, &tag).await {
        Ok(_) => {
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "inbound").await;
                let _ = mqtt_client.publish_update_notification(node_id, "config").await;
            }
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok"
            }))
        }
        Err(e) => HttpResponse::NotFound().json(&serde_json::json!({
            "msg": "error",
            "error": e
        })),
    }
}

/// 从查询参数获取 node_id，如果没有提供则返回错误
fn get_node_id_from_query(params: &HashMap<String, String>) -> Result<u64, HttpResponse> {
    params.get("node_id")
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| HttpResponse::BadRequest().json(&serde_json::json!({
            "msg": "error",
            "error": "node_id 参数是必需的"
        })))
}

// ========== 查询接口 ==========

#[web::get("/api/admin/nodes")]
pub async fn list_nodes(
    state: State<AdminState>,
    query: Query<HashMap<String, String>>,
) -> HttpResponse {
    match state.config.list_nodes().await {
        Ok(nodes) => {
            let filtered_nodes = if let Some(is_online) = query
                .get("is_online")
                .and_then(|v| v.parse::<bool>().ok())
            {
                nodes
                    .into_iter()
                    .filter(|n| {
                        n.get("is_online").and_then(|v| v.as_bool()).unwrap_or(false) == is_online
                    })
                    .collect::<Vec<_>>()
            } else {
                nodes
            };
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": filtered_nodes
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

#[web::post("/api/admin/nodes/{node_id}/network_interfaces/refresh")]
pub async fn refresh_node_network_interfaces(
    state: State<AdminState>,
    path: web::types::Path<u64>,
) -> HttpResponse {
    let node_id = path.into_inner();

    let mqtt_client = match &state.mqtt_client {
        Some(c) => c,
        None => {
            return HttpResponse::ServiceUnavailable().json(&serde_json::json!({
                "msg": "error",
                "error": "MQTT 未连接，无法刷新网络接口信息"
            }))
        }
    };

    let result = mqtt_client.query_node_network_interfaces(node_id, 8).await;
    let Some(result) = result else {
        return HttpResponse::GatewayTimeout().json(&serde_json::json!({
            "msg": "error",
            "error": "刷新网络接口信息超时或失败"
        }));
    };

    if result.get("msg").and_then(|v| v.as_str()) != Some("ok") {
        return HttpResponse::Ok().json(&result);
    }

    let network = result
        .get("data")
        .and_then(|v| v.get("network"))
        .or_else(|| result.get("data"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let network_json = sea_orm::JsonValue::from(network);
    if let Err(e) = state
        .config
        .update_node_network_interfaces(node_id, network_json.clone())
        .await
    {
        return HttpResponse::InternalServerError().json(&serde_json::json!({
            "msg": "error",
            "error": e
        }));
    }

    HttpResponse::Ok().json(&serde_json::json!({
        "msg": "ok",
        "data": {
            "network_interfaces": network_json
        }
    }))
}

#[derive(Deserialize)]
pub struct UpdateNodeMetaRequest {
    #[serde(default)]
    pub name: Option<Option<String>>,
    #[serde(default)]
    pub region: Option<Option<String>>,
    #[serde(default)]
    pub description: Option<Option<String>>,
}

#[web::put("/api/admin/nodes/{node_id}/meta")]
pub async fn update_node_meta(
    state: State<AdminState>,
    path: web::types::Path<u64>,
    Json(body): Json<UpdateNodeMetaRequest>,
) -> HttpResponse {
    let node_id = path.into_inner();

    match state
        .config
        .update_node_meta(node_id, body.name, body.region, body.description)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(&serde_json::json!({
            "msg": "ok"
        })),
        Err(e) => HttpResponse::InternalServerError().json(&serde_json::json!({
            "msg": "error",
            "error": e
        })),
    }
}

#[web::get("/api/admin/query")]
pub async fn query_handler(
    state: State<AdminState>,
    Query(params): Query<HashMap<String, String>>,
) -> HttpResponse {
    let act = params.get("act").map(|s| s.as_str()).unwrap_or("");
    
    // 获取 node_id，优先从查询参数获取，否则返回错误
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    
    match act {
        "user" => {
            // 使用 AdminConfigStore 查询用户数据
            match state.config.get_users(node_id).await {
                Ok(users) => {
                    HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": users
                    }))
                }
                Err(_) => {
                    // 节点配置不存在时返回空数组
                    HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": []
                    }))
                }
            }
        }
        "config" => {
            // 查询 node_config
            match state.config.get_node_config(node_id).await {
                Ok(node_config) => {
                    HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": node_config
                    }))
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(&serde_json::json!({
                        "msg": "error",
                        "error": e
                    }))
                }
            }
        }
        "outbound" => {
            // 使用 AdminConfigStore 查询 outbound 数据
            match state.config.get_outbounds(node_id).await {
                Ok((outbounds, user_mapping)) => {
                    HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": {
                            "outbounds": outbounds,
                            "user_mapping": user_mapping
                        }
                    }))
                }
                Err(_) => {
                    // 节点配置不存在时返回空数据
                    HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": {
                            "outbounds": [],
                            "user_mapping": {}
                        }
                    }))
                }
            }
        }
        "inbound" => {
            match state.config.get_inbounds(node_id).await {
                Ok(inbounds) => {
                    HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": {
                            "inbounds": inbounds
                        }
                    }))
                }
                Err(_) => {
                    HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": {
                            "inbounds": []
                        }
                    }))
                }
            }
        }
        "routing" => {
            // 使用 AdminConfigStore 查询路由配置
            match state.config.get_routing(node_id).await {
                Ok(routing) => {
                    HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": {
                            "domainStrategy": routing.domain_strategy,
                            "rules": routing.rules
                        }
                    }))
                }
                Err(_) => {
                    // 节点配置不存在时返回默认值
                    HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": {
                            "domainStrategy": "AsIs",
                            "rules": []
                        }
                    }))
                }
            }
        }
        "maintenance" => {
            match state.config.get_maintenance_mode(node_id).await {
                Ok(mode) => {
                    HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": {
                            "maintenance_mode": mode,
                            "description": "维护模式：开启时跳过新用户添加，仅允许已存在用户"
                        }
                    }))
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(&serde_json::json!({
                        "msg": "error",
                        "error": e
                    }))
                }
            }
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
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<AddUserRequest>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    
    // 检查维护模式
    let maintenance_mode = match state.config.get_maintenance_mode(node_id).await {
        Ok(mode) => mode,
        Err(e) => {
            return HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }));
        }
    };
    
    if maintenance_mode {
        // 检查用户是否已存在
        let users = match state.config.get_users(node_id).await {
            Ok(users) => users,
            Err(e) => {
                return HttpResponse::InternalServerError().json(&serde_json::json!({
                    "msg": "error",
                    "error": e
                }));
            }
        };
        if !users.iter().any(|u| u.uuid == body.uuid) {
            return HttpResponse::build(StatusCode::SERVICE_UNAVAILABLE).json(&serde_json::json!({
                "msg": "error",
                "error": "维护模式已启用，无法添加新用户。仅允许已存在用户连接。",
                "maintenance_mode": true
            }));
        }
    }
    
    match state.config.add_user(node_id, body.uuid.clone(), body.st, body.dt).await {
        Ok(user) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "user").await;
            }
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": user,
                "message": "用户添加成功"
            }))
        }
        Err(_e) => {
            // 用户已存在，尝试查找并返回
            if let Ok(users) = state.config.get_users(node_id).await {
                if let Some(user) = users.iter().find(|u| u.uuid == body.uuid).cloned() {
                    return HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": user,
                        "message": "用户已存在，未添加"
                    }));
                }
            }
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
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
    Query(params): Query<HashMap<String, String>>,
    path: web::types::Path<u64>,
    Json(body): Json<UpdateUserRequest>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let id = path.into_inner();
    
    match state.config.update_user(node_id, id, body.uuid, body.st, body.dt).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "user").await;
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
    Query(params): Query<HashMap<String, String>>,
    path: web::types::Path<u64>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let id = path.into_inner();
    
    match state.config.delete_user(node_id, id).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "user").await;
                let _ = mqtt_client.publish_update_notification(node_id, "inbound").await;
                let _ = mqtt_client.publish_update_notification(node_id, "outbound").await;
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
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<AddOutboundRequest>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    
    let outbound = OutboundConfig {
        tag: body.tag.clone(),
        protocol: body.protocol,
        settings: body.settings,
        stream_settings: None,
    };
    
    match state.config.add_outbound(node_id, outbound.clone()).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "outbound").await;
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
    Query(params): Query<HashMap<String, String>>,
    path: web::types::Path<String>,
    Json(body): Json<UpdateOutboundRequest>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let tag = path.into_inner();
    
    match state.config.update_outbound(node_id, &tag, body.protocol, body.settings).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "outbound").await;
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
    Query(params): Query<HashMap<String, String>>,
    path: web::types::Path<String>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let tag = path.into_inner();
    
    match state.config.delete_outbound(node_id, &tag).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "outbound").await;
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
    
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
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
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<UpdateRoutingRequest>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    
    // 如果提供了完整的 routing 配置，直接替换
    let result = if let Some(routing) = body.routing {
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
            
            state.config.update_routing(node_id, Some(domain_strategy), Some(rules)).await
        } else {
            Ok(())
        }
    } else {
        state.config.update_routing(node_id, body.domain_strategy, body.rules).await
    };
    
    if let Err(e) = result {
        return HttpResponse::InternalServerError().json(&serde_json::json!({
            "msg": "error",
            "error": e
        }));
    }
    
    // 推送更新通知
    if let Some(ref mqtt_client) = state.mqtt_client {
        let _ = mqtt_client.publish_update_notification(node_id, "routing").await;
    }
    
    match state.config.get_routing(node_id).await {
        Ok(routing) => {
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": {
                    "domainStrategy": routing.domain_strategy,
                    "rules": routing.rules
                }
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

#[web::post("/api/admin/routing/rule")]
pub async fn add_routing_rule(
    state: State<AdminState>,
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<RoutingRule>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    
    match state.config.add_routing_rule(node_id, body.clone()).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "routing").await;
            }
            
            match state.config.get_routing(node_id).await {
                Ok(routing) => {
                    HttpResponse::Ok().json(&serde_json::json!({
                        "msg": "ok",
                        "data": {
                            "domainStrategy": routing.domain_strategy,
                            "rules": routing.rules
                        }
                    }))
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(&serde_json::json!({
                        "msg": "error",
                        "error": e
                    }))
                }
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

#[web::put("/api/admin/routing/rule/{index}")]
pub async fn update_routing_rule(
    state: State<AdminState>,
    Query(params): Query<HashMap<String, String>>,
    path: web::types::Path<usize>,
    Json(body): Json<RoutingRule>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let index = path.into_inner();
    
    match state.config.update_routing_rule(node_id, index, body.clone()).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "routing").await;
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
    Query(params): Query<HashMap<String, String>>,
    path: web::types::Path<usize>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let index = path.into_inner();
    
    match state.config.delete_routing_rule(node_id, index).await {
        Ok(deleted_rule) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "routing").await;
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
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<AddMappingRequest>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    
    match state.config.add_mapping(node_id, body.uuid.clone(), body.outbound_tag.clone()).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "outbound").await;
            }
            
            let mut data = serde_json::Map::new();
            data.insert(body.uuid.clone(), serde_json::Value::String(body.outbound_tag.clone()));
            
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": data
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateMappingRequest {
    pub outbound_tag: String,
}

#[web::put("/api/admin/mapping/{uuid}")]
pub async fn update_mapping(
    state: State<AdminState>,
    Query(params): Query<HashMap<String, String>>,
    path: web::types::Path<String>,
    Json(body): Json<UpdateMappingRequest>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let uuid = path.into_inner();
    
    match state.config.update_mapping(node_id, &uuid, body.outbound_tag.clone()).await {
        Ok(old_tag) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "outbound").await;
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
    Query(params): Query<HashMap<String, String>>,
    path: web::types::Path<String>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let uuid = path.into_inner();
    
    match state.config.delete_mapping(node_id, &uuid).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref mqtt_client) = state.mqtt_client {
                let _ = mqtt_client.publish_update_notification(node_id, "outbound").await;
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
    Query(params): Query<HashMap<String, String>>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    
    match state.config.get_maintenance_mode(node_id).await {
        Ok(mode) => {
            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": {
                    "maintenance_mode": mode,
                    "description": "维护模式：开启时跳过新用户添加，仅允许已存在用户"
                }
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct SetMaintenanceRequest {
    pub enabled: bool,
}

#[web::post("/api/admin/maintenance")]
pub async fn set_maintenance_mode(
    state: State<AdminState>,
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<SetMaintenanceRequest>,
) -> HttpResponse {
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    
    match state.config.set_maintenance_mode(node_id, body.enabled).await {
        Ok(old_mode) => {
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
        Err(e) => {
            HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    }
}
