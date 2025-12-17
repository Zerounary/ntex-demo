use sea_orm::{entity::prelude::*, JsonValue, sea_query::Expr};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "admin_node_configs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub node_id: u64,
    pub node_type: String,
    pub node_speed_limit: u64,
    pub traffic_rate: f64,
    pub sort: u64,
    pub inbounds: JsonValue,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
    #[sea_orm(column_type = "Timestamp", default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::admin_user::Entity")]
    Users,
    #[sea_orm(has_many = "super::admin_outbound::Entity")]
    Outbounds,
    #[sea_orm(has_one = "super::admin_routing::Entity")]
    Routing,
}

impl Related<super::admin_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::admin_outbound::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Outbounds.def()
    }
}

impl Related<super::admin_routing::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Routing.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

