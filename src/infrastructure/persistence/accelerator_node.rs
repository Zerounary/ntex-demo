use sea_orm::{entity::prelude::*, sea_query::Expr};
use struct_convert::Convert;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Convert)]
#[sea_orm(table_name = "accelerator_nodes")]
#[convert(into = "crate::domain::accelerator::Node")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "String(Some(128))")]
    pub id: String,
    pub vmess_uuid: String,
    pub vmess_server: String,
    pub vmess_port: i32,
    pub vmess_email: String,
    pub udp_proxy: String,
    pub mode: String,
    pub ping: i32,
    pub status: String,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub last_heartbeat: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

