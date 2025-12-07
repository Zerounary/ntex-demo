use sea_orm::entity::prelude::*;
use struct_convert::Convert;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Convert)]
#[sea_orm(table_name = "accelerator_profiles")]
#[convert(into = "crate::domain::accelerator::Profile")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(Some(128))")]
    pub id: String,
    #[sea_orm(column_type = "String(Some(64))")]
    pub game_id: String,
    pub display_name: String,
    pub process_name: String,
    pub vmess_uuid: String,
    pub vmess_server: String,
    pub vmess_port: i32,
    pub vmess_email: String,
    pub udp_proxy: String,
    pub mode: String,
    pub status: String,
    pub region: String,
    pub ping: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
