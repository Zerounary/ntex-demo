use sea_orm::entity::prelude::*;
use struct_convert::Convert;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Convert)]
#[sea_orm(table_name = "todos")]
#[convert(into = "crate::domain::todo::Todo")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub title: String,
    #[sea_orm(default_value = false)]
    pub completed: bool,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

