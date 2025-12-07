use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::accelerator::AcceleratorUser;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WechatTicket {
    pub ticket_id: String,
    pub qr_code_url: String,
    pub expires_in: i64,
    pub status: TicketStatus,
    pub scene: String,
    pub success: bool,
    pub user: Option<AcceleratorUser>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TicketStatus {
    Pending,
    Scanned,
    Confirmed,
    Expired,
}

impl TicketStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TicketStatus::Pending => "pending",
            TicketStatus::Scanned => "scanned",
            TicketStatus::Confirmed => "confirmed",
            TicketStatus::Expired => "expired",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "scanned" => TicketStatus::Scanned,
            "confirmed" => TicketStatus::Confirmed,
            "expired" => TicketStatus::Expired,
            _ => TicketStatus::Pending,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountLoginRequest {
    pub phone: String,
    pub password: String,
    pub remember: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountLoginResponse {
    pub success: bool,
    pub token: String,
    pub user: AcceleratorUser,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountUser {
    pub id: String,
    pub phone: String,
    pub password_hash: String,
    pub name: String,
    pub valid_until: DateTime<Utc>,
}
