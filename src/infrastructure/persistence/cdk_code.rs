use sea_orm::{entity::prelude::*, sea_query::Expr};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cdk_codes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "String(Some(128))")]
    pub id: String,
    #[sea_orm(unique, column_type = "String(Some(64))")]
    pub code: String,
    #[sea_orm(column_type = "String(Some(16))")]
    pub cdk_type: String,
    pub duration_minutes: i64,
    #[sea_orm(column_type = "String(Some(16))")]
    pub status: String,
    #[sea_orm(nullable, column_type = "String(Some(64))")]
    pub used_by: Option<String>,
    #[sea_orm(nullable, column_type = "DateTime")]
    pub used_at: Option<DateTimeUtc>,
    #[sea_orm(nullable, column_type = "DateTime")]
    pub expires_at: Option<DateTimeUtc>,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for crate::domain::cdk::CdkCode {
    fn from(model: Model) -> Self {
        use crate::domain::cdk::{CdkStatus, CdkType};
        Self {
            id: model.id,
            code: model.code,
            cdk_type: CdkType::from_str(&model.cdk_type).unwrap_or(CdkType::Day),
            duration_minutes: model.duration_minutes,
            status: CdkStatus::from_str(&model.status),
            used_by: model.used_by,
            used_at: model.used_at.map(Into::into),
            expires_at: model.expires_at.map(Into::into),
            created_at: model.created_at.into(),
        }
    }
}

