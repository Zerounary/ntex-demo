//! 管理服务器配置存储模块
//! 
//! 用于存储用户、上游代理、路由配置等数据（数据库存储）

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};
use sea_orm::JsonValue;
use sea_orm::NotSet;
use sea_orm::sea_query::Expr;
use crate::infrastructure::persistence::{
    admin_chain, admin_user, admin_outbound, admin_routing, admin_user_mapping, admin_node_config, admin_inbound,
    node_traffic_log, node_status_log, node_online_user_log, node_illegal_log,
    node_outbound_event_log, node_outbound_latency_log,
};
use log::{info, warn};

/// 用户配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub uuid: String,
    pub st: u64,  // 限速值（Mbps）
    pub dt: u64,  // 设备限制
}

/// 入站配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundConfig {
    pub tag: String,
    pub protocol: String,
    pub port: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen: Option<String>,
    #[serde(default)]
    pub settings: Value,
    #[serde(rename = "streamSettings", skip_serializing_if = "Option::is_none")]
    pub stream_settings: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sniffing: Option<Value>,
}

impl From<admin_inbound::Model> for InboundConfig {
    fn from(model: admin_inbound::Model) -> Self {
        InboundConfig {
            tag: model.tag,
            protocol: model.protocol,
            port: model.port,
            listen: model.listen,
            settings: model.settings,
            stream_settings: model.stream_settings,
            sniffing: model.sniffing,
        }
    }
}

/// 上游代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundConfig {
    pub tag: String,
    pub protocol: String,
    #[serde(default)]
    pub settings: Value,
    #[serde(rename = "sendThrough", skip_serializing_if = "Option::is_none")]
    pub send_through: Option<String>,
    #[serde(rename = "streamSettings", skip_serializing_if = "Option::is_none")]
    pub stream_settings: Option<Value>,
}

/// 路由规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    #[serde(rename = "type")]
    pub rule_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbound_tag: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outbound_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<Vec<String>>,
}

/// 路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub domain_strategy: String,
    pub rules: Vec<RoutingRule>,
}

/// 链路路由表行（用于链式链路维护）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainRouteEntry {
    pub id: String,
    pub order: u32,
    pub from_node_id: u64,
    pub to_node_id: u64,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
}

/// 链路定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainDefinition {
    pub id: i64,
    pub name: String,
    pub protocol: String,
    pub routes: Vec<ChainRouteEntry>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_id: u64,
    pub node_type: String,
    pub node_speed_limit: u64,
    pub traffic_rate: f64,
    pub sort: u64,
    pub inbounds: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_threads: Option<u32>,  // CPU 线程数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mem_total: Option<u64>,    // 内存总容量（字节）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_total: Option<u64>,   // 磁盘总容量（字节）
}

/// 管理配置存储
#[derive(Clone)]
pub struct AdminConfigStore {
    db: DatabaseConnection,
}

impl AdminConfigStore {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub async fn list_node_ids(&self) -> Result<Vec<u64>, String> {
        admin_node_config::Entity::find()
            .select_only()
            .column(admin_node_config::Column::NodeId)
            .into_tuple::<u64>()
            .all(&self.db)
            .await
            .map_err(|e| format!("查询节点列表失败: {}", e))
    }
    
    /// 解析百分比值（兼容字符串格式 "50%" 和数值格式 0.5）
    fn parse_percentage(value: Option<&Value>) -> Option<f64> {
        let v = value?;
        
        // 先尝试作为数值解析（0.0-1.0）
        if let Some(num) = v.as_f64() {
            return Some(num);
        }
        
        // 再尝试作为字符串解析（"50%"）
        if let Some(str_val) = v.as_str() {
            // 移除 % 符号并解析
            let cleaned = str_val.trim_end_matches('%').trim();
            if let Ok(num) = cleaned.parse::<f64>() {
                // 如果是百分比格式（0-100），转换为 0.0-1.0
                if num > 1.0 {
                    return Some(num / 100.0);
                } else {
                    return Some(num);
                }
            }
        }
        
        None
    }

    pub async fn get_node_public_ip(&self, node_id: u64) -> Result<String, String> {
        let node = admin_node_config::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
            .map_err(|e| format!("查询节点失败: {}", e))?
            .ok_or_else(|| "节点不存在".to_string())?;

        node.public_ip
            .ok_or_else(|| format!("node_id={} 未设置 public_ip", node_id))
    }

    pub async fn select_default_forward_port(&self, node_id: u64) -> Result<i32, String> {
        let inbounds = self.get_inbounds(node_id).await?;
        let mut candidates: Vec<i32> = inbounds
            .into_iter()
            .filter(|i| i.port > 0)
            .filter(|i| i.protocol != "dokodemo-door")
            .filter(|i| !i.tag.starts_with("chain_"))
            .map(|i| i.port)
            .collect();
        candidates.sort();
        candidates
            .into_iter()
            .next()
            .ok_or_else(|| format!("node_id={} 没有可用的末端入站端口用于转发", node_id))
    }

    pub async fn upsert_inbound(&self, node_id: u64, inbound: InboundConfig) -> Result<(), String> {
        let existing = admin_inbound::Entity::find()
            .filter(admin_inbound::Column::NodeId.eq(node_id))
            .filter(admin_inbound::Column::Tag.eq(&inbound.tag))
            .one(&self.db)
            .await
            .map_err(|e| format!("查询入站失败: {}", e))?;

        if let Some(model) = existing {
            let mut active_model: admin_inbound::ActiveModel = model.into();
            active_model.protocol = Set(inbound.protocol);
            active_model.port = Set(inbound.port);
            active_model.listen = Set(inbound.listen);
            active_model.settings = Set(inbound.settings);
            active_model.stream_settings = Set(inbound.stream_settings);
            active_model.sniffing = Set(inbound.sniffing);
            active_model.updated_at = Set(Utc::now().into());
            active_model
                .update(&self.db)
                .await
                .map_err(|e| format!("更新入站失败: {}", e))?;
            Ok(())
        } else {
            self.add_inbound(node_id, inbound).await
        }
    }

    // ========== 用户管理 ==========
    pub async fn get_users(&self, node_id: u64) -> Result<Vec<User>, String> {
        let users = admin_user::Entity::find()
            .filter(admin_user::Column::NodeId.eq(node_id))
            .all(&self.db)
            .await
            .map_err(|e| format!("查询用户失败: {}", e))?;
        
        Ok(users.into_iter().map(|u| User {
            id: u.id,
            uuid: u.uuid,
            st: u.st,
            dt: u.dt,
        }).collect())
    }

