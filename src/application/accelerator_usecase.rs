use crate::domain::accelerator::{BootstrapPayload, Game, Node, Profile};

use super::errors::UsecaseError;
use super::ports::{AcceleratorRepository, NodeRepository};

/// 用于从旧的 ProfileVO 格式转换的辅助结构
pub struct ProfileVOData {
    pub id: String,
    pub game_id: String,
    pub display_name: String,
    pub process_name: String,
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
    pub status: String,
    pub region: String,
    pub ping: i32,
}

pub struct AcceleratorUseCase<A, N> {
    accelerator_repo: A,
    node_repo: N,
}

impl<A, N> AcceleratorUseCase<A, N> {
    pub fn new(accelerator_repo: A, node_repo: N) -> Self {
        Self {
            accelerator_repo,
            node_repo,
        }
    }
}

impl<A, N> AcceleratorUseCase<A, N>
where
    A: AcceleratorRepository,
    N: NodeRepository,
{
    pub async fn bootstrap(&self) -> Result<BootstrapPayload, UsecaseError> {
        let payload = self.accelerator_repo.bootstrap().await?;
        Ok(payload)
    }

    pub async fn sync_profiles_from_vo(
        &self,
        profile_vos: Vec<ProfileVOData>,
    ) -> Result<(), UsecaseError> {
        if profile_vos.is_empty() {
            return Err(UsecaseError::Validation(
                "profiles payload cannot be empty".into(),
            ));
        }

        use chrono::Utc;
        use std::collections::HashMap;

        // 收集所有唯一的节点和游戏
        let mut node_map: HashMap<String, Node> = HashMap::new();
        let mut game_map: HashMap<String, Game> = HashMap::new();
        let mut profiles: Vec<Profile> = Vec::new();

        // 首先获取现有的游戏和节点
        let existing_games = self.accelerator_repo.list_games().await?;
        for game in existing_games {
            game_map.insert(game.id.clone(), game);
        }

        for profile_vo in profile_vos {
            // 创建或更新节点
            let node_id = format!("node-{}", profile_vo.id);
            let node = Node {
                id: node_id.clone(),
                vless_id: profile_vo.vless_id,
                vless_server: profile_vo.vless_server,
                vless_port: profile_vo.vless_port,
                vless_encryption: profile_vo.vless_encryption,
                reality_server_name: profile_vo.reality_server_name,
                reality_public_key: profile_vo.reality_public_key,
                reality_short_id: profile_vo.reality_short_id,
                reality_fingerprint: profile_vo.reality_fingerprint,
                reality_spider_x: profile_vo.reality_spider_x,
                udp_proxy: profile_vo.udp_proxy,
                mode: profile_vo.mode,
                ping: profile_vo.ping,
                status: "active".to_string(),
                last_heartbeat: Utc::now(),
            };
            node_map.insert(node_id.clone(), node.clone());
            self.node_repo.register_node(node).await?;

            // 更新游戏信息（process_name 和 region）
            if let Some(game) = game_map.get_mut(&profile_vo.game_id) {
                game.process_name = profile_vo.process_name.clone();
                game.region = profile_vo.region.clone();
            } else {
                // 如果游戏不存在，创建一个新的（这种情况不应该发生，但为了健壮性）
                let game = Game {
                    id: profile_vo.game_id.clone(),
                    name: "Unknown".to_string(),
                    icon: "".to_string(),
                    status: "idle".to_string(),
                    ping: 0,
                    process_name: profile_vo.process_name.clone(),
                    routing_rules: "".to_string(),
                    region: profile_vo.region.clone(),
                };
                game_map.insert(game.id.clone(), game);
            }

            // 创建新的 Profile
            let profile = Profile {
                id: profile_vo.id,
                game_id: profile_vo.game_id,
                display_name: profile_vo.display_name,
                node_id,
                status: profile_vo.status,
            };
            profiles.push(profile);
        }

        // 更新游戏（如果需要）
        // 注意：这里我们假设游戏已经存在，如果需要创建新游戏，需要添加相应的方法

        // 更新 profiles
        self.accelerator_repo.upsert_profiles(profiles).await?;
        Ok(())
    }

    pub async fn sync_profiles(&self, profiles: Vec<Profile>) -> Result<(), UsecaseError> {
        if profiles.is_empty() {
            return Err(UsecaseError::Validation(
                "profiles payload cannot be empty".into(),
            ));
        }
        self.accelerator_repo.upsert_profiles(profiles).await?;
        Ok(())
    }
}
