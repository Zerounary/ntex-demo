use async_trait::async_trait;
use chrono::{Duration, Utc};
use moka::future::Cache;
use once_cell::sync::Lazy;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, JsonValue, QueryFilter, Set,
};
use sha2::{Digest, Sha256};

use crate::application::errors::RepositoryError;
use crate::application::ports::{
    AcceleratorRepository, AuthRepository, CdkRepository, ConfigRepository, NodeRepository,
};
use crate::domain::accelerator::{AcceleratorUser, BootstrapPayload, Game, Node, Profile};
use crate::domain::auth::{AccountLoginRequest, AccountLoginResponse, TicketStatus, WechatTicket};
use crate::domain::cdk::{CdkCode, CdkGenerateRequest, CdkRedeemRequest, CdkRedeemResponse, CdkStatus, CdkType};
use log::{info, warn};

use super::accelerator_game;
use super::accelerator_node;
use super::accelerator_game_node_binding;
use super::accelerator_profile;
use super::accelerator_user;
use super::account_user;
use super::cdk_code;
use super::config_entry;
use super::user_wallet;
use super::wechat_ticket;

static GAME_CACHE: Lazy<Cache<&'static str, Vec<Game>>> = Lazy::new(|| {
    Cache::builder()
        .max_capacity(32)
        .time_to_live(std::time::Duration::from_secs(30))
        .build()
});

static PROFILE_CACHE: Lazy<Cache<&'static str, Vec<Profile>>> = Lazy::new(|| {
    Cache::builder()
        .max_capacity(32)
        .time_to_live(std::time::Duration::from_secs(30))
        .build()
});

static BOOTSTRAP_CACHE: Lazy<Cache<&'static str, BootstrapPayload>> = Lazy::new(|| {
    Cache::builder()
        .max_capacity(16)
        .time_to_live(std::time::Duration::from_secs(30))
        .build()
});

