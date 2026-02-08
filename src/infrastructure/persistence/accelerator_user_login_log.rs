use sea_orm::{entity::prelude::*, sea_query::Expr};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "accelerator_user_login_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u64,
    pub user_id: i64,
    #[sea_orm(column_type = "String(Some(64))")]
    pub ip: String,
    #[sea_orm(column_type = "String(Some(255))")]
    pub user_agent: String,
    pub success: bool,
    #[sea_orm(column_type = "String(Some(64))")]
    pub reason_code: String,
    #[sea_orm(column_type = "String(Some(255))")]
    pub reason_message: String,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
