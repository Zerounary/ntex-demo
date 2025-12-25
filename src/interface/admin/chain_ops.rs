use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;

use crate::infrastructure::admin_config::{AdminConfigStore, ChainDefinition, InboundConfig, OutboundConfig, RoutingRule};
use crate::infrastructure::mqtt_client::MqttClientManager;

fn hop_inbound_prefix(chain_id: &str) -> String {
    format!("chain_{}_", chain_id)
}

fn entry_inbound_tag(chain_id: &str) -> String {
    format!("chain_entry_in_{}", chain_id)
}

fn entry_outbound_tag(chain_id: &str) -> String {
    format!("chain_entry_{}", chain_id)
}

fn select_unused_port(used_ports: &HashSet<u16>, start: u16, end: u16) -> Option<u16> {
    if start == 0 || end == 0 || start > end {
        return None;
    }
    for p in start..=end {
        if !used_ports.contains(&p) {
            return Some(p);
        }
    }
    None
}

fn publish_updates(mqtt: Option<&Arc<MqttClientManager>>, node_id: u64, kinds: &[&str]) {
    if let Some(mqtt) = mqtt {
        let mqtt = mqtt.clone();
        let kinds: Vec<&str> = kinds.to_vec();
        tokio::spawn(async move {
            for k in kinds {
                let _ = mqtt.publish_update_notification(node_id, k).await;
            }
        });
    }
}

pub async fn apply_chain(
    config: &AdminConfigStore,
    mqtt: Option<&Arc<MqttClientManager>>,
    chain_id: &str,
    base_port: u16,
) -> Result<(String, Vec<Value>), String> {
    let chains = config.get_chains(0).await?;
    let chain = chains
        .into_iter()
        .find(|c| c.id == chain_id)
        .ok_or_else(|| "chain not found".to_string())?;

    let chain_id = chain.id.clone();
    let chain_uuid = chain.uuid.clone();

    if chain.routes.is_empty() {
        return Err("chain has no routes".to_string());
    }

    cleanup_chain_artifacts(config, mqtt, &chain_id).await.ok();

    let mut hops = chain.routes.clone();
    hops.sort_by_key(|r| r.order);

    for r in &hops {
        if r.from_node_id == r.to_node_id {
            return Err("invalid route: from == to".to_string());
        }
    }

    let mut node_path: Vec<u64> = Vec::new();
    node_path.push(hops[0].from_node_id);
    for r in &hops {
        if let Some(last) = node_path.last().copied() {
            if last != r.from_node_id {
                return Err(format!("route chain is not continuous at order {}", r.order));
            }
        }
        node_path.push(r.to_node_id);
    }

    let mut endpoints: Vec<(String, u16)> = Vec::new();
    for (idx, node_id) in node_path.iter().enumerate() {
        let ip = config.get_node_public_ip(*node_id).await?;

        let port: u16 = if idx == node_path.len() - 1 {
            config.select_default_forward_port(*node_id).await? as u16
        } else {
            base_port.saturating_add(idx as u16)
        };
        endpoints.push((ip, port));
    }

    let mut results: Vec<Value> = Vec::new();

    for i in 0..(node_path.len() - 1) {
        let node_id = node_path[i];
        let (next_ip, next_port) = endpoints[i + 1].clone();
        let listen_port = endpoints[i].1;

        let tag = format!("chain_{}_{}", chain_id, i + 1);
        let listen = if i == 0 { "127.0.0.1" } else { "0.0.0.0" };
        let inbound = InboundConfig {
            tag: tag.clone(),
            protocol: "dokodemo-door".to_string(),
            port: listen_port as i32,
            listen: Some(listen.to_string()),
            settings: serde_json::json!({
                "address": next_ip,
                "port": next_port,
                "network": ["tcp", "udp"],
                "followRedirect": false
            }),
            stream_settings: None,
            sniffing: None,
        };

        let apply_res = match config.upsert_inbound(node_id, inbound).await {
            Ok(_) => {
                publish_updates(mqtt, node_id, &["inbound", "config"]);
                serde_json::json!({
                    "node_id": node_id,
                    "tag": tag,
                    "status": "ok"
                })
            }
            Err(e) => serde_json::json!({
                "node_id": node_id,
                "tag": tag,
                "status": "error",
                "error": e
            }),
        };
        results.push(apply_res);
    }

    if let Some(first_node_id) = node_path.first().copied() {
        let chain_listen_port = endpoints[0].1;
        let outbound_tag = entry_outbound_tag(&chain_id);
        let inbound_tag = entry_inbound_tag(&chain_id);

        let existing_inbounds = config.get_inbounds(first_node_id).await.unwrap_or_default();
        let mut used_ports: HashSet<u16> = existing_inbounds
            .iter()
            .filter(|i| i.port > 0)
            .filter_map(|i| u16::try_from(i.port).ok())
            .collect();
        used_ports.insert(chain_listen_port);

        let start = base_port.saturating_add(1000);
        let end = base_port.saturating_add(1100);
        let entry_port = select_unused_port(&used_ports, start, end)
            .ok_or_else(|| format!("no available port for chain entry inbound in range {}-{}", start, end))?;

        let entry_inbound = InboundConfig {
            tag: inbound_tag.clone(),
            protocol: "vmess".to_string(),
            port: entry_port as i32,
            listen: Some("0.0.0.0".to_string()),
            settings: serde_json::json!({
                "clients": [{
                    "id": chain_uuid,
                    "alterId": 0,
                    "email": format!("chain:{}@local", chain_id),
                    "security": "auto"
                }]
            }),
            stream_settings: Some(serde_json::json!({
                "network": "tcp"
            })),
            sniffing: None,
        };

        let entry_inbound_res = match config.upsert_inbound(first_node_id, entry_inbound).await {
            Ok(_) => {
                publish_updates(mqtt, first_node_id, &["inbound", "config"]);
                serde_json::json!({
                    "node_id": first_node_id,
                    "tag": inbound_tag,
                    "port": entry_port,
                    "status": "ok"
                })
            }
            Err(e) => serde_json::json!({
                "node_id": first_node_id,
                "tag": inbound_tag,
                "port": entry_port,
                "status": "error",
                "error": e
            }),
        };
        results.push(entry_inbound_res);

        let outbound = OutboundConfig {
            tag: outbound_tag.clone(),
            protocol: "freedom".to_string(),
            settings: serde_json::json!({
                "domainStrategy": "AsIs",
                "redirect": format!("127.0.0.1:{}", chain_listen_port)
            }),
            stream_settings: None,
        };

        let outbound_res = match config.upsert_outbound(first_node_id, outbound).await {
            Ok(_) => {
                publish_updates(mqtt, first_node_id, &["outbound"]);
                serde_json::json!({
                    "node_id": first_node_id,
                    "tag": outbound_tag,
                    "status": "ok"
                })
            }
            Err(e) => serde_json::json!({
                "node_id": first_node_id,
                "tag": outbound_tag,
                "status": "error",
                "error": e
            }),
        };
        results.push(outbound_res);

        let routing_res = match config.get_routing(first_node_id).await {
            Ok(mut routing) => {
                routing.rules.retain(|r| {
                    if r.outbound_tag.as_deref() == Some(outbound_tag.as_str()) {
                        return false;
                    }
                    if let Some(tags) = r.inbound_tag.as_ref() {
                        return !tags.iter().any(|t| t == &inbound_tag);
                    }
                    true
                });
                routing.rules.insert(
                    0,
                    RoutingRule {
                        rule_type: "field".to_string(),
                        inbound_tag: Some(vec![inbound_tag.clone()]),
                        outbound_tag: Some(outbound_tag.clone()),
                        domain: None,
                        ip: None,
                        port: None,
                        network: None,
                        source: None,
                        protocol: None,
                    },
                );

                match config
                    .update_routing(first_node_id, None, Some(routing.rules))
                    .await
                {
                    Ok(_) => {
                        publish_updates(mqtt, first_node_id, &["routing", "config"]);
                        serde_json::json!({
                            "node_id": first_node_id,
                            "tag": format!("routing:{}", inbound_tag),
                            "status": "ok"
                        })
                    }
                    Err(e) => serde_json::json!({
                        "node_id": first_node_id,
                        "tag": format!("routing:{}", inbound_tag),
                        "status": "error",
                        "error": e
                    }),
                }
            }
            Err(e) => serde_json::json!({
                "node_id": first_node_id,
                "tag": format!("routing:{}", inbound_tag),
                "status": "error",
                "error": e
            }),
        };
        results.push(routing_res);
    }

    if let Some(last_node_id) = node_path.last().copied() {
        publish_updates(mqtt, last_node_id, &["config"]);
    }

    Ok((chain_id, results))
}

