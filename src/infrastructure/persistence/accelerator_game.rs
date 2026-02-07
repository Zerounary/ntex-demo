use sea_orm::entity::prelude::*;
use crate::domain::accelerator::Game;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "accelerator_games")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "String(Some(64))")]
    pub id: String,
    pub name: String,
    pub icon: String,
    pub status: String,
    pub ping: i32,
    #[sea_orm(column_type = "Text")]
    pub process_name: String,
    #[sea_orm(column_type = "Text")]
    pub routing_rules: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub sniff_domains_excluded: Option<String>,
    pub region: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for Game {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            icon: m.icon,
            status: m.status,
            ping: m.ping,
            process_name: m.process_name,
            routing_rules: m.routing_rules.unwrap_or_else(|| "[]".to_string()),
            sniff_domains_excluded: m
                .sniff_domains_excluded
                .unwrap_or_else(|| "[]".to_string()),
            region: m.region,
        }
    }
}
