use sea_orm::{entity::prelude::*, sea_query::Expr};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "accelerator_user_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "String(Some(64))")]
    pub token: String,
    pub user_id: i64,
    pub expires_at: DateTimeUtc,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
