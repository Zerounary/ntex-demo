use sea_orm::entity::prelude::*;
use struct_convert::Convert;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Convert)]
#[sea_orm(table_name = "account_users")]
#[convert(into = "crate::domain::auth::AccountUser")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(Some(64))")]
    pub id: String,
    #[sea_orm(column_type = "String(Some(32))")]
    pub phone: String,
    pub password_hash: String,
    pub name: String,
    pub valid_until: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
