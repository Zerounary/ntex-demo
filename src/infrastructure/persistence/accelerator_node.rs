use sea_orm::{entity::prelude::*, sea_query::Expr};
use struct_convert::Convert;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Convert)]
#[sea_orm(table_name = "accelerator_nodes")]
#[convert(into = "crate::domain::accelerator::Node")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "String(Some(128))")]
    pub id: String,
    pub vless_id: String,
    pub vless_server: String,
    pub vless_port: i32,
    pub vless_encryption: String,
    pub reality_server_name: String,
    pub reality_public_key: String,
    pub reality_short_id: String,
    pub reality_fingerprint: String,
    pub reality_spider_x: String,
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

