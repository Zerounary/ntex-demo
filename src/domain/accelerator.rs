use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type GameId = String;
pub type ProfileId = String;
pub type NodeId = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    pub id: GameId,
    pub name: String,
    pub icon: String,
    pub status: String,
    pub ping: i32,
    pub process_name: String,
    pub routing_rules: String,
    pub sniff_domains_excluded: String,
    pub region: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub id: ProfileId,
    pub game_id: GameId,
    pub display_name: String,
    pub node_id: NodeId,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub vless_id: String,
    pub vless_server: String,
    pub vless_port: i32,
    pub vless_encryption: String,
    pub reality_server_name: String,
    pub reality_public_key: String,
    pub reality_short_id: String,
    pub reality_fingerprint: String,
    pub reality_spider_x: String,
    pub udp_proxy: String,
    pub mode: String,
    pub ping: i32,
    pub status: String,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcceleratorUser {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub valid_until: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BootstrapPayload {
    pub games: Vec<Game>,
    pub nodes: Vec<Node>,
    pub profiles: Vec<Profile>,
    pub user: Option<AcceleratorUser>,
}
