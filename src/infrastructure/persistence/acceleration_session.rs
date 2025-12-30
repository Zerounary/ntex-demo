use sea_orm::{entity::prelude::*, sea_query::Expr};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "acceleration_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "String(Some(64))")]
    pub session_id: String,
    #[sea_orm(column_type = "String(Some(64))")]
    pub user_id: String,
    #[sea_orm(column_type = "String(Some(64))")]
    pub game_id: String,
    pub node_id: u64,
    pub admin_user_id: u64,
    pub uuid: String,
    pub outbound_tag: String,
    #[sea_orm(column_type = "String(Some(32))")]
    pub status: String,
    #[sea_orm(column_type = "String(Some(16))")]
    pub bill_type: String,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub started_at: DateTimeUtc,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub last_activity_at: DateTimeUtc,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub last_accounted_at: DateTimeUtc,
    pub billed_minutes: i64,
    #[sea_orm(nullable, column_type = "DateTime", default_expr = "Expr::cust(\"NULL\")")]
    pub ended_at: Option<DateTimeUtc>,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
