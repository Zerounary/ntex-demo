use sea_orm::entity::prelude::*;
use struct_convert::Convert;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Convert)]
#[sea_orm(table_name = "accelerator_profiles")]
#[convert(into = "crate::domain::accelerator::Profile")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "String(Some(128))")]
    pub id: String,
    #[sea_orm(column_type = "String(Some(64))")]
    pub game_id: String,
    pub display_name: String,
    #[sea_orm(column_type = "String(Some(128))")]
    pub node_id: String,
    pub status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
