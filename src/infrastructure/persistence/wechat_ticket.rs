use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "wechat_tickets")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(Some(64))")]
    pub ticket_id: String,
    pub qr_code_url: String,
    pub expires_at: DateTimeUtc,
    pub status: String,
    pub scene: String,
    pub success: bool,
    pub user_id: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
