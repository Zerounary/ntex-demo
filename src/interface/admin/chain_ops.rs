use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::infrastructure::admin_config::{AdminConfigStore, InboundConfig};
use crate::infrastructure::node_transport::NodeTransport;

fn hop_inbound_prefix(chain_id: i64) -> String {
    format!("chain_{}_", chain_id)
}

fn allocate_listen_port(
    node_id: u64,
    start_port: u16,
    node_ports_cache: &mut HashMap<u64, HashSet<u16>>,
) -> Result<u16, String> {
    let ports = node_ports_cache
        .get_mut(&node_id)
        .ok_or_else(|| format!("missing port cache for node_id={}", node_id))?;

    let mut candidate = start_port;
    loop {
        if !ports.contains(&candidate) {
            ports.insert(candidate);
            return Ok(candidate);
        }

        candidate = candidate
            .checked_add(1)
            .ok_or_else(|| format!("no available port on node_id={} for chain inbounds", node_id))?;
    }
}

fn entry_inbound_tag(chain_id: i64) -> String {
    format!("chain_entry_in_{}", chain_id)
}

fn entry_outbound_tag(chain_id: i64) -> String {
    format!("chain_entry_{}", chain_id)
}

fn publish_updates(transport: Option<&Arc<dyn NodeTransport>>, node_id: u64, kinds: &[&str]) {
    if let Some(transport) = transport {
        let transport = transport.clone();
        let kinds: Vec<String> = kinds.iter().map(|k| k.to_string()).collect();
        tokio::spawn(async move {
            for k in kinds {
                let _ = transport.publish_update_notification(node_id, &k).await;
            }
        });
    }
}

pub async fn apply_chain(
    config: &AdminConfigStore,
    transport: Option<&Arc<dyn NodeTransport>>,
    chain_id: i64,
    base_port: u16,
) -> Result<(i64, Vec<Value>), String> {
    let chains = config.get_chains().await?;
    let chain = chains
        .into_iter()
        .find(|c| c.id == chain_id)
        .ok_or_else(|| "chain not found".to_string())?;

    let chain_id = chain.id;

    if chain.routes.is_empty() {
        return Err("chain has no routes".to_string());
    }

    cleanup_chain_artifacts(config, transport, chain_id).await.ok();

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

    let mut node_ports_cache: HashMap<u64, HashSet<u16>> = HashMap::new();
    for node_id in node_path.iter().copied().take(node_path.len() - 1) {
        if node_ports_cache.contains_key(&node_id) {
            continue;
        }
        let inbounds = config.get_inbounds(node_id).await?;
        let mut ports = HashSet::new();
        for inbound in inbounds {
            if inbound.port > 0 && inbound.port <= u16::MAX as i32 {
                ports.insert(inbound.port as u16);
            }
        }
        node_ports_cache.insert(node_id, ports);
    }

    let mut endpoints: Vec<(String, u16)> = Vec::new();
    for (idx, node_id) in node_path.iter().enumerate() {
        let ip = config.get_node_public_ip(*node_id).await?;

        let port: u16 = if idx == node_path.len() - 1 {
            config.select_default_forward_port(*node_id).await? as u16
        } else {
            allocate_listen_port(
                *node_id,
                base_port.saturating_add(idx as u16),
                &mut node_ports_cache,
            )?
        };
        endpoints.push((ip, port));
    }

    let mut results: Vec<Value> = Vec::new();

    for i in 0..(node_path.len() - 1) {
        let node_id = node_path[i];
        let (next_ip, next_port) = endpoints[i + 1].clone();
        let listen_port = endpoints[i].1;

        let tag = format!("chain_{}_{}", chain_id, i + 1);
        let listen = "0.0.0.0";
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
                publish_updates(transport, node_id, &["inbound", "config"]);
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

    Ok((chain_id, results))
}

pub async fn cleanup_chain_artifacts(
    config: &AdminConfigStore,
    transport: Option<&Arc<dyn NodeTransport>>,
    chain_id: i64,
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
            publish_updates(transport, node_id, &["inbound", "config"]);
        }
        if deleted_outbound {
            publish_updates(transport, node_id, &["outbound", "config"]);
        }
        if updated_routing {
            publish_updates(transport, node_id, &["routing", "config"]);
        }
    }

    Ok(())
}
