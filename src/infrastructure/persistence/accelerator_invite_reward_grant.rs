use sea_orm::{entity::prelude::*, sea_query::Expr};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "accelerator_invite_reward_grants")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column_type = "String(Some(64))")]
    pub inviter_id: String,
    #[sea_orm(column_type = "String(Some(64))")]
    pub invitee_id: String,
    pub tier: i32,
    #[sea_orm(column_type = "String(Some(16))")]
    pub cdk_type: String,
    pub num: i64,
    pub bandwidth_mbps: Option<i64>,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub granted_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
