//! 管理服务器配置存储模块
//! 
//! 用于存储用户、上游代理、路由配置等数据（内存存储）

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    users: Arc<RwLock<Vec<User>>>,
    outbounds: Arc<RwLock<Vec<OutboundConfig>>>,
    user_mapping: Arc<RwLock<HashMap<String, String>>>,  // UUID -> outbound_tag
    routing: Arc<RwLock<RoutingConfig>>,
    node_config: Arc<RwLock<NodeConfig>>,
    maintenance_mode: Arc<RwLock<bool>>,
}

impl AdminConfigStore {
    pub fn new() -> Self {
        // 初始化默认配置
        let users = vec![
            User {
                id: 1,
                uuid: "a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(),
                st: 5,
                dt: 0,
            },
            User {
                id: 2,
                uuid: "b2c3d4e5-f6a7-8901-bcde-f12345678901".to_string(),
                st: 1,
                dt: 0,
            },
            User {
                id: 3,
                uuid: "c3d4e5f6-a7b8-9012-cdef-123456789012".to_string(),
                st: 1,
                dt: 0,
            },
            User {
                id: 4,
                uuid: "d4e5f6a7-b8c9-0123-def0-234567890123".to_string(),
                st: 1,
                dt: 0,
            },
            User {
                id: 5,
                uuid: "e5f6a7b8-c9d0-1234-ef01-345678901234".to_string(),
                st: 1,
                dt: 0,
            },
            User {
                id: 6,
                uuid: "f6a7b8c9-d0e1-2345-f012-456789012345".to_string(),
                st: 1,
                dt: 0,
            },
        ];

        let mut user_mapping = HashMap::new();
        user_mapping.insert("a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(), "ss_1".to_string());
        user_mapping.insert("b2c3d4e5-f6a7-8901-bcde-f12345678901".to_string(), "ss_2".to_string());
        user_mapping.insert("c3d4e5f6-a7b8-9012-cdef-123456789012".to_string(), "ss_3".to_string());
        user_mapping.insert("d4e5f6a7-b8c9-0123-def0-234567890123".to_string(), "ss_1".to_string());
        user_mapping.insert("e5f6a7b8-c9d0-1234-ef01-345678901234".to_string(), "ss_2".to_string());

        let outbounds = vec![
            OutboundConfig {
                tag: "block".to_string(),
                protocol: "blackhole".to_string(),
                settings: serde_json::json!({
                    "response": {
                        "type": "http"
                    }
                }),
                stream_settings: None,
            },
            OutboundConfig {
                tag: "direct".to_string(),
                protocol: "freedom".to_string(),
                settings: serde_json::json!({}),
                stream_settings: None,
            },
            OutboundConfig {
                tag: "ss_1".to_string(),
                protocol: "shadowsocks".to_string(),
                settings: serde_json::json!({
                    "servers": [{
                        "address": "67.209.176.181",
                        "port": 19166,
                        "method": "aes-256-gcm",
                        "password": "bxaeWJ4Kf9ZL59R3"
                    }]
                }),
                stream_settings: None,
            },
            OutboundConfig {
                tag: "ss_2".to_string(),
                protocol: "shadowsocks".to_string(),
                settings: serde_json::json!({
                    "servers": [{
                        "address": "65.49.212.165",
                        "port": 19166,
                        "method": "aes-256-gcm",
                        "password": "bxaeWJ4Kf9ZL59R3"
                    }]
                }),
                stream_settings: None,
            },
            OutboundConfig {
                tag: "ss_3".to_string(),
                protocol: "shadowsocks".to_string(),
                settings: serde_json::json!({
                    "servers": [{
                        "address": "65.49.212.165",
                        "port": 19166,
                        "method": "aes-256-gcm",
                        "password": "bxaeWJ4Kf9ZL59R3"
                    }]
                }),
                stream_settings: None,
            },
            OutboundConfig {
                tag: "vmess_loopback".to_string(),
                protocol: "vmess".to_string(),
                settings: serde_json::json!({
                    "vnext": [{
                        "address": "127.0.0.1",
                        "port": 10086,
                        "users": [{
                            "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                            "alterId": 0,
                            "email": "t@t.tt",
                            "security": "auto"
                        }]
                    }]
                }),
                stream_settings: Some(serde_json::json!({
                    "network": "tcp"
                })),
            },
        ];

        let routing = RoutingConfig {
            domain_strategy: "AsIs".to_string(),
            rules: vec![],
        };

        let node_config = NodeConfig {
            node_id: 41,
            node_type: "Vmess".to_string(),
            node_speed_limit: 0,
            traffic_rate: 1.0,
            sort: 1,
            inbounds: vec![serde_json::json!({
                "port": 10086,
                "protocol": "vmess",
                "settings": {},
                "streamSettings": {
                    "network": "tcp"
                }
            })],
        };

        Self {
            users: Arc::new(RwLock::new(users)),
            outbounds: Arc::new(RwLock::new(outbounds)),
            user_mapping: Arc::new(RwLock::new(user_mapping)),
            routing: Arc::new(RwLock::new(routing)),
            node_config: Arc::new(RwLock::new(node_config)),
            maintenance_mode: Arc::new(RwLock::new(false)),
        }
    }

    // ========== 用户管理 ==========
    pub async fn get_users(&self) -> Vec<User> {
        self.users.read().await.clone()
    }

    pub async fn add_user(&self, uuid: String, st: u64, dt: u64) -> Result<User, String> {
        let mut users = self.users.write().await;
        
        // 检查用户是否已存在
        if users.iter().any(|u| u.uuid == uuid) {
            return Err("用户已存在".to_string());
        }
        
        let new_id = users.iter().map(|u| u.id).max().unwrap_or(0) + 1;
        let user = User {
            id: new_id,
            uuid,
            st,
            dt,
        };
        users.push(user.clone());
        Ok(user)
    }

    pub async fn update_user(&self, id: u64, uuid: Option<String>, st: Option<u64>, dt: Option<u64>) -> Result<(), String> {
        let mut users = self.users.write().await;
        if let Some(user) = users.iter_mut().find(|u| u.id == id) {
            if let Some(uuid) = uuid {
                user.uuid = uuid;
            }
            if let Some(st) = st {
                user.st = st;
            }
            if let Some(dt) = dt {
                user.dt = dt;
            }
            Ok(())
        } else {
            Err("用户不存在".to_string())
        }
    }

    pub async fn delete_user(&self, id: u64) -> Result<User, String> {
        let mut users = self.users.write().await;
        let mut user_mapping = self.user_mapping.write().await;
        
        if let Some(pos) = users.iter().position(|u| u.id == id) {
            let user = users.remove(pos);
            // 删除映射
            user_mapping.remove(&user.uuid);
            Ok(user)
        } else {
            Err("用户不存在".to_string())
        }
    }

    // ========== 上游代理管理 ==========
    pub async fn get_outbounds(&self) -> (Vec<OutboundConfig>, HashMap<String, String>) {
        let outbounds = self.outbounds.read().await.clone();
        let mapping = self.user_mapping.read().await.clone();
        (outbounds, mapping)
    }

    pub async fn add_outbound(&self, outbound: OutboundConfig) -> Result<(), String> {
        let mut outbounds = self.outbounds.write().await;
        
        if outbounds.iter().any(|o| o.tag == outbound.tag) {
            return Err(format!("tag {} 已存在", outbound.tag));
        }
        
        outbounds.push(outbound);
        Ok(())
    }

    pub async fn update_outbound(&self, tag: &str, protocol: Option<String>, settings: Option<Value>) -> Result<(), String> {
        let mut outbounds = self.outbounds.write().await;
        
        if let Some(outbound) = outbounds.iter_mut().find(|o| o.tag == tag) {
            if let Some(protocol) = protocol {
                outbound.protocol = protocol;
            }
            if let Some(settings) = settings {
                outbound.settings = settings;
            }
            Ok(())
        } else {
            Err("上游代理不存在".to_string())
        }
    }

    pub async fn delete_outbound(&self, tag: &str) -> Result<(), String> {
        let mut outbounds = self.outbounds.write().await;
        let mut user_mapping = self.user_mapping.write().await;
        
        if let Some(pos) = outbounds.iter().position(|o| o.tag == tag) {
            outbounds.remove(pos);
            // 删除所有使用该 outbound 的映射
            user_mapping.retain(|_, v| v != tag);
            Ok(())
        } else {
            Err("上游代理不存在".to_string())
        }
    }

    // ========== 用户映射管理 ==========
    pub async fn add_mapping(&self, uuid: String, outbound_tag: String) {
        let mut mapping = self.user_mapping.write().await;
        mapping.insert(uuid, outbound_tag);
    }

    pub async fn update_mapping(&self, uuid: &str, outbound_tag: String) -> Result<String, String> {
        let mut mapping = self.user_mapping.write().await;
        
        if let Some(old_tag) = mapping.get(uuid).cloned() {
            mapping.insert(uuid.to_string(), outbound_tag.clone());
            Ok(old_tag)
        } else {
            Err("映射不存在".to_string())
        }
    }

    pub async fn delete_mapping(&self, uuid: &str) -> Result<(), String> {
        let mut mapping = self.user_mapping.write().await;
        if mapping.remove(uuid).is_some() {
            Ok(())
        } else {
            Err("映射不存在".to_string())
        }
    }

    // ========== 路由配置管理 ==========
    pub async fn get_routing(&self) -> RoutingConfig {
        self.routing.read().await.clone()
    }

    pub async fn update_routing(&self, domain_strategy: Option<String>, rules: Option<Vec<RoutingRule>>) {
        let mut routing = self.routing.write().await;
        if let Some(domain_strategy) = domain_strategy {
            routing.domain_strategy = domain_strategy;
        }
        if let Some(rules) = rules {
            routing.rules = rules;
        }
    }

    pub async fn add_routing_rule(&self, rule: RoutingRule) {
        let mut routing = self.routing.write().await;
        routing.rules.push(rule);
    }

    pub async fn update_routing_rule(&self, index: usize, rule: RoutingRule) -> Result<(), String> {
        let mut routing = self.routing.write().await;
        if index < routing.rules.len() {
            routing.rules[index] = rule;
            Ok(())
        } else {
            Err("规则索引超出范围".to_string())
        }
    }

    pub async fn delete_routing_rule(&self, index: usize) -> Result<RoutingRule, String> {
        let mut routing = self.routing.write().await;
        if index < routing.rules.len() {
            Ok(routing.rules.remove(index))
        } else {
            Err("规则索引超出范围".to_string())
        }
    }

    // ========== 节点配置 ==========
    pub async fn get_node_config(&self) -> NodeConfig {
        self.node_config.read().await.clone()
    }

    // ========== 维护模式 ==========
    pub async fn get_maintenance_mode(&self) -> bool {
        *self.maintenance_mode.read().await
    }

    pub async fn set_maintenance_mode(&self, enabled: bool) -> bool {
        let mut mode = self.maintenance_mode.write().await;
        let old = *mode;
        *mode = enabled;
        old
    }
}

