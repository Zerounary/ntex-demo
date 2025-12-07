use async_trait::async_trait;
use serde_json::Value;

use crate::domain::accelerator::{AcceleratorUser, BootstrapPayload, Game, Profile};
use crate::domain::auth::{AccountLoginRequest, AccountLoginResponse, WechatTicket};

use super::errors::RepositoryError;

#[async_trait]
pub trait AcceleratorRepository: Send + Sync {
    async fn list_games(&self) -> Result<Vec<Game>, RepositoryError>;
    async fn list_profiles(&self) -> Result<Vec<Profile>, RepositoryError>;
    async fn upsert_profiles(&self, profiles: Vec<Profile>) -> Result<(), RepositoryError>;
    async fn current_user(&self) -> Result<Option<AcceleratorUser>, RepositoryError>;
    async fn bootstrap(&self) -> Result<BootstrapPayload, RepositoryError>;
}

#[async_trait]
pub trait ConfigRepository: Send + Sync {
    async fn get_entry(&self, key: &str) -> Result<Option<Value>, RepositoryError>;
}

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn insert_ticket(&self, ticket: &WechatTicket) -> Result<(), RepositoryError>;
    async fn get_ticket(&self, ticket_id: &str) -> Result<Option<WechatTicket>, RepositoryError>;
    async fn save_ticket(&self, ticket: &WechatTicket) -> Result<(), RepositoryError>;
    async fn login_account(
        &self,
        request: AccountLoginRequest,
    ) -> Result<AccountLoginResponse, RepositoryError>;
}
