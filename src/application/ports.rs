use async_trait::async_trait;
use serde_json::Value;

use crate::domain::accelerator::{AcceleratorUser, BootstrapPayload, Game, Node, Profile};
use crate::domain::auth::{AccountLoginRequest, AccountLoginResponse, WechatTicket};
use crate::domain::cdk::{CdkCode, CdkGenerateRequest, CdkRedeemRequest, CdkRedeemResponse};

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
pub trait NodeRepository: Send + Sync {
    async fn register_node(&self, node: Node) -> Result<(), RepositoryError>;
    async fn unregister_node(&self, node_id: &str) -> Result<(), RepositoryError>;
    async fn update_node_heartbeat(&self, node_id: &str) -> Result<(), RepositoryError>;
    async fn get_node(&self, node_id: &str) -> Result<Option<Node>, RepositoryError>;
    async fn list_nodes(&self) -> Result<Vec<Node>, RepositoryError>;
    async fn list_active_nodes(&self) -> Result<Vec<Node>, RepositoryError>;
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
    async fn get_user_by_id(&self, user_id: i64) -> Result<Option<AcceleratorUser>, RepositoryError>;
    async fn update_user_valid_until(
        &self,
        user_id: i64,
        valid_until: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RepositoryError>;

    async fn get_remaining_minutes(&self, user_id: i64) -> Result<i64, RepositoryError>;
    async fn add_remaining_minutes(&self, user_id: i64, minutes: i64) -> Result<i64, RepositoryError>;
    async fn get_bandwidth_mbps(&self, user_id: i64) -> Result<Option<i64>, RepositoryError>;
    async fn set_bandwidth_mbps(&self, user_id: i64, bandwidth_mbps: i64) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait CdkRepository: Send + Sync {
    async fn generate_cdks(&self, request: CdkGenerateRequest) -> Result<Vec<CdkCode>, RepositoryError>;
    async fn get_cdk_by_code(&self, code: &str) -> Result<Option<CdkCode>, RepositoryError>;
    async fn redeem_cdk(&self, request: CdkRedeemRequest) -> Result<CdkRedeemResponse, RepositoryError>;
    async fn list_cdks(&self, status: Option<&str>) -> Result<Vec<CdkCode>, RepositoryError>;
}
