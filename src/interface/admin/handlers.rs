//! 管理服务器处理器

use ntex::http::StatusCode;
use ntex::web::types::{Json, Query, State};
use ntex::web::{self, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::time::Instant;
use log::{info, error};
use uuid::Uuid;

use crate::infrastructure::admin_config::{AdminConfigStore, ChainDefinition, InboundConfig, OutboundConfig, RoutingRule};
use crate::infrastructure::node_transport::NodeTransport;
use crate::infrastructure::persistence::{
    accelerator_game, accelerator_game_node_binding, accelerator_node, admin_node_config, admin_inbound, admin_outbound,
};
use crate::interface::admin::chain_ops;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};
use chrono::Utc;

fn default_chain_type() -> String {
    "tcp".to_string()
}

#[derive(Clone)]
pub struct AdminState {
    pub config: AdminConfigStore,
    pub db: DatabaseConnection,
    pub node_transport: Option<std::sync::Arc<dyn NodeTransport>>,
}

fn require_admin_token(req: &HttpRequest) -> Result<(), HttpResponse> {
    let expected = env::var("ADMIN_TOKEN").unwrap_or_default();
    if expected.is_empty() {
        return Ok(());
    }

    let provided = req
        .headers()
        .get("X-Admin-Token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided != expected {
        return Err(HttpResponse::Unauthorized().json(&serde_json::json!({
            "msg": "error",
            "error": "unauthorized"
        })));
    }
    Ok(())
}

fn parse_sync_from_query(params: &HashMap<String, String>) -> (bool, u64) {
    let sync = params
        .get("sync")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let timeout_secs = params
        .get("sync_timeout")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(5);
    (sync, timeout_secs)
}

#[derive(Deserialize)]
pub struct ApplyChainRequest {
    pub chain_id: i64,
    #[serde(default)]
    pub base_port: Option<u16>,
}

#[derive(Deserialize)]
pub struct ListGamesQuery {
    #[serde(default)]
    pub keyword: Option<String>,
}

#[web::get("/api/admin/games")]
pub async fn list_games(
    state: State<AdminState>,
    req: HttpRequest,
    Query(query): Query<ListGamesQuery>,
) -> HttpResponse {
    if let Err(resp) = require_admin_token(&req) {
        return resp;
    }

    let mut q = accelerator_game::Entity::find().order_by_desc(accelerator_game::Column::Id);
    if let Some(keyword) = query.keyword.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        q = q.filter(
            sea_orm::Condition::any()
                .add(accelerator_game::Column::Name.contains(keyword))
                .add(accelerator_game::Column::Region.contains(keyword))
                .add(accelerator_game::Column::Status.contains(keyword))
                .add(accelerator_game::Column::ProcessName.contains(keyword)),
        );
    }

    match q.all(&state.db).await {
        Ok(items) => HttpResponse::Ok().json(&serde_json::json!({
            "msg": "ok",
            "data": items.into_iter().map(|m| serde_json::json!({
                "id": m.id,
                "name": m.name,
                "icon": m.icon,
                "status": m.status,
                "ping": m.ping,
                "process_name": m.process_name,
                "region": m.region
            })).collect::<Vec<_>>()
        })),
        Err(e) => HttpResponse::InternalServerError().json(&serde_json::json!({
            "msg": "error",
            "error": format!("list games failed: {}", e)
        })),
    }
}

#[web::get("/api/admin/accelerator_nodes")]
pub async fn list_accelerator_nodes(state: State<AdminState>, req: HttpRequest) -> HttpResponse {
    if let Err(resp) = require_admin_token(&req) {
        return resp;
    }

    match accelerator_node::Entity::find().order_by_asc(accelerator_node::Column::Id).all(&state.db).await {
        Ok(items) => HttpResponse::Ok().json(&serde_json::json!({
            "msg": "ok",
            "data": items.into_iter().map(|n| serde_json::json!({
                "id": n.id,
                "mode": n.mode,
                "ping": n.ping,
                "status": n.status
            })).collect::<Vec<_>>()
        })),
        Err(e) => HttpResponse::InternalServerError().json(&serde_json::json!({
            "msg": "error",
            "error": format!("list accelerator nodes failed: {}", e)
        })),
    }
}