pub struct AcceleratorRepositoryImpl<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> AcceleratorRepositoryImpl<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl<'a> AcceleratorRepository for AcceleratorRepositoryImpl<'a> {
    async fn list_games(&self) -> Result<Vec<Game>, RepositoryError> {
        if let Some(cached) = GAME_CACHE.get("all").await {
            return Ok(cached);
        }

        let models = accelerator_game::Entity::find()
            .all(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        let list: Vec<Game> = models.into_iter().map(Into::into).collect();
        GAME_CACHE.insert("all", list.clone()).await;
        Ok(list)
    }

    async fn list_profiles(&self) -> Result<Vec<Profile>, RepositoryError> {
        if let Some(cached) = PROFILE_CACHE.get("all").await {
            return Ok(cached);
        }

        let models = accelerator_profile::Entity::find()
            .all(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        let list: Vec<Profile> = models.into_iter().map(Into::into).collect();
        PROFILE_CACHE.insert("all", list.clone()).await;
        Ok(list)
    }

    async fn upsert_profiles(&self, profiles: Vec<Profile>) -> Result<(), RepositoryError> {
        accelerator_profile::Entity::delete_many()
            .exec(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;

        for profile in profiles {
            let active = profile_into_active_model(profile);
            active
                .insert(self.db)
                .await
                .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        }

        PROFILE_CACHE.invalidate_all();
        BOOTSTRAP_CACHE.invalidate_all();
        Ok(())
    }

    async fn current_user(&self) -> Result<Option<AcceleratorUser>, RepositoryError> {
        let user = accelerator_user::Entity::find()
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(user.map(Into::into))
    }

    async fn bootstrap(&self) -> Result<BootstrapPayload, RepositoryError> {
        if let Some(cached) = BOOTSTRAP_CACHE.get("all").await {
            return Ok(cached);
        }

        let games = self.list_games().await?;
        let user = self.current_user().await?;
        
        // 获取所有节点
        let node_repo = NodeRepositoryImpl::new(self.db);
        let nodes = node_repo.list_nodes().await?;

        // profiles：从 accelerator_game_node_bindings 构造（由 game config 决定可选节点）
        let binding_models = accelerator_game_node_binding::Entity::find()
            .all(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;

        let node_map: std::collections::HashMap<String, crate::domain::accelerator::Node> =
            nodes.iter().cloned().map(|n| (n.id.clone(), n)).collect();

        let mut profiles: Vec<crate::domain::accelerator::Profile> = Vec::new();
        for b in binding_models {
            if b.r#type != "node" {
                continue;
            }
            let node_id = match b.node_id.clone() {
                Some(v) => v,
                None => continue,
            };
            let node = match node_map.get(&node_id) {
                Some(v) => v,
                None => continue,
            };

            let display_name = b
                .display_name
                .clone()
                .unwrap_or_else(|| node_id.clone());
            let status = b.status.clone().unwrap_or_else(|| node.status.clone());

            profiles.push(crate::domain::accelerator::Profile {
                id: format!("bind_{}", b.id),
                game_id: b.game_id.clone(),
                display_name,
                node_id,
                status,
            });
        }
        
        let payload = BootstrapPayload {
            games,
            nodes,
            profiles,
            user,
        };
        BOOTSTRAP_CACHE.insert("all", payload.clone()).await;
        Ok(payload)
    }
}

fn profile_into_active_model(profile: Profile) -> accelerator_profile::ActiveModel {
    accelerator_profile::ActiveModel {
        id: Set(profile.id),
        game_id: Set(profile.game_id),
        display_name: Set(profile.display_name),
        node_id: Set(profile.node_id),
        status: Set(profile.status),
    }
}

pub struct NodeRepositoryImpl<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> NodeRepositoryImpl<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl<'a> NodeRepository for NodeRepositoryImpl<'a> {
    async fn register_node(&self, node: Node) -> Result<(), RepositoryError> {
        let node_id = node.id.clone();
        let active = node_into_active_model(node);
        active
            .insert(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        info!("Node registered: {}", node_id);
        Ok(())
    }

    async fn unregister_node(&self, node_id: &str) -> Result<(), RepositoryError> {
        let result = accelerator_node::Entity::delete_by_id(node_id.to_string())
            .exec(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        
        if result.rows_affected > 0 {
            info!("Node unregistered: {}", node_id);
        } else {
            warn!("Node not found for unregister: {}", node_id);
        }
        Ok(())
    }

    async fn update_node_heartbeat(&self, node_id: &str) -> Result<(), RepositoryError> {
        let node = accelerator_node::Entity::find_by_id(node_id.to_string())
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?
            .ok_or_else(|| RepositoryError::Persistence("node not found".into()))?;

        let mut active: accelerator_node::ActiveModel = node.into();
        active.last_heartbeat = Set(Utc::now().into());
        active
            .update(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(())
    }

    async fn get_node(&self, node_id: &str) -> Result<Option<Node>, RepositoryError> {
        let model = accelerator_node::Entity::find_by_id(node_id.to_string())
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(model.map(Into::into))
    }

    async fn list_nodes(&self) -> Result<Vec<Node>, RepositoryError> {
        let models = accelerator_node::Entity::find()
            .all(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(models.into_iter().map(Into::into).collect())
    }

    async fn list_active_nodes(&self) -> Result<Vec<Node>, RepositoryError> {
        let threshold = Utc::now() - Duration::minutes(5);
        let models = accelerator_node::Entity::find()
            .filter(accelerator_node::Column::LastHeartbeat.gte(threshold))
            .filter(accelerator_node::Column::Status.eq("active"))
            .all(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(models.into_iter().map(Into::into).collect())
    }
}

fn node_into_active_model(node: Node) -> accelerator_node::ActiveModel {
    accelerator_node::ActiveModel {
        id: Set(node.id),
        vmess_uuid: Set(node.vmess_uuid),
        vmess_server: Set(node.vmess_server),
        vmess_port: Set(node.vmess_port),
        vmess_email: Set(node.vmess_email),
        udp_proxy: Set(node.udp_proxy),
        mode: Set(node.mode),
        ping: Set(node.ping),
        status: Set(node.status),
        last_heartbeat: Set(node.last_heartbeat.into()),
    }
}

pub struct ConfigRepositoryImpl<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> ConfigRepositoryImpl<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl<'a> ConfigRepository for ConfigRepositoryImpl<'a> {
    async fn get_entry(&self, key: &str) -> Result<Option<JsonValue>, RepositoryError> {
        let entry = config_entry::Entity::find_by_id(key.to_string())
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(entry.map(|m| m.payload))
    }
}

pub struct AuthRepositoryImpl<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> AuthRepositoryImpl<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl<'a> AuthRepository for AuthRepositoryImpl<'a> {
    async fn insert_ticket(&self, ticket: &WechatTicket) -> Result<(), RepositoryError> {
        info!(
            "Inserting ticket {} with status {}",
            ticket.ticket_id,
            ticket.status.as_str()
        );
        let active = ticket_into_active(ticket);
        active
            .insert(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(())
    }

    async fn get_ticket(&self, ticket_id: &str) -> Result<Option<WechatTicket>, RepositoryError> {
        let model = wechat_ticket::Entity::find_by_id(ticket_id.to_string())
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;

        if let Some(model) = model {
            let user = match &model.user_id {
                Some(user_id) => accelerator_user::Entity::find_by_id(user_id.clone())
                    .one(self.db)
                    .await
                    .map_err(|err| RepositoryError::Persistence(err.to_string()))?
                    .map(Into::into),
                None => None,
            };
            let mut ticket = ticket_from_model(model);
            ticket.user = user;
            Ok(Some(ticket))
        } else {
            Ok(None)
        }
    }

    async fn save_ticket(&self, ticket: &WechatTicket) -> Result<(), RepositoryError> {
        let existing = wechat_ticket::Entity::find_by_id(ticket.ticket_id.clone())
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?
            .ok_or_else(|| {
                warn!(
                    "Attempted to update ticket {} but it was not found",
                    ticket.ticket_id
                );
                RepositoryError::Persistence("ticket not found".into())
            })?;

        let expires_at = Utc::now() + Duration::seconds(ticket.expires_in);
        let mut active: wechat_ticket::ActiveModel = existing.into();
        active.qr_code_url = Set(ticket.qr_code_url.clone());
        active.expires_at = Set(expires_at.into());
        active.status = Set(ticket.status.as_str().to_string());
        active.scene = Set(ticket.scene.clone());
        active.success = Set(ticket.success);
        active.user_id = Set(ticket.user.as_ref().map(|u| u.id.clone()));
        active.updated_at = Set(Utc::now().into());

        info!(
            "Updating ticket {} to status {} (expires in {}s)",
            ticket.ticket_id,
            ticket.status.as_str(),
            ticket.expires_in
        );
        active
            .update(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(())
    }

    async fn login_account(
        &self,
        request: AccountLoginRequest,
    ) -> Result<AccountLoginResponse, RepositoryError> {
        let model = account_user::Entity::find()
            .filter(account_user::Column::Phone.eq(request.phone))
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?
            .ok_or_else(|| RepositoryError::Persistence("invalid credentials".into()))?;

        let hashed = hash_password(&request.password);
        if hashed != model.password_hash {
            return Err(RepositoryError::Persistence("invalid credentials".into()));
        }

        let user: AcceleratorUser = model.clone().into();
        let response = AccountLoginResponse {
            success: true,
            token: format!("mock-token-{}", user.id),
            user,
        };
        Ok(response)
    }

    async fn get_user_by_id(&self, user_id: &str) -> Result<Option<AcceleratorUser>, RepositoryError> {
        // 先尝试从 accelerator_users 表查找
        let user = accelerator_user::Entity::find_by_id(user_id.to_string())
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        
        if let Some(user) = user {
            return Ok(Some(user.into()));
        }

        // 如果不存在，从 account_users 表查找
        let account = account_user::Entity::find_by_id(user_id.to_string())
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        
        Ok(account.map(Into::into))
    }

    async fn update_user_valid_until(
        &self,
        user_id: &str,
        valid_until: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RepositoryError> {
        // 先尝试更新 accelerator_users
        let user = accelerator_user::Entity::find_by_id(user_id.to_string())
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;

        if let Some(user) = user {
            let mut active: accelerator_user::ActiveModel = user.into();
            active.valid_until = Set(valid_until.into());
            active
                .update(self.db)
                .await
                .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
            return Ok(());
        }

        // 如果不存在，更新 account_users
        let account = account_user::Entity::find_by_id(user_id.to_string())
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?
            .ok_or_else(|| RepositoryError::Persistence("user not found".into()))?;

        let mut active: account_user::ActiveModel = account.into();
        active.valid_until = Set(valid_until.into());
        active
            .update(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(())
    }

    async fn get_remaining_minutes(&self, user_id: &str) -> Result<i64, RepositoryError> {
        let wallet = user_wallet::Entity::find_by_id(user_id.to_string())
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(wallet.map(|w| w.remaining_minutes).unwrap_or(0))
    }

    async fn add_remaining_minutes(&self, user_id: &str, minutes: i64) -> Result<i64, RepositoryError> {
        if minutes <= 0 {
            return Err(RepositoryError::Persistence("minutes must be positive".into()));
        }

        let existing = user_wallet::Entity::find_by_id(user_id.to_string())
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;

        match existing {
            Some(model) => {
                let current = model.remaining_minutes;
                let mut active: user_wallet::ActiveModel = model.into();
                let next = current + minutes;
                active.remaining_minutes = Set(next);
                active.updated_at = Set(Utc::now().into());
                active
                    .update(self.db)
                    .await
                    .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
                Ok(next)
            }
            None => {
                let active = user_wallet::ActiveModel {
                    user_id: Set(user_id.to_string()),
                    remaining_minutes: Set(minutes),
                    updated_at: Set(Utc::now().into()),
                };
                active
                    .insert(self.db)
                    .await
                    .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
                Ok(minutes)
            }
        }
    }
}

fn ticket_into_active(ticket: &WechatTicket) -> wechat_ticket::ActiveModel {
    let expires_at = Utc::now() + Duration::seconds(ticket.expires_in);
    wechat_ticket::ActiveModel {
        ticket_id: Set(ticket.ticket_id.clone()),
        qr_code_url: Set(ticket.qr_code_url.clone()),
        expires_at: Set(expires_at.into()),
        status: Set(ticket.status.as_str().to_string()),
        scene: Set(ticket.scene.clone()),
        success: Set(ticket.success),
        user_id: Set(ticket.user.as_ref().map(|u| u.id.clone())),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
}

fn ticket_from_model(model: wechat_ticket::Model) -> WechatTicket {
    let expires_in = (model.expires_at - Utc::now()).num_seconds().max(0);
    WechatTicket {
        ticket_id: model.ticket_id,
        qr_code_url: model.qr_code_url,
        expires_in,
        status: TicketStatus::from_str(&model.status),
        scene: model.scene,
        success: model.success,
        user: None,
    }
}

fn hash_password(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

impl From<account_user::Model> for AcceleratorUser {
    fn from(model: account_user::Model) -> Self {
        AcceleratorUser {
            id: model.id,
            name: model.name,
            valid_until: model.valid_until.into(),
        }
    }
}

pub struct CdkRepositoryImpl<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> CdkRepositoryImpl<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl<'a> CdkRepository for CdkRepositoryImpl<'a> {
    async fn generate_cdks(&self, request: CdkGenerateRequest) -> Result<Vec<CdkCode>, RepositoryError> {
        use uuid::Uuid;
        let mut cdks = Vec::new();
        let duration = if request.cdk_type == CdkType::Minute {
            request.duration_minutes.unwrap_or(0)
        } else {
            request.cdk_type.duration_minutes()
        };

        for _ in 0..request.count {
            let id = Uuid::new_v4().to_string();
            let code = generate_cdk_code();
            let cdk = CdkCode {
                id: id.clone(),
                code: code.clone(),
                cdk_type: request.cdk_type.clone(),
                duration_minutes: duration,
                status: CdkStatus::Unused,
                used_by: None,
                used_at: None,
                expires_at: request.expires_at,
                created_at: Utc::now(),
            };

            let active = cdk_into_active_model(cdk.clone());
            active
                .insert(self.db)
                .await
                .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
            cdks.push(cdk);
        }

        info!("Generated {} CDK codes", cdks.len());
        Ok(cdks)
    }

    async fn get_cdk_by_code(&self, code: &str) -> Result<Option<CdkCode>, RepositoryError> {
        let model = cdk_code::Entity::find()
            .filter(cdk_code::Column::Code.eq(code))
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(model.map(Into::into))
    }

    async fn redeem_cdk(&self, request: CdkRedeemRequest) -> Result<CdkRedeemResponse, RepositoryError> {
        let model = cdk_code::Entity::find()
            .filter(cdk_code::Column::Code.eq(&request.code))
            .one(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?
            .ok_or_else(|| RepositoryError::Persistence("CDK not found".into()))?;

        let cdk: CdkCode = model.clone().into();

        // 检查CDK状态
        if cdk.status != CdkStatus::Unused {
            return Err(RepositoryError::Persistence("CDK already used or expired".into()));
        }

        // 检查CDK是否过期
        if let Some(expires_at) = cdk.expires_at {
            if expires_at < Utc::now() {
                return Err(RepositoryError::Persistence("CDK expired".into()));
            }
        }

        // 更新CDK状态
        let mut active: cdk_code::ActiveModel = model.into();
        active.status = Set(CdkStatus::Used.as_str().to_string());
        active.used_by = Set(Some(request.user_id.clone()));
        active.used_at = Set(Some(Utc::now().into()));
        active
            .update(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;

        // 计算新的有效期
        let now = Utc::now();
        let valid_until = now + chrono::Duration::minutes(cdk.duration_minutes);

        Ok(CdkRedeemResponse {
            success: true,
            message: "CDK redeemed successfully".into(),
            duration_minutes: cdk.duration_minutes,
            valid_until: Some(valid_until),
            remaining_minutes: None,
        })
    }

    async fn list_cdks(&self, status: Option<&str>) -> Result<Vec<CdkCode>, RepositoryError> {
        let mut query = cdk_code::Entity::find();
        if let Some(status_str) = status {
            query = query.filter(cdk_code::Column::Status.eq(status_str));
        }
        let models = query
            .all(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(models.into_iter().map(Into::into).collect())
    }
}

fn cdk_into_active_model(cdk: CdkCode) -> cdk_code::ActiveModel {
    cdk_code::ActiveModel {
        id: Set(cdk.id),
        code: Set(cdk.code),
        cdk_type: Set(cdk.cdk_type.as_str().to_string()),
        duration_minutes: Set(cdk.duration_minutes),
        status: Set(cdk.status.as_str().to_string()),
        used_by: Set(cdk.used_by),
        used_at: Set(cdk.used_at.map(Into::into)),
        expires_at: Set(cdk.expires_at.map(Into::into)),
        created_at: Set(cdk.created_at.into()),
    }
}

fn generate_cdk_code() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // 排除容易混淆的字符
    const CODE_LENGTH: usize = 12;
    
    let mut rng = rand::thread_rng();
    let code: String = (0..CODE_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    
    // 格式化为 XXXX-XXXX-XXXX
    format!("{}-{}-{}", &code[0..4], &code[4..8], &code[8..12])
}
