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
    AcceleratorRepository, AuthRepository, ConfigRepository, NodeRepository,
};
use crate::domain::accelerator::{AcceleratorUser, BootstrapPayload, Game, Node, Profile};
use crate::domain::auth::{AccountLoginRequest, AccountLoginResponse, TicketStatus, WechatTicket};
use log::{info, warn};

use super::accelerator_game;
use super::accelerator_node;
use super::accelerator_profile;
use super::accelerator_user;
use super::account_user;
use super::config_entry;
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
        let profiles = self.list_profiles().await?;
        let user = self.current_user().await?;
        
        // 获取所有节点
        let node_repo = NodeRepositoryImpl::new(self.db);
        let nodes = node_repo.list_nodes().await?;
        
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