    pub async fn add_user(&self, node_id: u64, uuid: String, st: u64, dt: u64) -> Result<User, String> {
        // 检查用户是否已存在
        let existing = admin_user::Entity::find()
            .filter(admin_user::Column::NodeId.eq(node_id))
            .filter(admin_user::Column::Uuid.eq(&uuid))
            .one(&self.db)
            .await
            .map_err(|e| format!("查询用户失败: {}", e))?;
        
        if existing.is_some() {
            return Err("用户已存在".to_string());
        }
        
        let active_model = admin_user::ActiveModel {
            node_id: Set(node_id),
            uuid: Set(uuid.clone()),
            st: Set(st),
            dt: Set(dt),
            ..Default::default()
        };
        
        let result = active_model.insert(&self.db).await
            .map_err(|e| format!("添加用户失败: {}", e))?;
        
        Ok(User {
            id: result.id,
            uuid: result.uuid,
            st: result.st,
            dt: result.dt,
        })
    }

    pub async fn update_user(&self, node_id: u64, id: u64, uuid: Option<String>, st: Option<u64>, dt: Option<u64>) -> Result<(), String> {
        let user = admin_user::Entity::find_by_id(id)
            .filter(admin_user::Column::NodeId.eq(node_id))
            .one(&self.db)
            .await
            .map_err(|e| format!("查询用户失败: {}", e))?
            .ok_or_else(|| "用户不存在".to_string())?;
        
        let mut active_model: admin_user::ActiveModel = user.into();
        
        if let Some(uuid) = uuid {
            active_model.uuid = Set(uuid);
        }
        if let Some(st) = st {
            active_model.st = Set(st);
        }
        if let Some(dt) = dt {
            active_model.dt = Set(dt);
        }
        
        active_model.update(&self.db).await
            .map_err(|e| format!("更新用户失败: {}", e))?;
        
        Ok(())
    }

    pub async fn upsert_outbound(&self, node_id: u64, outbound: OutboundConfig) -> Result<(), String> {
        let existing = admin_outbound::Entity::find()
            .filter(admin_outbound::Column::NodeId.eq(node_id))
            .filter(admin_outbound::Column::Tag.eq(&outbound.tag))
            .one(&self.db)
            .await
            .map_err(|e| format!("查询上游代理失败: {}", e))?;

        if let Some(model) = existing {
            let mut active_model: admin_outbound::ActiveModel = model.into();
            active_model.protocol = Set(outbound.protocol);
            active_model.settings = Set(outbound.settings);
            active_model.send_through = Set(outbound.send_through);
            active_model.stream_settings = Set(outbound.stream_settings);
            active_model.updated_at = Set(Utc::now().into());
            active_model
                .update(&self.db)
                .await
                .map_err(|e| format!("更新上游代理失败: {}", e))?;
            Ok(())
        } else {
            self.add_outbound(node_id, outbound).await
        }
    }

    pub async fn update_node_network_interfaces(
        &self,
        node_id: u64,
        network_interfaces: JsonValue,
    ) -> Result<(), String> {
        if let Ok(Some(node_config)) = admin_node_config::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
        {
            let mut active_model: admin_node_config::ActiveModel = node_config.into();
            active_model.network_interfaces = Set(Some(network_interfaces));
            active_model.update(&self.db).await
                .map_err(|e| format!("更新节点网络接口信息失败: {}", e))?;
        }
        Ok(())
    }

    pub async fn delete_user(&self, node_id: u64, id: u64) -> Result<User, String> {
        let user = admin_user::Entity::find_by_id(id)
            .filter(admin_user::Column::NodeId.eq(node_id))
            .one(&self.db)
            .await
            .map_err(|e| format!("查询用户失败: {}", e))?
            .ok_or_else(|| "用户不存在".to_string())?;
        
        let uuid = user.uuid.clone();
        let user_result = User {
            id: user.id,
            uuid: user.uuid,
            st: user.st,
            dt: user.dt,
        };
        
        // 删除用户
        admin_user::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| format!("删除用户失败: {}", e))?;
        
        // 删除映射
        admin_user_mapping::Entity::delete_many()
            .filter(admin_user_mapping::Column::NodeId.eq(node_id))
            .filter(admin_user_mapping::Column::Uuid.eq(uuid))
            .exec(&self.db)
            .await
            .ok(); // 忽略映射删除错误，可能不存在
        
