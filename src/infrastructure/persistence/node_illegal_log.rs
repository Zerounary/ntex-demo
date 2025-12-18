use sea_orm::{entity::prelude::*, sea_query::Expr};

/// 非法行为上报历史记录
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "node_illegal_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u64,
    pub node_id: u64,
    pub user_id: u64,      // 用户 ID
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::admin_node_config::Entity",
        from = "Column::NodeId",
        to = "super::admin_node_config::Column::NodeId"
    )]
    NodeConfig,
}

impl Related<super::admin_node_config::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeConfig.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

