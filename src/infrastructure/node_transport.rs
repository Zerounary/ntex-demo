use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;

use once_cell::sync::Lazy;
use sea_orm::EntityTrait;
use std::env;

use crate::infrastructure::admin_config::AdminConfigStore;
use crate::infrastructure::mqtt_client::MqttClientManager;
use crate::infrastructure::persistence::admin_node_config;
use crate::interface::grpc_server;

#[async_trait]
pub trait NodeTransport: Send + Sync {
    async fn publish_update_notification(&self, node_id: u64, update_type: &str) -> Result<(), String>;

    async fn wait_for_node_pull(
        &self,
        node_id: u64,
        action: &str,
        timeout_secs: u64,
    ) -> Result<(), String>;

    async fn query_node_logs(&self, node_id: u64, query_params: Value, timeout: u64) -> Option<Value>;

    async fn query_node_network_interfaces(&self, node_id: u64, timeout: u64) -> Option<Value>;

    async fn query_udp_latency(
        &self,
        node_id: u64,
        outbound_tag: &str,
        timeout: u64,
    ) -> Option<Value>;
}

#[derive(Clone)]
pub struct GrpcTransport;

impl GrpcTransport {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl NodeTransport for GrpcTransport {
    async fn publish_update_notification(&self, node_id: u64, update_type: &str) -> Result<(), String> {
        grpc_server::send_update_event(node_id, update_type).await
    }

    async fn wait_for_node_pull(&self, node_id: u64, action: &str, timeout_secs: u64) -> Result<(), String> {
        grpc_server::wait_for_pull_ack(node_id, action, timeout_secs).await
    }

    async fn query_node_logs(&self, _node_id: u64, _query_params: Value, _timeout: u64) -> Option<Value> {
        None
    }

    async fn query_node_network_interfaces(&self, _node_id: u64, _timeout: u64) -> Option<Value> {
        None
    }

    async fn query_udp_latency(
        &self,
        _node_id: u64,
        _outbound_tag: &str,
        _timeout: u64,
    ) -> Option<Value> {
        None
    }
}

static GRPC_GRAY_NODE_IDS: Lazy<Option<HashSet<u64>>> = Lazy::new(|| {
    let raw = env::var("GRPC_GRAY_NODE_IDS").ok()?;
    let mut set = HashSet::new();
    for part in raw.split(',') {
        let s = part.trim();
        if s.is_empty() {
            continue;
        }
        if let Ok(id) = s.parse::<u64>() {
            set.insert(id);
        }
    }
    Some(set)
});

#[derive(Clone)]
pub struct DispatchTransport {
    admin_config: AdminConfigStore,
    mqtt: Option<Arc<dyn NodeTransport>>,
    grpc: GrpcTransport,
}

impl DispatchTransport {
    pub fn new(admin_config: AdminConfigStore, mqtt: Option<Arc<dyn NodeTransport>>, grpc: GrpcTransport) -> Self {
        Self {
            admin_config,
            mqtt,
            grpc,
        }
    }

    async fn node_comm_mode(&self, node_id: u64) -> String {
        let row = admin_node_config::Entity::find_by_id(node_id)
            .one(self.admin_config.db())
            .await
            .ok()
            .flatten();
        row.map(|r| r.node_comm_mode.to_lowercase())
            .unwrap_or_else(|| "mqtt".to_string())
    }

    fn should_try_grpc(&self, node_id: u64, mode: &str) -> bool {
        match mode {
            "grpc" => true,
            "auto" => GRPC_GRAY_NODE_IDS
                .as_ref()
                .map(|s| s.contains(&node_id))
                .unwrap_or(false),
            _ => false,
        }
    }
}

#[async_trait]
impl NodeTransport for DispatchTransport {
    async fn publish_update_notification(&self, node_id: u64, update_type: &str) -> Result<(), String> {
        let mode = self.node_comm_mode(node_id).await;

        if mode == "grpc" {
            return self.grpc.publish_update_notification(node_id, update_type).await;
        }

        if self.should_try_grpc(node_id, &mode) {
            if let Ok(()) = self.grpc.publish_update_notification(node_id, update_type).await {
                return Ok(());
            }
        }

        let Some(mqtt) = self.mqtt.as_ref() else {
            return Err("mqtt transport not available".to_string());
        };

        mqtt.publish_update_notification(node_id, update_type).await
    }

    async fn wait_for_node_pull(&self, node_id: u64, action: &str, timeout_secs: u64) -> Result<(), String> {
        let mode = self.node_comm_mode(node_id).await;

        if mode == "grpc" {
            return self.grpc.wait_for_node_pull(node_id, action, timeout_secs).await;
        }

        if self.should_try_grpc(node_id, &mode) {
            if let Ok(()) = self.grpc.wait_for_node_pull(node_id, action, timeout_secs).await {
                return Ok(());
            }
        }

        let Some(mqtt) = self.mqtt.as_ref() else {
            return Err("mqtt transport not available".to_string());
        };

        mqtt.wait_for_node_pull(node_id, action, timeout_secs).await
    }

    async fn query_node_logs(&self, node_id: u64, query_params: Value, timeout: u64) -> Option<Value> {
        match self.mqtt.as_ref() {
            Some(mqtt) => mqtt.query_node_logs(node_id, query_params, timeout).await,
            None => None,
        }
    }

    async fn query_node_network_interfaces(&self, node_id: u64, timeout: u64) -> Option<Value> {
        match self.mqtt.as_ref() {
            Some(mqtt) => mqtt.query_node_network_interfaces(node_id, timeout).await,
            None => None,
        }
    }

    async fn query_udp_latency(
        &self,
        node_id: u64,
        outbound_tag: &str,
        timeout: u64,
    ) -> Option<Value> {
        match self.mqtt.as_ref() {
            Some(mqtt) => mqtt.query_udp_latency(node_id, outbound_tag, timeout).await,
            None => None,
        }
    }
}

#[derive(Clone)]
pub struct MqttTransport {
    mqtt: Arc<MqttClientManager>,
}

impl MqttTransport {
    pub fn new(mqtt: Arc<MqttClientManager>) -> Self {
        Self { mqtt }
    }
}

#[async_trait]
impl NodeTransport for MqttTransport {
    async fn publish_update_notification(&self, node_id: u64, update_type: &str) -> Result<(), String> {
        self.mqtt
            .publish_update_notification(node_id, update_type)
            .await
            .map_err(|e| e.to_string())
    }

    async fn wait_for_node_pull(
        &self,
        node_id: u64,
        action: &str,
        timeout_secs: u64,
    ) -> Result<(), String> {
        self.mqtt.wait_for_node_pull(node_id, action, timeout_secs).await
    }

    async fn query_node_logs(&self, node_id: u64, query_params: Value, timeout: u64) -> Option<Value> {
        self.mqtt.query_node_logs(node_id, query_params, timeout).await
    }

    async fn query_node_network_interfaces(&self, node_id: u64, timeout: u64) -> Option<Value> {
        self.mqtt.query_node_network_interfaces(node_id, timeout).await
    }

    async fn query_udp_latency(
        &self,
        node_id: u64,
        outbound_tag: &str,
        timeout: u64,
    ) -> Option<Value> {
        self.mqtt
            .query_udp_latency(node_id, outbound_tag, timeout)
            .await
    }
}
