use sea_orm::{entity::prelude::*, JsonValue, sea_query::Expr};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "admin_chains")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub protocol: String,
    pub chain_type: String,
    pub routes: JsonValue,
    pub description: Option<String>,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