        Ok(user_result)
    }

    // ========== 入站管理 ==========
    pub async fn get_inbounds(&self, node_id: u64) -> Result<Vec<InboundConfig>, String> {
        let inbounds = admin_inbound::Entity::find()
            .filter(admin_inbound::Column::NodeId.eq(node_id))
            .order_by_asc(admin_inbound::Column::Id)
            .all(&self.db)
            .await
            .map_err(|e| format!("查询入站失败: {}", e))?;

        Ok(inbounds.into_iter().map(Into::into).collect())
    }

    pub async fn add_inbound(&self, node_id: u64, inbound: InboundConfig) -> Result<(), String> {
        let existing = admin_inbound::Entity::find()
            .filter(admin_inbound::Column::NodeId.eq(node_id))
            .filter(admin_inbound::Column::Tag.eq(&inbound.tag))
            .one(&self.db)
            .await
            .map_err(|e| format!("查询入站失败: {}", e))?;

        if existing.is_some() {
            return Err(format!("tag {} 已存在", inbound.tag));
        }

        let active_model = admin_inbound::ActiveModel {
            node_id: Set(node_id),
            tag: Set(inbound.tag),
            protocol: Set(inbound.protocol),
            port: Set(inbound.port),
            listen: Set(inbound.listen),
            settings: Set(inbound.settings),
            stream_settings: Set(inbound.stream_settings),
            sniffing: Set(inbound.sniffing),
            ..Default::default()
        };

        active_model
            .insert(&self.db)
            .await
            .map_err(|e| format!("添加入站失败: {}", e))?;

        Ok(())
    }

    pub async fn update_inbound(
        &self,
        node_id: u64,
        tag: &str,
        protocol: Option<String>,
        port: Option<i32>,
        listen: Option<Option<String>>,
        settings: Option<Value>,
        stream_settings: Option<Option<Value>>,
        sniffing: Option<Option<Value>>,
    ) -> Result<(), String> {
        let inbound = admin_inbound::Entity::find()
            .filter(admin_inbound::Column::NodeId.eq(node_id))
            .filter(admin_inbound::Column::Tag.eq(tag))
            .one(&self.db)
            .await
            .map_err(|e| format!("查询入站失败: {}", e))?
            .ok_or_else(|| "入站不存在".to_string())?;

        let mut active_model: admin_inbound::ActiveModel = inbound.into();

        if let Some(protocol) = protocol {
            active_model.protocol = Set(protocol);
        }
        if let Some(port) = port {
            active_model.port = Set(port);
        }
        if let Some(listen) = listen {
            active_model.listen = Set(listen);
        }
        if let Some(settings) = settings {
            active_model.settings = Set(settings);
        }
        if let Some(stream_settings) = stream_settings {
            active_model.stream_settings = Set(stream_settings);
        }
        if let Some(sniffing) = sniffing {
            active_model.sniffing = Set(sniffing);
        }

        active_model
            .update(&self.db)
            .await
            .map_err(|e| format!("更新入站失败: {}", e))?;

        Ok(())
    }

    pub async fn delete_inbound(&self, node_id: u64, tag: &str) -> Result<(), String> {
        let existing = admin_inbound::Entity::find()
            .filter(admin_inbound::Column::NodeId.eq(node_id))
            .filter(admin_inbound::Column::Tag.eq(tag))
            .one(&self.db)
            .await
            .map_err(|e| format!("查询入站失败: {}", e))?;

        if existing.is_none() {
            return Err("入站不存在".to_string());
        }

        admin_inbound::Entity::delete_many()
            .filter(admin_inbound::Column::NodeId.eq(node_id))
            .filter(admin_inbound::Column::Tag.eq(tag))
            .exec(&self.db)
            .await
            .map_err(|e| format!("删除入站失败: {}", e))?;

        Ok(())
    }

    // ========== 上游代理管理 ==========
    pub async fn get_outbounds(&self, node_id: u64) -> Result<(Vec<OutboundConfig>, HashMap<String, String>), String> {
        let outbounds = admin_outbound::Entity::find()
            .filter(admin_outbound::Column::NodeId.eq(node_id))
            .all(&self.db)
            .await
            .map_err(|e| format!("查询上游代理失败: {}", e))?;
        
        let outbounds_data: Vec<OutboundConfig> = outbounds.into_iter().map(|o| OutboundConfig {
            tag: o.tag,
            protocol: o.protocol,
            settings: o.settings,
            send_through: o.send_through,
            stream_settings: o.stream_settings,
        }).collect();
        
        let mappings = admin_user_mapping::Entity::find()
            .filter(admin_user_mapping::Column::NodeId.eq(node_id))
            .all(&self.db)
            .await
            .map_err(|e| format!("查询用户映射失败: {}", e))?;
        
        let mut user_mapping = HashMap::new();
        for m in mappings {
            user_mapping.insert(m.uuid, m.outbound_tag);
        }
        
        Ok((outbounds_data, user_mapping))
    }

    pub async fn add_outbound(&self, node_id: u64, outbound: OutboundConfig) -> Result<(), String> {
        // 检查 tag 是否已存在
        let existing = admin_outbound::Entity::find()
            .filter(admin_outbound::Column::NodeId.eq(node_id))
            .filter(admin_outbound::Column::Tag.eq(&outbound.tag))
            .one(&self.db)
            .await
            .map_err(|e| format!("查询上游代理失败: {}", e))?;
        
        if existing.is_some() {
            return Err(format!("tag {} 已存在", outbound.tag));
        }
        
        let active_model = admin_outbound::ActiveModel {
            node_id: Set(node_id),
            tag: Set(outbound.tag),
            protocol: Set(outbound.protocol),
            settings: Set(outbound.settings),
            send_through: Set(outbound.send_through),
            stream_settings: Set(outbound.stream_settings),
            ..Default::default()
        };
        
        active_model.insert(&self.db).await
            .map_err(|e| format!("添加上游代理失败: {}", e))?;
        
        Ok(())
    }

    pub async fn update_outbound(
        &self,
        node_id: u64,
        tag: &str,
        protocol: Option<String>,
        settings: Option<Value>,
        send_through: Option<Option<String>>,
        stream_settings: Option<Option<Value>>,
    ) -> Result<(), String> {
        let outbound = admin_outbound::Entity::find()
            .filter(admin_outbound::Column::NodeId.eq(node_id))
            .filter(admin_outbound::Column::Tag.eq(tag))
            .one(&self.db)
            .await
            .map_err(|e| format!("查询上游代理失败: {}", e))?
            .ok_or_else(|| "上游代理不存在".to_string())?;
        
        let mut active_model: admin_outbound::ActiveModel = outbound.into();
        
        if let Some(protocol) = protocol {
            active_model.protocol = Set(protocol);
        }
        if let Some(settings) = settings {
            active_model.settings = Set(settings);
        }
        if let Some(send_through) = send_through {
            active_model.send_through = Set(send_through);
        }
        if let Some(stream_settings) = stream_settings {
            active_model.stream_settings = Set(stream_settings);
        }

        active_model.update(&self.db).await
            .map_err(|e| format!("更新上游代理失败: {}", e))?;
        
        Ok(())
    }

    pub async fn delete_outbound(&self, node_id: u64, tag: &str) -> Result<(), String> {
        // 检查是否存在
        let existing = admin_outbound::Entity::find()
            .filter(admin_outbound::Column::NodeId.eq(node_id))
            .filter(admin_outbound::Column::Tag.eq(tag))
            .one(&self.db)
            .await
            .map_err(|e| format!("查询上游代理失败: {}", e))?;
        
        if existing.is_none() {
            return Err("上游代理不存在".to_string());
        }
        
        // 删除上游代理
        admin_outbound::Entity::delete_many()
            .filter(admin_outbound::Column::NodeId.eq(node_id))
            .filter(admin_outbound::Column::Tag.eq(tag))
            .exec(&self.db)
            .await
            .map_err(|e| format!("删除上游代理失败: {}", e))?;
        
        // 删除所有使用该 outbound 的映射
        admin_user_mapping::Entity::delete_many()
            .filter(admin_user_mapping::Column::NodeId.eq(node_id))
            .filter(admin_user_mapping::Column::OutboundTag.eq(tag))
            .exec(&self.db)
            .await
            .ok(); // 忽略映射删除错误
        
        Ok(())
    }

    // ========== 用户映射管理 ==========
    pub async fn add_mapping(&self, node_id: u64, uuid: String, outbound_tag: String) -> Result<(), String> {
        // 检查是否已存在
        let existing = admin_user_mapping::Entity::find()
            .filter(admin_user_mapping::Column::NodeId.eq(node_id))
            .filter(admin_user_mapping::Column::Uuid.eq(&uuid))
            .one(&self.db)
            .await
            .map_err(|e| format!("查询映射失败: {}", e))?;
        
        if existing.is_some() {
            // 如果已存在，更新它（忽略返回值中的旧标签）
            self.update_mapping(node_id, &uuid, outbound_tag).await?;
            return Ok(());
        }
        
        let active_model = admin_user_mapping::ActiveModel {
            node_id: Set(node_id),
            uuid: Set(uuid),
            outbound_tag: Set(outbound_tag),
            ..Default::default()
        };
        
        active_model.insert(&self.db).await
            .map_err(|e| format!("添加映射失败: {}", e))?;
        
        Ok(())
    }

    pub async fn update_mapping(&self, node_id: u64, uuid: &str, outbound_tag: String) -> Result<String, String> {
        let mapping = admin_user_mapping::Entity::find()
            .filter(admin_user_mapping::Column::NodeId.eq(node_id))
            .filter(admin_user_mapping::Column::Uuid.eq(uuid))
            .one(&self.db)
            .await
            .map_err(|e| format!("查询映射失败: {}", e))?
            .ok_or_else(|| "映射不存在".to_string())?;
        
        let old_tag = mapping.outbound_tag.clone();
        
        let mut active_model: admin_user_mapping::ActiveModel = mapping.into();
        active_model.outbound_tag = Set(outbound_tag);
        
        active_model.update(&self.db).await
            .map_err(|e| format!("更新映射失败: {}", e))?;
        
        Ok(old_tag)
    }

    pub async fn delete_mapping(&self, node_id: u64, uuid: &str) -> Result<(), String> {
        let result = admin_user_mapping::Entity::delete_many()
            .filter(admin_user_mapping::Column::NodeId.eq(node_id))
            .filter(admin_user_mapping::Column::Uuid.eq(uuid))
            .exec(&self.db)
            .await
            .map_err(|e| format!("删除映射失败: {}", e))?;
        
        if result.rows_affected == 0 {
            return Err("映射不存在".to_string());
        }
        
        Ok(())
    }

    // ========== 链路（Chain）配置管理 ==========
    pub async fn get_chains(&self) -> Result<Vec<ChainDefinition>, String> {
        let rows = admin_chain::Entity::find()
            .order_by_asc(admin_chain::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| format!("查询链路配置失败: {}", e))?;

        let mut out: Vec<ChainDefinition> = Vec::with_capacity(rows.len());
        for row in rows {
            let routes: Vec<ChainRouteEntry> = serde_json::from_value(row.routes)
                .map_err(|e| format!("解析链路 routes 失败: {}", e))?;
            out.push(ChainDefinition {
                id: row.id,
                name: row.name,
                protocol: row.protocol,
                routes,
                created_at: row.created_at.to_rfc3339(),
                updated_at: row.updated_at.to_rfc3339(),
                description: row.description,
            });
        }
        Ok(out)
    }

    pub async fn upsert_chain(
        &self,
        id: Option<i64>,
        name: String,
        protocol: String,
        routes: Vec<ChainRouteEntry>,
        description: Option<String>,
    ) -> Result<ChainDefinition, String> {
        let routes: JsonValue = serde_json::to_value(&routes)
            .map_err(|e| format!("序列化链路 routes 失败: {}", e))?;

        let now = Utc::now();
        if let Some(id) = id {
            let existing = admin_chain::Entity::find_by_id(id)
                .one(&self.db)
                .await
                .map_err(|e| format!("查询链路配置失败: {}", e))?
                .ok_or_else(|| "链路不存在".to_string())?;

            let created_at = existing.created_at;

            let mut active: admin_chain::ActiveModel = existing.into();
            active.name = Set(name);
            active.protocol = Set(protocol);
            active.routes = Set(routes);
            active.description = Set(description);
            active.updated_at = Set(now.into());

            let saved = active
                .update(&self.db)
                .await
                .map_err(|e| format!("更新链路配置失败: {}", e))?;

            let routes: Vec<ChainRouteEntry> = serde_json::from_value(saved.routes)
                .map_err(|e| format!("解析链路 routes 失败: {}", e))?;

            Ok(ChainDefinition {
                id: saved.id,
                name: saved.name,
                protocol: saved.protocol,
                routes,
                created_at: created_at.to_rfc3339(),
                updated_at: saved.updated_at.to_rfc3339(),
                description: saved.description,
            })
        } else {
            let active = admin_chain::ActiveModel {
                id: NotSet,
                name: Set(name),
                protocol: Set(protocol),
                routes: Set(routes),
                description: Set(description),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
            };

            let saved = active
                .insert(&self.db)
                .await
                .map_err(|e| format!("创建链路配置失败: {}", e))?;

            let routes: Vec<ChainRouteEntry> = serde_json::from_value(saved.routes)
                .map_err(|e| format!("解析链路 routes 失败: {}", e))?;

            Ok(ChainDefinition {
                id: saved.id,
                name: saved.name,
                protocol: saved.protocol,
                routes,
                created_at: saved.created_at.to_rfc3339(),
                updated_at: saved.updated_at.to_rfc3339(),
                description: saved.description,
            })
        }
    }

    pub async fn delete_chain(&self, id: i64) -> Result<(), String> {
        let result = admin_chain::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| format!("删除链路配置失败: {}", e))?;

        if result.rows_affected == 0 {
            return Err("链路不存在".to_string());
        }
        Ok(())
    }

    // ========== 路由配置管理 ==========
    pub async fn get_routing(&self, node_id: u64) -> Result<RoutingConfig, String> {
        let routing = admin_routing::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
            .map_err(|e| format!("查询路由配置失败: {}", e))?;
        
        if let Some(r) = routing {
            let rules: Vec<RoutingRule> = serde_json::from_value(r.rules.clone())
                .map_err(|e| format!("解析路由规则失败: {}", e))?;
            Ok(RoutingConfig {
                domain_strategy: r.domain_strategy,
                rules,
            })
        } else {
            // 返回默认配置
            Ok(RoutingConfig {
                domain_strategy: "AsIs".to_string(),
                rules: vec![],
            })
        }
    }

    pub async fn update_routing(&self, node_id: u64, domain_strategy: Option<String>, rules: Option<Vec<RoutingRule>>) -> Result<(), String> {
        let routing = admin_routing::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
            .map_err(|e| format!("查询路由配置失败: {}", e))?;
        
        if let Some(routing_model) = routing {
            let mut active_model: admin_routing::ActiveModel = routing_model.into();
            
            if let Some(domain_strategy) = domain_strategy {
                active_model.domain_strategy = Set(domain_strategy);
            }
            if let Some(rules) = rules {
                let rules_json: JsonValue = serde_json::to_value(rules)
                    .map_err(|e| format!("序列化路由规则失败: {}", e))?;
                active_model.rules = Set(rules_json);
            }
            
            active_model.update(&self.db).await
                .map_err(|e| format!("更新路由配置失败: {}", e))?;
        } else {
            // 创建新配置
            let domain_strategy = domain_strategy.unwrap_or_else(|| "AsIs".to_string());
            let rules = rules.unwrap_or_default();
            let rules_json: JsonValue = serde_json::to_value(rules)
                .map_err(|e| format!("序列化路由规则失败: {}", e))?;
            
            let active_model = admin_routing::ActiveModel {
                node_id: Set(node_id),
                domain_strategy: Set(domain_strategy),
                rules: Set(rules_json),
                ..Default::default()
            };
            
            active_model.insert(&self.db).await
                .map_err(|e| format!("创建路由配置失败: {}", e))?;
        }
        
        Ok(())
    }

    pub async fn add_routing_rule(&self, node_id: u64, rule: RoutingRule) -> Result<(), String> {
        let mut routing_config = self.get_routing(node_id).await?;
        routing_config.rules.push(rule);
        self.update_routing(node_id, None, Some(routing_config.rules)).await
    }

    pub async fn update_routing_rule(&self, node_id: u64, index: usize, rule: RoutingRule) -> Result<(), String> {
        let mut routing_config = self.get_routing(node_id).await?;
        if index < routing_config.rules.len() {
            routing_config.rules[index] = rule;
            self.update_routing(node_id, None, Some(routing_config.rules)).await
        } else {
            Err("规则索引超出范围".to_string())
        }
    }

    pub async fn delete_routing_rule(&self, node_id: u64, index: usize) -> Result<RoutingRule, String> {
        let mut routing_config = self.get_routing(node_id).await?;
        if index < routing_config.rules.len() {
            let rule = routing_config.rules.remove(index);
            self.update_routing(node_id, None, Some(routing_config.rules)).await?;
            Ok(rule)
        } else {
            Err("规则索引超出范围".to_string())
        }
    }

    // ========== 节点配置 ==========
    
    /// 更新节点硬件信息（从 config 请求中提取）
    pub async fn update_node_hardware_info(&self, node_id: u64, request_data: &Value) -> Result<(), String> {
        // 解析硬件信息（如果提供）
        let cpu_threads = request_data.get("cpu_threads").and_then(|v| v.as_u64()).map(|v| v as u32);
        let mem_total = request_data.get("mem_total").and_then(|v| v.as_u64());
        let disk_total = request_data.get("disk_total").and_then(|v| v.as_u64());
        let public_ip = request_data.get("public_ip").and_then(|v| v.as_str()).map(|v| v.to_string());
        
        // 如果没有任何硬件信息，直接返回
        if cpu_threads.is_none() && mem_total.is_none() && disk_total.is_none() && public_ip.is_none() {
            return Ok(());
        }
        
        // 更新节点配置表的硬件信息
        if let Ok(Some(node_config)) = admin_node_config::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
        {
            let mut active_model: admin_node_config::ActiveModel = node_config.into();
            
            // 更新硬件信息（如果提供）
            if let Some(cpu_threads) = cpu_threads {
                active_model.cpu_threads = Set(Some(cpu_threads));
            }
            if let Some(mem_total) = mem_total {
                active_model.mem_total = Set(Some(mem_total));
            }
            if let Some(disk_total) = disk_total {
                active_model.disk_total = Set(Some(disk_total));
            }
            if let Some(public_ip) = public_ip {
                active_model.public_ip = Set(Some(public_ip));
            }
            
            active_model.update(&self.db).await
                .map_err(|e| format!("更新节点硬件信息失败: {}", e))?;
        }
        
        Ok(())
    }
    
    /// 为指定节点创建默认配置（含默认入站与默认上游代理）
    async fn create_default_node_config(&self, node_id: u64) -> Result<admin_node_config::Model, String> {
        let active = admin_node_config::ActiveModel {
            node_id: Set(node_id),
            node_type: Set("VlessReality".to_string()),
            node_speed_limit: Set(0),
            traffic_rate: Set(1.0),
            sort: Set(1),
            ..Default::default()
        };
        
        let inserted_config = active.insert(&self.db).await
            .map_err(|e| format!("创建默认节点配置失败: {}", e))?;

        admin_inbound::ActiveModel {
            node_id: Set(node_id),
            tag: Set("entrydoor".to_string()),
            protocol: Set("vless".to_string()),
            port: Set(10086),
            listen: Set(None),
            settings: Set(json!({
                "decryption": "none"
            })),
            stream_settings: Set(Some(json!({
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
        }
        .insert(&self.db)
        .await
        .map_err(|e| format!("创建默认入站失败: {}", e))?;

        // 初始化默认上游代理：block 与 direct
        let default_outbounds = vec![
            (
                "block".to_string(),
                "blackhole".to_string(),
                json!({
                    "response": { "type": "http" }
                }),
            ),
            ("direct".to_string(), "freedom".to_string(), json!({})),
        ];

        for (tag, protocol, settings) in default_outbounds {
            admin_outbound::ActiveModel {
                node_id: Set(node_id),
                tag: Set(tag),
                protocol: Set(protocol),
                settings: Set(settings),
                send_through: Set(None),
                stream_settings: Set(None),
                ..Default::default()
            }
            .insert(&self.db)
            .await
            .map_err(|e| format!("创建默认上游代理失败: {}", e))?;
        }

        Ok(inserted_config)
    }

    pub async fn get_node_config(&self, node_id: u64) -> Result<NodeConfig, String> {
        let node_config = admin_node_config::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
            .map_err(|e| format!("查询节点配置失败: {}", e))?;

        let node_config = if let Some(config) = node_config {
            config
        } else {
            self.create_default_node_config(node_id).await?
        };
        
        let inbounds_from_table = admin_inbound::Entity::find()
            .filter(admin_inbound::Column::NodeId.eq(node_id))
            .order_by_asc(admin_inbound::Column::Id)
            .all(&self.db)
            .await
            .map_err(|e| format!("查询入站失败: {}", e))?;

        let inbounds: Vec<Value> = inbounds_from_table
            .into_iter()
            .map(|m| {
                serde_json::json!({
                    "tag": m.tag,
                    "port": m.port,
                    "protocol": m.protocol,
                    "listen": m.listen,
                    "settings": m.settings,
                    "streamSettings": m.stream_settings,
                    "sniffing": m.sniffing
                })
            })
            .collect();
        
        Ok(NodeConfig {
            node_id: node_config.node_id,
            node_type: node_config.node_type,
            node_speed_limit: node_config.node_speed_limit,
            traffic_rate: node_config.traffic_rate,
            sort: node_config.sort,
            inbounds,
            cpu_threads: node_config.cpu_threads,
            mem_total: node_config.mem_total,
            disk_total: node_config.disk_total,
        })
    }

    pub async fn list_nodes(&self) -> Result<Vec<serde_json::Value>, String> {
        let node_configs = admin_node_config::Entity::find()
            .order_by_asc(admin_node_config::Column::Sort)
            .all(&self.db)
            .await
            .map_err(|e| format!("查询节点列表失败: {}", e))?;
        
        let mut result = Vec::new();
        for node_config in node_configs {
            result.push(serde_json::json!({
                "node_id": node_config.node_id,
                "name": node_config.name,
                "region": node_config.region,
                "description": node_config.description,
                "node_type": node_config.node_type,
                "node_speed_limit": node_config.node_speed_limit,
                "traffic_rate": node_config.traffic_rate,
                "sort": node_config.sort,
                "maintenance_mode": node_config.maintenance_mode,
                "is_online": node_config.is_online,
                "last_seen_at": node_config.last_seen_at.map(|t| t.to_rfc3339()),
                "cpu_usage": node_config.cpu_usage,
                "mem_usage": node_config.mem_usage,
                "disk_usage": node_config.disk_usage,
                "uptime": node_config.uptime,
                "online_user_count": node_config.online_user_count,
                "cpu_threads": node_config.cpu_threads,
                "mem_total": node_config.mem_total,
                "disk_total": node_config.disk_total,
                "public_ip": node_config.public_ip,
                "network_interfaces": node_config.network_interfaces,
                "created_at": node_config.created_at.to_rfc3339(),
                "updated_at": node_config.updated_at.to_rfc3339(),
            }));
        }
        
        Ok(result)
    }

    pub async fn set_node_online_status(&self, node_id: u64, is_online: bool) -> Result<(), String> {
        let node_config = admin_node_config::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
            .map_err(|e| format!("查询节点配置失败: {}", e))?
            .ok_or_else(|| "节点配置不存在".to_string())?;

        let mut active_model: admin_node_config::ActiveModel = node_config.into();
        active_model.is_online = Set(is_online);
        if is_online {
            active_model.last_seen_at = Set(Some(chrono::Utc::now()));
        }

        active_model
            .update(&self.db)
            .await
            .map_err(|e| format!("更新节点在线状态失败: {}", e))?;

        Ok(())
    }

    pub async fn set_all_nodes_offline(&self) -> Result<u64, String> {
        let res = admin_node_config::Entity::update_many()
            .col_expr(admin_node_config::Column::IsOnline, Expr::value(false))
            .exec(&self.db)
            .await
            .map_err(|e| format!("重置节点在线状态失败: {}", e))?;

        Ok(res.rows_affected)
    }

    pub async fn set_node_public_ip_if_empty(&self, node_id: u64, public_ip: String) -> Result<(), String> {
        let public_ip = public_ip.trim().to_string();
        if public_ip.is_empty() {
            return Ok(());
        }

        let node_config = admin_node_config::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
            .map_err(|e| format!("查询节点配置失败: {}", e))?
            .ok_or_else(|| "节点配置不存在".to_string())?;

        if node_config.public_ip.as_deref().unwrap_or("").trim().is_empty() {
            let mut active_model: admin_node_config::ActiveModel = node_config.into();
            active_model.public_ip = Set(Some(public_ip));
            active_model
                .update(&self.db)
                .await
                .map_err(|e| format!("更新节点 public_ip 失败: {}", e))?;
        }

        Ok(())
    }

    pub async fn update_node_meta(
        &self,
        node_id: u64,
        name: Option<Option<String>>,
        region: Option<Option<String>>,
        description: Option<Option<String>>,
    ) -> Result<(), String> {
        let node_config = admin_node_config::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
            .map_err(|e| format!("查询节点配置失败: {}", e))?
            .ok_or_else(|| "节点配置不存在".to_string())?;

        let mut active_model: admin_node_config::ActiveModel = node_config.into();
        if let Some(name) = name {
            active_model.name = Set(name);
        }
        if let Some(region) = region {
            active_model.region = Set(region);
        }
        if let Some(description) = description {
            active_model.description = Set(description);
        }

        active_model
            .update(&self.db)
            .await
            .map_err(|e| format!("更新节点信息失败: {}", e))?;

        Ok(())
    }

    // ========== 维护模式 ==========
    pub async fn get_maintenance_mode(&self, node_id: u64) -> Result<bool, String> {
        let node_config = admin_node_config::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
            .map_err(|e| format!("查询节点配置失败: {}", e))?
            .ok_or_else(|| "节点配置不存在".to_string())?;
        
        Ok(node_config.maintenance_mode)
    }

    pub async fn set_maintenance_mode(&self, node_id: u64, enabled: bool) -> Result<bool, String> {
        let node_config = admin_node_config::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
            .map_err(|e| format!("查询节点配置失败: {}", e))?
            .ok_or_else(|| "节点配置不存在".to_string())?;
        
        let old = node_config.maintenance_mode;
        
        let mut active_model: admin_node_config::ActiveModel = node_config.into();
        active_model.maintenance_mode = Set(enabled);
        
        active_model.update(&self.db).await
            .map_err(|e| format!("更新维护模式失败: {}", e))?;
        
        Ok(old)
    }

    // ========== 节点上报数据处理 ==========
    
    /// 处理流量上报
    pub async fn handle_traffic_report(&self, node_id: u64, data_array: &Vec<Value>) -> Result<(), String> {
        let now = Utc::now();
        let _ = admin_node_config::Entity::update_many()
            .col_expr(admin_node_config::Column::IsOnline, Expr::value(true))
            .col_expr(admin_node_config::Column::LastSeenAt, Expr::value(now))
            .filter(admin_node_config::Column::NodeId.eq(node_id))
            .exec(&self.db)
            .await;

        for item in data_array {
            let user_id = item.get("uid").and_then(|v| v.as_u64()).unwrap_or(0);
            let upload = item
                .get("upload")
                .or_else(|| item.get("u"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let download = item
                .get("download")
                .or_else(|| item.get("d"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            
            if user_id == 0 {
                continue;
            }
            
            // 检查上一条记录是否有变化
            let should_insert = if let Ok(Some(last_log)) = node_traffic_log::Entity::find()
                .filter(node_traffic_log::Column::NodeId.eq(node_id))
                .filter(node_traffic_log::Column::UserId.eq(user_id))
                .order_by_desc(node_traffic_log::Column::CreatedAt)
                .limit(1)
                .one(&self.db)
                .await
            {
                // 如果上一条记录的 upload 和 download 相同，则不插入
                last_log.upload != upload || last_log.download != download
            } else {
                // 没有上一条记录，需要插入
                true
            };
            
            if !should_insert {
                continue;
            }
            
            let log = node_traffic_log::ActiveModel {
                node_id: Set(node_id),
                user_id: Set(user_id),
                upload: Set(upload),
                download: Set(download),
                ..Default::default()
            };
            
            node_traffic_log::Entity::insert(log).exec(&self.db).await
                .map_err(|e| format!("存储流量上报失败: {}", e))?;
        }
        Ok(())
    }
    
    /// 处理节点状态上报
    pub async fn handle_node_status_report(&self, node_id: u64, status_data: &Value) -> Result<(), String> {
        let now_seen = Utc::now();
        let _ = admin_node_config::Entity::update_many()
            .col_expr(admin_node_config::Column::IsOnline, Expr::value(true))
            .col_expr(admin_node_config::Column::LastSeenAt, Expr::value(now_seen))
            .filter(admin_node_config::Column::NodeId.eq(node_id))
            .exec(&self.db)
            .await;

        // 解析使用率（兼容字符串格式 "50%" 和数值格式 0.5）
        let cpu = Self::parse_percentage(status_data.get("cpu")).unwrap_or(0.0).clamp(0.0, 1.0);
        let mem = Self::parse_percentage(status_data.get("mem")).unwrap_or(0.0).clamp(0.0, 1.0);
        let disk = Self::parse_percentage(status_data.get("disk")).unwrap_or(0.0).clamp(0.0, 1.0);
        let uptime = status_data.get("uptime").and_then(|v| v.as_u64()).unwrap_or(0);


        // 可选硬件信息（节点可能在 nodestatus 中上报）
        let cpu_threads = status_data.get("cpu_threads").and_then(|v| v.as_u64()).map(|v| v as u32);
        let mem_total = status_data.get("mem_total").and_then(|v| v.as_u64());
        let disk_total = status_data.get("disk_total").and_then(|v| v.as_u64());
        let public_ip = status_data.get("public_ip").and_then(|v| v.as_str()).map(|v| v.to_string());
        
        // 解析网络接口信息（如果提供）
        let network_interfaces = status_data.get("network")
            .map(|v| JsonValue::from(v.clone()));
        
        // 更新节点配置表的实时状态
        if let Ok(Some(node_config)) = admin_node_config::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
        {
            let mut active_model: admin_node_config::ActiveModel = node_config.into();
            active_model.cpu_usage = Set(Some(cpu));
            active_model.mem_usage = Set(Some(mem));
            active_model.disk_usage = Set(Some(disk));
            active_model.uptime = Set(Some(uptime));


            // 更新硬件信息（如果提供）
            if let Some(cpu_threads) = cpu_threads {
                active_model.cpu_threads = Set(Some(cpu_threads));
            }
            if let Some(mem_total) = mem_total {
                active_model.mem_total = Set(Some(mem_total));
            }
            if let Some(disk_total) = disk_total {
                active_model.disk_total = Set(Some(disk_total));
            }
            if let Some(public_ip) = public_ip {
                active_model.public_ip = Set(Some(public_ip));
            }
            
            // 更新网络接口信息（如果提供）
            if let Some(network_interfaces) = network_interfaces {
                active_model.network_interfaces = Set(Some(network_interfaces));
            }
            
            active_model.update(&self.db).await
                .map_err(|e| format!("更新节点实时状态失败: {}", e))?;
        }
        
        // 检查上一条记录是否有变化
        let should_insert = if let Ok(Some(last_log)) = node_status_log::Entity::find()
            .filter(node_status_log::Column::NodeId.eq(node_id))
            .order_by_desc(node_status_log::Column::CreatedAt)
            .limit(1)
            .one(&self.db)
            .await
        {
            // 如果所有字段都相同，则不插入（使用浮点数比较，允许小的误差）
            (last_log.cpu - cpu).abs() > 0.001 
                || (last_log.mem - mem).abs() > 0.001 
                || (last_log.disk - disk).abs() > 0.001 
                || last_log.uptime != uptime
        } else {
            // 没有上一条记录，需要插入
            true
        };
        
        if !should_insert {
            return Ok(());
        }
        
        // 插入历史记录
        let log = node_status_log::ActiveModel {
            node_id: Set(node_id),
            cpu: Set(cpu),
            mem: Set(mem),
            disk: Set(disk),
            uptime: Set(uptime),
            ..Default::default()
        };
        
        node_status_log::Entity::insert(log).exec(&self.db).await
            .map_err(|e| format!("存储节点状态历史记录失败: {}", e))?;
        
        Ok(())
    }
    
    /// 处理在线用户上报
    pub async fn handle_online_users_report(&self, node_id: u64, users_array: &Vec<Value>) -> Result<(), String> {
        let mut online_count = 0;
        let now = Utc::now();

        let _ = admin_node_config::Entity::update_many()
            .col_expr(admin_node_config::Column::IsOnline, Expr::value(true))
            .col_expr(admin_node_config::Column::LastSeenAt, Expr::value(now))
            .filter(admin_node_config::Column::NodeId.eq(node_id))
            .exec(&self.db)
            .await;

        let mut refreshed_sessions: u64 = 0;
        
        for item in users_array {
            let user_id = item.get("uid").and_then(|v| v.as_u64()).unwrap_or(0);
            let user_ip = item.get("ip").and_then(|v| v.as_str()).unwrap_or("").to_string();
            
            if user_id == 0 {
                continue;
            }
            
            online_count += 1;

            let updated = match crate::infrastructure::persistence::acceleration_session::Entity::update_many()
                .col_expr(
                    crate::infrastructure::persistence::acceleration_session::Column::LastActivityAt,
                    Expr::value(now),
                )
                .col_expr(
                    crate::infrastructure::persistence::acceleration_session::Column::UpdatedAt,
                    Expr::value(now),
                )
                .filter(crate::infrastructure::persistence::acceleration_session::Column::NodeId.eq(node_id))
                .filter(crate::infrastructure::persistence::acceleration_session::Column::AdminUserId.eq(user_id))
                .filter(crate::infrastructure::persistence::acceleration_session::Column::Status.eq("active"))
                .filter(crate::infrastructure::persistence::acceleration_session::Column::BillType.eq("minute"))
                .exec(&self.db)
                .await
            {
                Ok(r) => r.rows_affected,
                Err(e) => {
                    warn!(
                        "[onlineusers] refresh session activity failed: node_id={}, admin_user_id={}, err={}",
                        node_id, user_id, e
                    );
                    0
                }
            };
            refreshed_sessions += updated;
            
            // 检查上一条记录是否有变化（比较 user_id 和 user_ip）
            let should_insert = if let Ok(Some(last_log)) = node_online_user_log::Entity::find()
                .filter(node_online_user_log::Column::NodeId.eq(node_id))
                .filter(node_online_user_log::Column::UserId.eq(user_id))
                .order_by_desc(node_online_user_log::Column::CreatedAt)
                .limit(1)
                .one(&self.db)
                .await
            {
                // 如果 user_ip 不同，则需要插入
                last_log.user_ip != user_ip
            } else {
                // 没有上一条记录，需要插入
                true
            };
            
            if !should_insert {
                continue;
            }
            
            let log = node_online_user_log::ActiveModel {
                node_id: Set(node_id),
                user_id: Set(user_id),
                user_ip: Set(user_ip),
                ..Default::default()
            };
            
            node_online_user_log::Entity::insert(log).exec(&self.db).await
                .map_err(|e| format!("存储在线用户记录失败: {}", e))?;
        }

        if refreshed_sessions > 0 {
            info!(
                "[onlineusers] heartbeat refreshed: node_id={} snapshot_users={} refreshed_sessions={}",
                node_id,
                users_array.len(),
                refreshed_sessions
            );
        }
        
        // 更新节点配置表的在线用户数
        if let Ok(Some(node_config)) = admin_node_config::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
        {
            let mut active_model: admin_node_config::ActiveModel = node_config.into();
            active_model.online_user_count = Set(Some(online_count));
            
            active_model.update(&self.db).await
                .map_err(|e| format!("更新在线用户数失败: {}", e))?;
        }
        
        Ok(())
    }
    
    /// 处理非法行为上报
    pub async fn handle_illegal_report(&self, node_id: u64, illegal_array: &Vec<Value>) -> Result<(), String> {
        for item in illegal_array {
            let user_id = item.get("uid").and_then(|v| v.as_u64()).unwrap_or(0);
            
            if user_id == 0 {
                continue;
            }
            
            let log = node_illegal_log::ActiveModel {
                node_id: Set(node_id),
                user_id: Set(user_id),
                ..Default::default()
            };
            
            node_illegal_log::Entity::insert(log).exec(&self.db).await
                .map_err(|e| format!("存储非法行为记录失败: {}", e))?;
        }
        Ok(())
    }
    
    /// 处理 Outbound 事件（失败/恢复）
    pub async fn handle_outbound_event(&self, node_id: u64, event_type: &str, event_data: &Value) -> Result<(), String> {
        let outbound_tag = event_data.get("outbound_tag")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        if outbound_tag.is_empty() {
            return Ok(());
        }
        
        let error_message = event_data.get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let config = event_data.get("config")
            .map(|v| JsonValue::from(v.clone()));
        
        let log = node_outbound_event_log::ActiveModel {
            node_id: Set(node_id),
            outbound_tag: Set(outbound_tag),
            event_type: Set(event_type.to_string()),
            error_message: Set(error_message),
            config: Set(config),
            ..Default::default()
        };
        
        node_outbound_event_log::Entity::insert(log).exec(&self.db).await
            .map_err(|e| format!("存储 Outbound 事件失败: {}", e))?;
        
        Ok(())
    }
    
    /// 处理 Outbound 延迟上报
    pub async fn handle_outbound_latency(&self, node_id: u64, latency_data: &Value, probe_type: &str) -> Result<(), String> {
        let outbound_tag = latency_data.get("outbound_tag")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        if outbound_tag.is_empty() {
            return Ok(());
        }
        
        let latency_ms = latency_data.get("latency_ms")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let error_message = latency_data.get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // 检查上一条记录是否有变化（比较 outbound_tag, probe_type, latency_ms, error_message）
        let should_insert = if let Ok(Some(last_log)) = node_outbound_latency_log::Entity::find()
            .filter(node_outbound_latency_log::Column::NodeId.eq(node_id))
            .filter(node_outbound_latency_log::Column::OutboundTag.eq(&outbound_tag))
            .filter(node_outbound_latency_log::Column::ProbeType.eq(probe_type))
            .order_by_desc(node_outbound_latency_log::Column::CreatedAt)
            .limit(1)
            .one(&self.db)
            .await
        {
            // 如果 latency_ms 或 error_message 不同，则需要插入
            // 使用浮点数比较，允许小的误差（0.1ms）
            (last_log.latency_ms - latency_ms).abs() > 0.1 || last_log.error_message != error_message
        } else {
            // 没有上一条记录，需要插入
            true
        };
        
        if !should_insert {
            return Ok(());
        }
        
        let log = node_outbound_latency_log::ActiveModel {
            node_id: Set(node_id),
            outbound_tag: Set(outbound_tag),
            latency_ms: Set(latency_ms),
            probe_type: Set(probe_type.to_string()),
            error_message: Set(error_message),
            ..Default::default()
        };
        
        node_outbound_latency_log::Entity::insert(log).exec(&self.db).await
            .map_err(|e| format!("存储 Outbound 延迟记录失败: {}", e))?;
        
        Ok(())
    }
}