pub async fn cleanup_chain_artifacts(
    config: &AdminConfigStore,
    mqtt: Option<&Arc<MqttClientManager>>,
    chain_id: &str,
) -> Result<(), String> {
    let node_ids = config.list_node_ids().await?;

    let hop_prefix = hop_inbound_prefix(chain_id);
    let in_tag = entry_inbound_tag(chain_id);
    let out_tag = entry_outbound_tag(chain_id);

    for node_id in node_ids {
        let inbounds = config.get_inbounds(node_id).await.unwrap_or_default();
        let mut deleted_any_inbound = false;
        for inbound in inbounds {
            if inbound.tag == in_tag || inbound.tag.starts_with(&hop_prefix) {
                if config.delete_inbound(node_id, &inbound.tag).await.is_ok() {
                    deleted_any_inbound = true;
                }
            }
        }

        let deleted_outbound = config.delete_outbound(node_id, &out_tag).await.is_ok();

        let mut updated_routing = false;
        if let Ok(mut routing) = config.get_routing(node_id).await {
            let before = routing.rules.len();
            routing.rules.retain(|r| {
                if r.outbound_tag.as_deref() == Some(out_tag.as_str()) {
                    return false;
                }
                if let Some(tags) = r.inbound_tag.as_ref() {
                    if tags.iter().any(|t| t == &in_tag) {
                        return false;
                    }
                }
                true
            });
            if routing.rules.len() != before {
                if config.update_routing(node_id, None, Some(routing.rules)).await.is_ok() {
                    updated_routing = true;
                }
            }
        }

        if deleted_any_inbound {
            publish_updates(mqtt, node_id, &["inbound", "config"]);
        }
        if deleted_outbound {
            publish_updates(mqtt, node_id, &["outbound", "config"]);
        }
        if updated_routing {
            publish_updates(mqtt, node_id, &["routing", "config"]);
        }
    }

    Ok(())
}