#[web::get("/api/admin/games/{game_id}/bindings")]
pub async fn list_game_bindings(
    state: State<AdminState>,
    req: HttpRequest,
    path: web::types::Path<String>,
) -> HttpResponse {
    if let Err(resp) = require_admin_token(&req) {
        return resp;
    }

    let game_id = path.into_inner();
    let game = match accelerator_game::Entity::find_by_id(game_id.clone()).one(&state.db).await {
        Ok(Some(g)) => g,
        Ok(None) => {
            return HttpResponse::NotFound().json(&serde_json::json!({
                "msg": "error",
                "error": "game not found"
            }))
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": format!("query game failed: {}", e)
            }))
        }
    };

    let bindings = match accelerator_game_node_binding::Entity::find()
        .filter(accelerator_game_node_binding::Column::GameId.eq(game_id.as_str()))
        .order_by_asc(accelerator_game_node_binding::Column::CreatedAt)
        .all(&state.db)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": format!("query bindings failed: {}", e)
            }))
        }
    };

    if bindings.is_empty() {
        return HttpResponse::Ok().json(&serde_json::json!({
            "msg": "ok",
            "data": Vec::<Value>::new()
        }));
    }

    let node_ids: Vec<String> = bindings
        .iter()
        .filter_map(|b| b.node_id.clone())
        .collect();
    let nodes = match accelerator_node::Entity::find()
        .filter(accelerator_node::Column::Id.is_in(node_ids.clone()))
        .all(&state.db)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": format!("query nodes failed: {}", e)
            }))
        }
    };

    let node_map: std::collections::HashMap<String, accelerator_node::Model> =
        nodes.into_iter().map(|n| (n.id.clone(), n)).collect();

    let payload: Vec<Value> = bindings
        .into_iter()
        .map(|b| {
            let n = b
                .node_id
                .as_ref()
                .and_then(|node_id| node_map.get(node_id));
            serde_json::json!({
                "id": b.id,
                "type": b.r#type,
                "node_id": b.node_id,
                "display_name": b.display_name.clone().or_else(|| n.map(|x| x.id.clone())).unwrap_or_default(),
                "region": b.region.clone().unwrap_or_else(|| game.region.clone()),
                "mode": b.mode.clone().or_else(|| n.map(|x| x.mode.clone())).unwrap_or_default(),
                "ping": b.ping.or_else(|| n.map(|x| x.ping)).unwrap_or_default(),
                "status": b.status.clone().or_else(|| n.map(|x| x.status.clone())).unwrap_or_default(),
                "tcp_chain_id": b.tcp_chain_id,
                "udp_chain_id": b.udp_chain_id,
                "remark": b.remark,
                "created_at": b.created_at.to_rfc3339(),
                "updated_at": b.updated_at.to_rfc3339()
            })
        })
        .collect();

    HttpResponse::Ok().json(&serde_json::json!({
        "msg": "ok",
        "data": payload
    }))
}

#[derive(Deserialize)]
pub struct UpsertGameBindingRequest {
    #[serde(default)]
    pub id: Option<i64>,
    pub r#type: String,
    #[serde(default, alias = "node_id")]
    pub node_id: Option<String>,
    #[serde(default, alias = "tcp_chain_id")]
    pub tcp_chain_id: Option<i64>,
    #[serde(default, alias = "udp_chain_id")]
    pub udp_chain_id: Option<i64>,
    #[serde(default, alias = "display_name")]
    pub display_name: Option<String>,
    #[serde(default, alias = "region")]
    pub region: Option<String>,
    #[serde(default, alias = "mode")]
    pub mode: Option<String>,
    #[serde(default, alias = "ping")]
    pub ping: Option<i32>,
    #[serde(default, alias = "status")]
    pub status: Option<String>,
    #[serde(default, alias = "remark")]
    pub remark: Option<String>,
}

