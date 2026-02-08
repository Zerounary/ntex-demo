use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CdkType {
    Day,     // 日卡
    Month,   // 月卡
    Year,    // 年卡
    Minute,  // 分钟卡
    Bandwidth, // 带宽卡（设置带宽，单位 Mbps）
}

impl CdkType {
    pub fn duration_minutes(&self) -> i64 {
        match self {
            CdkType::Day => 24 * 60,           // 1天 = 1440分钟
            CdkType::Month => 30 * 24 * 60,    // 30天 = 43200分钟
            CdkType::Year => 365 * 24 * 60,   // 365天 = 525600分钟
            CdkType::Minute => 0,              // 需要指定分钟数
            CdkType::Bandwidth => 0,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CdkType::Day => "day",
            CdkType::Month => "month",
            CdkType::Year => "year",
            CdkType::Minute => "minute",
            CdkType::Bandwidth => "bandwidth",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "day" => Some(CdkType::Day),
            "month" => Some(CdkType::Month),
            "year" => Some(CdkType::Year),
            "minute" => Some(CdkType::Minute),
            "bandwidth" => Some(CdkType::Bandwidth),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CdkStatus {
    Unused,  // 未使用
    Used,    // 已使用
    Expired, // 已过期
}

impl CdkStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CdkStatus::Unused => "unused",
            CdkStatus::Used => "used",
            CdkStatus::Expired => "expired",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "used" => CdkStatus::Used,
            "expired" => CdkStatus::Expired,
            _ => CdkStatus::Unused,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CdkCode {
    pub id: String,
    pub code: String,
    pub cdk_type: CdkType,
    pub num: i64, // 数量：day/月/年=张数单位数量；minute=分钟数；bandwidth=Mbps
    pub bandwidth_mbps: Option<i64>,
    pub status: CdkStatus,
    pub used_by: Option<String>, // 使用用户ID
    pub used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>, // CDK本身的过期时间（如果未使用）
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CdkGenerateRequest {
    pub cdk_type: CdkType,
    pub count: u32,
    pub num: Option<i64>,
    pub bandwidth_mbps: Option<i64>, // 兼容字段：带宽卡可用 num 或 bandwidth_mbps
    pub expires_at: Option<DateTime<Utc>>, // CDK过期时间
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CdkRedeemRequest {
    pub code: String,
    pub user_id: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CdkRedeemResponse {
    pub success: bool,
    pub message: String,
    pub cdk_type: CdkType,
    pub num: i64,
    pub valid_until: Option<DateTime<Utc>>,
    pub remaining_minutes: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountValidationRequest {
    pub user_id: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountValidationResponse {
    pub is_valid: bool,
    pub is_paid: bool,
    pub valid_until: Option<DateTime<Utc>>,
    pub remaining_minutes: i64,
    pub billing_mode: String,
    pub message: String,
}




