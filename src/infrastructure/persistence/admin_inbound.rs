use sea_orm::{entity::prelude::*, JsonValue, sea_query::Expr};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "admin_inbounds")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub node_id: u64,
    pub tag: String,
    pub protocol: String,
    pub port: i32,
    #[sea_orm(nullable)]
    pub listen: Option<String>,
    pub settings: JsonValue,
    #[sea_orm(nullable)]
    pub stream_settings: Option<JsonValue>,
    #[sea_orm(nullable)]
    pub sniffing: Option<JsonValue>,
    #[sea_orm(
        column_type = "Timestamp",
        default_expr = "Expr::current_timestamp()"
    )]
    pub created_at: DateTimeUtc,
    #[sea_orm(
        column_type = "Timestamp",
        default_expr = "Expr::current_timestamp()"
    )]
    pub updated_at: DateTimeUtc,
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