#[web::post("/api/admin/games/{game_id}/bindings")]
pub async fn upsert_game_binding(
    state: State<AdminState>,
    req: HttpRequest,
    path: web::types::Path<String>,
    Json(body): Json<UpsertGameBindingRequest>,
) -> HttpResponse {
    if let Err(resp) = require_admin_token(&req) {
        return resp;
    }

    let game_id = path.into_inner();
    let binding_type = body.r#type.trim().to_lowercase();
    if binding_type != "node" && binding_type != "chain" {
        return HttpResponse::BadRequest().json(&serde_json::json!({
            "msg": "error",
            "error": "type must be node or chain"
        }));
    }

    let node_id = body.node_id.clone().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());

    let game_exists = match accelerator_game::Entity::find_by_id(game_id.clone()).one(&state.db).await {
        Ok(v) => v.is_some(),
        Err(e) => {
            return HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": format!("query game failed: {}", e)
            }))
        }
    };
    if !game_exists {
        return HttpResponse::BadRequest().json(&serde_json::json!({
            "msg": "error",
            "error": "game not found"
        }));
    }

    if binding_type == "node" {
        let node_id = match node_id.clone() {
            Some(v) => v,
            None => {
                return HttpResponse::BadRequest().json(&serde_json::json!({
                    "msg": "error",
                    "error": "node_id is required when type=node"
                }));
            }
        };

        let node_id_u64: u64 = match node_id.parse() {
            Ok(v) => v,
            Err(_) => {
                return HttpResponse::BadRequest().json(&serde_json::json!({
                    "msg": "error",
                    "error": "node_id must be a number"
                }));
            }
        };

        let node_exists = match admin_node_config::Entity::find_by_id(node_id_u64)
            .one(&state.db)
            .await
        {
            Ok(v) => v.is_some(),
            Err(e) => {
                return HttpResponse::InternalServerError().json(&serde_json::json!({
                    "msg": "error",
                    "error": format!("query node failed: {}", e)
                }))
            }
        };
        if !node_exists {
            return HttpResponse::BadRequest().json(&serde_json::json!({
                "msg": "error",
                "error": "node not found"
            }));
        }
    }

    let chains = match state.config.get_chains().await {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": format!("get chains failed: {}", e)
            }))
        }
    };
    if let Some(cid) = body.tcp_chain_id {
        if !chains.iter().any(|c| c.id == cid) {
            return HttpResponse::BadRequest().json(&serde_json::json!({
                "msg": "error",
                "error": "tcp_chain_id not found"
            }));
        }
    }
    if let Some(cid) = body.udp_chain_id {
        if !chains.iter().any(|c| c.id == cid) {
            return HttpResponse::BadRequest().json(&serde_json::json!({
                "msg": "error",
                "error": "udp_chain_id not found"
            }));
        }
    }

    let now = Utc::now();
    let existing = if let Some(id) = body.id {
        match accelerator_game_node_binding::Entity::find_by_id(id)
            .one(&state.db)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                return HttpResponse::InternalServerError().json(&serde_json::json!({
                    "msg": "error",
                    "error": format!("query binding failed: {}", e)
                }))
            }
        }
    } else {
        None
    };

    let saved = if let Some(existing) = existing {
        if existing.game_id != game_id {
            return HttpResponse::BadRequest().json(&serde_json::json!({
                "msg": "error",
                "error": "binding does not belong to this game"
            }));
        }
        let mut active: accelerator_game_node_binding::ActiveModel = existing.into();
        active.r#type = Set(binding_type.clone());
        active.node_id = Set(node_id.clone());
        active.tcp_chain_id = Set(body.tcp_chain_id);
        active.udp_chain_id = Set(body.udp_chain_id);
        active.display_name = Set(body.display_name);
        active.region = Set(body.region);
        active.mode = Set(body.mode);
        active.ping = Set(body.ping);
        active.status = Set(body.status);
        active.remark = Set(body.remark);
        active.updated_at = Set(now.into());
        match active.update(&state.db).await {
            Ok(v) => v,
            Err(e) => {
                return HttpResponse::InternalServerError().json(&serde_json::json!({
                    "msg": "error",
                    "error": format!("update binding failed: {}", e)
                }))
            }
        }
    } else {
        let active = accelerator_game_node_binding::ActiveModel {
            game_id: Set(game_id.clone()),
            r#type: Set(binding_type.clone()),
            node_id: Set(node_id.clone()),
            tcp_chain_id: Set(body.tcp_chain_id),
            udp_chain_id: Set(body.udp_chain_id),
            display_name: Set(body.display_name),
            region: Set(body.region),
            mode: Set(body.mode),
            ping: Set(body.ping),
            status: Set(body.status),
            remark: Set(body.remark),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            ..Default::default()
        };
        match active.insert(&state.db).await {
            Ok(v) => v,
            Err(e) => {
                return HttpResponse::InternalServerError().json(&serde_json::json!({
                    "msg": "error",
                    "error": format!("create binding failed: {}", e)
                }))
            }
        }
    };

    HttpResponse::Ok().json(&serde_json::json!({
        "msg": "ok",
        "data": {
            "id": saved.id,
            "game_id": saved.game_id,
            "type": saved.r#type,
            "node_id": saved.node_id,
            "tcp_chain_id": saved.tcp_chain_id,
            "udp_chain_id": saved.udp_chain_id,
            "display_name": saved.display_name,
            "region": saved.region,
            "mode": saved.mode,
            "ping": saved.ping,
            "status": saved.status,
            "remark": saved.remark,
            "created_at": saved.created_at.to_rfc3339(),
            "updated_at": saved.updated_at.to_rfc3339()
        }
    }))
}

