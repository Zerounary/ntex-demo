use async_trait::async_trait;
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, JsonValue, QueryFilter, Set,
};
use sha2::{Digest, Sha256};

use crate::application::errors::RepositoryError;
use crate::application::ports::{AcceleratorRepository, AuthRepository, ConfigRepository};
use crate::domain::accelerator::{AcceleratorUser, BootstrapPayload, Game, Profile};
use crate::domain::auth::{AccountLoginRequest, AccountLoginResponse, TicketStatus, WechatTicket};

use super::accelerator_game;
use super::accelerator_profile;
use super::accelerator_user;
use super::account_user;
use super::config_entry;
use super::wechat_ticket;

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
        let models = accelerator_game::Entity::find()
            .all(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(models.into_iter().map(Into::into).collect())
    }

    async fn list_profiles(&self) -> Result<Vec<Profile>, RepositoryError> {
        let models = accelerator_profile::Entity::find()
            .all(self.db)
            .await
            .map_err(|err| RepositoryError::Persistence(err.to_string()))?;
        Ok(models.into_iter().map(Into::into).collect())
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
        let games = self.list_games().await?;
        let profiles = self.list_profiles().await?;
        let user = self.current_user().await?;
        Ok(BootstrapPayload {
            games,
            profiles,
            user,
        })
    }
}

fn profile_into_active_model(profile: Profile) -> accelerator_profile::ActiveModel {
    accelerator_profile::ActiveModel {
        id: Set(profile.id),
        game_id: Set(profile.game_id),
        display_name: Set(profile.display_name),
        process_name: Set(profile.process_name),
        vmess_uuid: Set(profile.vmess_uuid),
        vmess_server: Set(profile.vmess_server),
        vmess_port: Set(profile.vmess_port),
        vmess_email: Set(profile.vmess_email),
        udp_proxy: Set(profile.udp_proxy),
        mode: Set(profile.mode),
        status: Set(profile.status),
        region: Set(profile.region),
        ping: Set(profile.ping),
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
        let active = ticket_into_active(ticket);
        active
            .save(self.db)
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
