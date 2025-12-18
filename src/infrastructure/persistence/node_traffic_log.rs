use sea_orm::{entity::prelude::*, sea_query::Expr};

/// 节点流量上报历史记录
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "node_traffic_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u64,
    pub node_id: u64,
    pub user_id: u64,      // 用户 ID
    pub upload: u64,       // 上传流量（字节）
    pub download: u64,     // 下载流量（字节）
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