#[web::delete("/api/admin/games/{game_id}/bindings/{id}")]
pub async fn delete_game_binding(
    state: State<AdminState>,
    req: HttpRequest,
    path: web::types::Path<(String, i64)>,
) -> HttpResponse {
    if let Err(resp) = require_admin_token(&req) {
        return resp;
    }

    let (game_id, id) = path.into_inner();

    let existing = match accelerator_game_node_binding::Entity::find_by_id(id)
        .one(&state.db)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": format!("query binding failed: {}", e)
            }))
        }
    };

    let existing = match existing {
        Some(v) => v,
        None => {
            return HttpResponse::NotFound().json(&serde_json::json!({
                "msg": "error",
                "error": "binding not found"
            }))
        }
    };

    if existing.game_id != game_id {
        return HttpResponse::BadRequest().json(&serde_json::json!({
            "msg": "error",
            "error": "binding does not belong to this game"
        }));
    }

    match accelerator_game_node_binding::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(&serde_json::json!({ "msg": "ok" })),
        Err(e) => HttpResponse::InternalServerError().json(&serde_json::json!({
            "msg": "error",
            "error": format!("delete binding failed: {}", e)
        })),
    }
}

#[web::post("/api/admin/chains/apply")]
pub async fn apply_chain(
    state: State<AdminState>,
    req: HttpRequest,
    Query(_params): Query<HashMap<String, String>>,
    Json(body): Json<ApplyChainRequest>,
) -> HttpResponse {
    if let Err(resp) = require_admin_token(&req) {
        return resp;
    }
    let base_port: u16 = body.base_port.unwrap_or(40000);

    let chain_type = match state.config.get_chains().await {
        Ok(chains) => chains
            .into_iter()
            .find(|c| c.id == body.chain_id)
            .map(|c| c.chain_type)
            .unwrap_or_else(|| "tcp".to_string()),
        Err(e) => {
            return HttpResponse::BadRequest().json(&serde_json::json!({
                "msg": "error",
                "error": e
            }))
        }
    };

    let chain_type = chain_type.trim().to_lowercase();

    let apply_res = if chain_type == "udp" {
        chain_ops::apply_chain_udp(
            &state.config,
            state.node_transport.as_ref(),
            body.chain_id,
            base_port,
        )
        .await
    } else {
        chain_ops::apply_chain(&state.config, state.node_transport.as_ref(), body.chain_id, base_port).await
    };

    match apply_res {
        Ok((cid, results)) => HttpResponse::Ok().json(&serde_json::json!({
            "msg": "ok",
            "data": {
                "chain_id": cid,
                "applied_nodes": results
            }
        })),
        Err(e) => HttpResponse::BadRequest().json(&serde_json::json!({
            "msg": "error",
            "error": e
        })),
    }
}

// ========== 链路（Chain）配置管理 ==========

