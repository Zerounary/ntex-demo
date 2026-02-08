use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "accelerator_users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column_type = "String(Some(128))")]
    pub email: String,
    pub name: String,
    #[sea_orm(column_type = "String(Some(32))")]
    pub invite_code: String,
    #[sea_orm(nullable)]
    pub inviter_id: Option<i64>,
    pub valid_until: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for crate::domain::accelerator::AcceleratorUser {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            email: model.email,
            name: model.name,
            valid_until: model.valid_until.into(),
        }
    }
}
