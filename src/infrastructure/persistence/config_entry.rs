use sea_orm::{JsonValue, entity::prelude::*};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "config_entries")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(Some(64))")]
    pub key: String,
    pub payload: JsonValue,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
