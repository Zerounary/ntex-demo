//! 管理服务器配置存储模块
//! 
//! 用于存储用户、上游代理、路由配置等数据（数据库存储）

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set, JsonValue};
use crate::infrastructure::persistence::{
    admin_user, admin_outbound, admin_routing, admin_user_mapping, admin_node_config,
};

/// 用户配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub uuid: String,
    pub st: u64,  // 限速值（Mbps）
    pub dt: u64,  // 设备限制
}

/// 上游代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundConfig {
    pub tag: String,
    pub protocol: String,
    #[serde(default)]
    pub settings: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_settings: Option<Value>,
}

/// 路由规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    #[serde(rename = "type")]
    pub rule_type: String,
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

/// 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_id: u64,
    pub node_type: String,
    pub node_speed_limit: u64,
    pub traffic_rate: f64,
    pub sort: u64,
    pub inbounds: Vec<Value>,
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
            stream_settings: Set(outbound.stream_settings),
            ..Default::default()
        };
        
        active_model.insert(&self.db).await
            .map_err(|e| format!("添加上游代理失败: {}", e))?;
        
        Ok(())
    }

    pub async fn update_outbound(&self, node_id: u64, tag: &str, protocol: Option<String>, settings: Option<Value>) -> Result<(), String> {
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
    pub async fn get_node_config(&self, node_id: u64) -> Result<NodeConfig, String> {
        let node_config = admin_node_config::Entity::find_by_id(node_id)
            .one(&self.db)
            .await
            .map_err(|e| format!("查询节点配置失败: {}", e))?
            .ok_or_else(|| "节点配置不存在".to_string())?;
        
        let inbounds: Vec<Value> = serde_json::from_value(node_config.inbounds.clone())
            .map_err(|e| format!("解析节点入站配置失败: {}", e))?;
        
        Ok(NodeConfig {
            node_id: node_config.node_id,
            node_type: node_config.node_type,
            node_speed_limit: node_config.node_speed_limit,
            traffic_rate: node_config.traffic_rate,
            sort: node_config.sort,
            inbounds,
        })
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
}


