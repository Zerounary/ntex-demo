use sea_orm::{entity::prelude::*, sea_query::Expr};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user_wallets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "String(Some(64))")]
    pub user_id: String,
    pub remaining_minutes: i64,
    #[sea_orm(nullable)]
    pub bandwidth: Option<i64>,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