#[web::get("/api/admin/chains")]
pub async fn get_chains(
    state: State<AdminState>,
    Query(_params): Query<HashMap<String, String>>,
) -> HttpResponse {
    match state.config.get_chains().await {
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
#[serde(rename_all = "camelCase")]
pub struct UpsertChainRequest {
    #[serde(default)]
    pub id: Option<i64>,
    pub name: String,
    pub protocol: String,
    #[serde(default = "default_chain_type")]
    pub chain_type: String,
    #[serde(default)]
    pub routes: Vec<crate::infrastructure::admin_config::ChainRouteEntry>,
    #[serde(default)]
    pub description: Option<String>,
}

#[web::post("/api/admin/chains")]
pub async fn upsert_chain(
    state: State<AdminState>,
    Query(_params): Query<HashMap<String, String>>,
    Json(body): Json<UpsertChainRequest>,
) -> HttpResponse {
    match state
        .config
        .upsert_chain(
            body.id,
            body.name,
            body.protocol,
            body.chain_type,
            body.routes,
            body.description,
        )
        .await
    {
        Ok(chain) => HttpResponse::Ok().json(&serde_json::json!({
            "msg": "ok",
            "data": chain
        })),
        Err(e) => HttpResponse::InternalServerError().json(&serde_json::json!({
            "msg": "error",
            "error": e
        })),
    }
}

#[web::delete("/api/admin/chains/{id}")]
pub async fn delete_chain(
    state: State<AdminState>,
    Query(_params): Query<HashMap<String, String>>,
    path: web::types::Path<i64>,
) -> HttpResponse {
    let id = path.into_inner();
    if let Err(e) = state.config.delete_chain(id).await {
        return HttpResponse::BadRequest().json(&serde_json::json!({
            "msg": "error",
            "error": e
        }));
    }

    let _ = chain_ops::cleanup_chain_artifacts(&state.config, state.node_transport.as_ref(), id).await;

    HttpResponse::Ok().json(&serde_json::json!({
        "msg": "ok"
    }))
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
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "inbound").await;
                let _ = transport.publish_update_notification(node_id, "config").await;
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
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "inbound").await;
                let _ = transport.publish_update_notification(node_id, "config").await;
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
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "inbound").await;
                let _ = transport.publish_update_notification(node_id, "config").await;
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

    let transport = match &state.node_transport {
        Some(c) => c,
        None => {
            return HttpResponse::ServiceUnavailable().json(&serde_json::json!({
                "msg": "error",
                "error": "MQTT 未连接，无法刷新网络接口信息"
            }))
        }
    };

    let result = transport.query_node_network_interfaces(node_id, 8).await;
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
pub struct CreateNodeRequest {
    pub node_id: u64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[web::post("/api/admin/nodes")]
pub async fn create_node(
    state: State<AdminState>,
    req: HttpRequest,
    Json(body): Json<CreateNodeRequest>,
) -> HttpResponse {
    if let Err(resp) = require_admin_token(&req) {
        return resp;
    }

    let node_id = body.node_id;

    info!("[admin] creating node node_id={}", node_id);

    let node_token = Uuid::new_v4().to_string();
    let node_shared_secret = Uuid::new_v4().to_string();

    // 检查节点是否已存在
    let existing_node = admin_node_config::Entity::find_by_id(node_id)
        .one(&state.db)
        .await;

    let existing_node = match existing_node {
        Ok(node) => node,
        Err(e) => {
            error!("[admin] db error when checking node node_id={}: {}", node_id, e);
            return HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": format!("数据库查询失败: {}", e)
            }));
        }
    };

    if existing_node.is_some() {
        return HttpResponse::BadRequest().json(&serde_json::json!({
            "msg": "error",
            "error": format!("node_id={} 已存在", node_id)
        }));
    }

    // 创建节点配置
    let active_node = admin_node_config::ActiveModel {
        node_id: Set(node_id),
        node_type: Set("VlessReality".to_string()),
        node_speed_limit: Set(0),
        traffic_rate: Set(1.0),
        sort: Set(1),
        name: Set(body.name),
        region: Set(body.region),
        description: Set(body.description),
        node_token: Set(Some(node_token.clone())),
        node_shared_secret: Set(Some(node_shared_secret.clone())),
        ..Default::default()
    };

    let inserted_node = active_node.insert(&state.db).await;

    let inserted_node = match inserted_node {
        Ok(node) => node,
        Err(e) => {
            error!("[admin] failed to create node config node_id={}: {}", node_id, e);
            return HttpResponse::InternalServerError().json(&serde_json::json!({
                "msg": "error",
                "error": format!("创建节点配置失败: {}", e)
            }));
        }
    };

    // 创建默认入站
    let active_inbound = admin_inbound::ActiveModel {
        node_id: Set(node_id),
        tag: Set("entrydoor".to_string()),
        protocol: Set("vless".to_string()),
        port: Set(10086 as i32),
        listen: Set(None),
        settings: Set(serde_json::json!({
            "decryption": "none"
        })),
        stream_settings: Set(Some(serde_json::json!({
            "network": "tcp",
            "security": "reality",
            "realitySettings": {
                "show": false,
                "dest": "www.cloudflare.com:443",
                "xver": 0,
                "serverNames": ["www.cloudflare.com"],
                "privateKey": "aNd7UkHbEak7xUDUAcUCycsbi8sjk71TfNB5ZOp0yUk",
                "shortIds": ["a66cafe7"],
                "serverName": "www.cloudflare.com",
                "publicKey": "JbPBpbjEHQiL87HpoJ6wZ3o9wSTyjzTIN9ysP6tnywU",
                "shortId": "a66cafe7",
                "fingerprint": "chrome",
                "spiderX": "/"
            }
        }))),
        sniffing: Set(None),
        ..Default::default()
    };

    let _ = active_inbound.insert(&state.db).await.map_err(|e| {
        error!("[admin] failed to create default inbound node_id={}: {}", node_id, e);
    });

    // 创建默认出站（block 和 direct）
    let default_outbounds = vec![
        (
            "block".to_string(),
            "blackhole".to_string(),
            serde_json::json!({
                "response": { "type": "http" }
            }),
        ),
        ("direct".to_string(), "freedom".to_string(), serde_json::json!({})),
    ];

    for (tag, protocol, settings) in default_outbounds {
        let active_outbound = admin_outbound::ActiveModel {
            node_id: Set(node_id),
            tag: Set(tag.clone()),
            protocol: Set(protocol),
            settings: Set(settings),
            send_through: Set(None),
            stream_settings: Set(None),
            ..Default::default()
        };

        let _ = active_outbound.insert(&state.db).await.map_err(|e| {
            error!("[admin] failed to create default outbound node_id={} tag={}: {}", node_id, tag, e);
        });
    }

    info!("[admin] node created successfully node_id={}", node_id);

    // 返回创建的节点信息
    let node_info = serde_json::json!({
        "node_id": inserted_node.node_id,
        "name": inserted_node.name,
        "region": inserted_node.region,
        "description": inserted_node.description,
        "node_type": inserted_node.node_type,
        "node_speed_limit": inserted_node.node_speed_limit,
        "traffic_rate": inserted_node.traffic_rate,
        "sort": inserted_node.sort,
        "maintenance_mode": inserted_node.maintenance_mode,
        "is_online": inserted_node.is_online,
        "last_seen_at": inserted_node.last_seen_at.map(|t| t.to_rfc3339()),
        "cpu_usage": inserted_node.cpu_usage,
        "mem_usage": inserted_node.mem_usage,
        "disk_usage": inserted_node.disk_usage,
        "uptime": inserted_node.uptime,
        "online_user_count": inserted_node.online_user_count,
        "cpu_threads": inserted_node.cpu_threads,
        "mem_total": inserted_node.mem_total,
        "disk_total": inserted_node.disk_total,
        "public_ip": inserted_node.public_ip,
        "network_interfaces": inserted_node.network_interfaces,
        "node_token": inserted_node.node_token,
        "node_shared_secret": inserted_node.node_shared_secret,
        "created_at": inserted_node.created_at.to_rfc3339(),
        "updated_at": inserted_node.updated_at.to_rfc3339(),
    });

    HttpResponse::Ok().json(&serde_json::json!({
        "msg": "ok",
        "data": node_info
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
            
            if let Some(ref transport) = state.node_transport {
                let result = transport.query_node_logs(node_id, query_params, 15).await;
                
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
    req: HttpRequest,
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<AddUserRequest>,
) -> HttpResponse {
    if let Err(resp) = require_admin_token(&req) {
        return resp;
    }
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let (sync, sync_timeout) = parse_sync_from_query(&params);
    info!(
        "[admin:add_user] node_id={} uuid={} st={} dt={} sync={} timeout={}s",
        node_id, body.uuid, body.st, body.dt, sync, sync_timeout
    );
    
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
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "user").await;

                if sync {
                    let wait_start = Instant::now();
                    info!(
                        "[admin:add_user] waiting node pull: node_id={} action=user timeout={}s",
                        node_id, sync_timeout
                    );
                    if let Err(e) = transport.wait_for_node_pull(node_id, "user", sync_timeout).await {
                        error!(
                            "[admin:add_user] node pull timeout node_id={} action=user elapsed={:?} err={}",
                            node_id,
                            wait_start.elapsed(),
                            e
                        );
                        let _ = state.config.delete_user(node_id, user.id).await;
                        let _ = transport.publish_update_notification(node_id, "user").await;
                        let _ = transport.publish_update_notification(node_id, "outbound").await;
                        let _ = transport.publish_update_notification(node_id, "inbound").await;
                        return HttpResponse::GatewayTimeout().json(&serde_json::json!({
                            "msg": "error",
                            "error": e
                        }));
                    }
                    info!(
                        "[admin:add_user] node pull confirmed: node_id={} action=user elapsed={:?}",
                        node_id,
                        wait_start.elapsed()
                    );
                }
            } else if sync {
                return HttpResponse::ServiceUnavailable().json(&serde_json::json!({
                    "msg": "error",
                    "error": "MQTT 未连接，无法启用 sync"
                }));
            }

            HttpResponse::Ok().json(&serde_json::json!({
                "msg": "ok",
                "data": user,
                "message": "用户添加成功"
            }))
        }
        Err(_e) => {
            if let Ok(users) = state.config.get_users(node_id).await {
                if let Some(user) = users.iter().find(|u| u.uuid == body.uuid).cloned() {
                    if let Some(ref transport) = state.node_transport {
                        let _ = transport.publish_update_notification(node_id, "user").await;
                        if sync {
                            let wait_start = Instant::now();
                            info!(
                                "[admin:add_user] waiting existing user pull: node_id={} uuid={} timeout={}s",
                                node_id, body.uuid, sync_timeout
                            );
                            if let Err(e) = transport.wait_for_node_pull(node_id, "user", sync_timeout).await {
                                error!(
                                    "[admin:add_user] node pull timeout for existing user: node_id={} elapsed={:?} err={}",
                                    node_id,
                                    wait_start.elapsed(),
                                    e
                                );
                                return HttpResponse::GatewayTimeout().json(&serde_json::json!({
                                    "msg": "error",
                                    "error": e
                                }));
                            }
                            info!(
                                "[admin:add_user] node pull confirmed for existing user: node_id={} elapsed={:?}",
                                node_id,
                                wait_start.elapsed()
                            );
                        }
                    } else if sync {
                        return HttpResponse::ServiceUnavailable().json(&serde_json::json!({
                            "msg": "error",
                            "error": "MQTT 未连接，无法启用 sync"
                        }));
                    }

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
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "user").await;
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
    req: HttpRequest,
    Query(params): Query<HashMap<String, String>>,
    path: web::types::Path<u64>,
) -> HttpResponse {
    if let Err(resp) = require_admin_token(&req) {
        return resp;
    }
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let id = path.into_inner();
    let (sync, sync_timeout) = parse_sync_from_query(&params);
    info!(
        "[admin:delete_user] node_id={} user_id={} sync={} timeout={}s",
        node_id, id, sync, sync_timeout
    );
    
    match state.config.delete_user(node_id, id).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "user").await;
                let _ = transport.publish_update_notification(node_id, "inbound").await;
                let _ = transport.publish_update_notification(node_id, "outbound").await;

                if sync {
                    let wait_start = Instant::now();
                    info!(
                        "[admin:delete_user] waiting node pull: node_id={} action=user timeout={}s",
                        node_id, sync_timeout
                    );
                    if let Err(e) = transport.wait_for_node_pull(node_id, "user", sync_timeout).await {
                        error!(
                            "[admin:delete_user] node pull timeout action=user node_id={} elapsed={:?} err={}",
                            node_id,
                            wait_start.elapsed(),
                            e
                        );
                        return HttpResponse::GatewayTimeout().json(&serde_json::json!({
                            "msg": "error",
                            "error": e
                        }));
                    }
                    info!(
                        "[admin:delete_user] node pull confirmed: node_id={} action=user elapsed={:?}",
                        node_id,
                        wait_start.elapsed()
                    );
                    let wait_start = Instant::now();
                    info!(
                        "[admin:delete_user] waiting node pull: node_id={} action=outbound timeout={}s",
                        node_id, sync_timeout
                    );
                    if let Err(e) = transport.wait_for_node_pull(node_id, "outbound", sync_timeout).await {
                        error!(
                            "[admin:delete_user] node pull timeout action=outbound node_id={} elapsed={:?} err={}",
                            node_id,
                            wait_start.elapsed(),
                            e
                        );
                        return HttpResponse::GatewayTimeout().json(&serde_json::json!({
                            "msg": "error",
                            "error": e
                        }));
                    }
                    info!(
                        "[admin:delete_user] node pull confirmed: node_id={} action=outbound elapsed={:?}",
                        node_id,
                        wait_start.elapsed()
                    );
                }
            } else if sync {
                return HttpResponse::ServiceUnavailable().json(&serde_json::json!({
                    "msg": "error",
                    "error": "MQTT 未连接，无法启用 sync"
                }));
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
    #[serde(rename = "sendThrough", default)]
    pub send_through: Option<String>,
    #[serde(rename = "streamSettings", default, alias = "stream_settings")]
    pub stream_settings: Option<Value>,
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
        send_through: body.send_through,
        stream_settings: body.stream_settings,
    };
    
    match state.config.add_outbound(node_id, outbound.clone()).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "outbound").await;
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
    #[serde(rename = "sendThrough", default)]
    pub send_through: Option<Option<String>>,
    #[serde(rename = "streamSettings", default, alias = "stream_settings")]
    pub stream_settings: Option<Option<Value>>,
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
    
    match state
        .config
        .update_outbound(
            node_id,
            &tag,
            body.protocol,
            body.settings,
            body.send_through,
            body.stream_settings,
        )
        .await
    {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "outbound").await;
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
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "outbound").await;
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
    
    if let Some(ref transport) = state.node_transport {
        let result = transport.query_udp_latency(
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
    if let Some(ref transport) = state.node_transport {
        let _ = transport.publish_update_notification(node_id, "routing").await;
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
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "routing").await;
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
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "routing").await;
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
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "routing").await;
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
    req: HttpRequest,
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<AddMappingRequest>,
) -> HttpResponse {
    if let Err(resp) = require_admin_token(&req) {
        return resp;
    }
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let (sync, sync_timeout) = parse_sync_from_query(&params);
    info!(
        "[admin:add_mapping] node_id={} uuid={} outbound_tag={} sync={} timeout={}s",
        node_id, body.uuid, body.outbound_tag, sync, sync_timeout
    );
    
    match state.config.add_mapping(node_id, body.uuid.clone(), body.outbound_tag.clone()).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "outbound").await;

                if sync {
                    let wait_start = Instant::now();
                    info!(
                        "[admin:add_mapping] waiting node pull: node_id={} action=outbound timeout={}s",
                        node_id, sync_timeout
                    );
                    if let Err(e) = transport.wait_for_node_pull(node_id, "outbound", sync_timeout).await {
                        error!(
                            "[admin:add_mapping] node pull timeout node_id={} action=outbound elapsed={:?} err={}",
                            node_id,
                            wait_start.elapsed(),
                            e
                        );
                        return HttpResponse::GatewayTimeout().json(&serde_json::json!({
                            "msg": "error",
                            "error": e
                        }));
                    }
                    info!(
                        "[admin:add_mapping] node pull confirmed: node_id={} action=outbound elapsed={:?}",
                        node_id,
                        wait_start.elapsed()
                    );
                }
            } else if sync {
                return HttpResponse::ServiceUnavailable().json(&serde_json::json!({
                    "msg": "error",
                    "error": "MQTT 未连接，无法启用 sync"
                }));
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
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "outbound").await;
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
    req: HttpRequest,
    Query(params): Query<HashMap<String, String>>,
    path: web::types::Path<String>,
) -> HttpResponse {
    if let Err(resp) = require_admin_token(&req) {
        return resp;
    }
    let node_id = match get_node_id_from_query(&params) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let uuid = path.into_inner();
    
    match state.config.delete_mapping(node_id, &uuid).await {
        Ok(_) => {
            // 推送更新通知
            if let Some(ref transport) = state.node_transport {
                let _ = transport.publish_update_notification(node_id, "outbound").await;
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
