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
    pub vmess_uuid: String,
    pub vmess_server: String,
    pub vmess_port: i32,
    pub vmess_email: String,
    pub udp_proxy: String,
    pub mode: String,
    pub ping: i32,
    pub status: String,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcceleratorUser {
    pub id: String,
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
