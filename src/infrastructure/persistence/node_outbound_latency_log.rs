use sea_orm::{entity::prelude::*, sea_query::Expr};

/// Outbound 延迟上报历史记录
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "node_outbound_latency_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u64,
    pub node_id: u64,
    pub outbound_tag: String,
    pub latency_ms: f64,    // 延迟（毫秒）
    pub probe_type: String, // "tcp" 或 "udp"
    #[sea_orm(nullable)]
    pub error_message: Option<String>,  // 探测失败时的错误信息
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

