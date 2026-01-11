use sea_orm::{entity::prelude::*, sea_query::Expr};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "accelerator_game_node_bindings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column_type = "String(Some(64))")]
    pub game_id: String,
    #[sea_orm(column_type = "String(Some(16))")]
    pub r#type: String,
    #[sea_orm(column_type = "String(Some(128))")]
    pub node_id: Option<String>,
    pub tcp_chain_id: Option<i64>,
    pub udp_chain_id: Option<i64>,
    pub display_name: Option<String>,
    pub region: Option<String>,
    pub mode: Option<String>,
    pub ping: Option<i32>,
    pub status: Option<String>,
    pub remark: Option<String>,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
