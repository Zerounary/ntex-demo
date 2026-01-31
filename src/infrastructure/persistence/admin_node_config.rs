use sea_orm::{entity::prelude::*, JsonValue, sea_query::Expr};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "admin_node_configs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub node_id: u64,
    #[sea_orm(nullable)]
    pub name: Option<String>,
    #[sea_orm(nullable)]
    pub region: Option<String>,
    #[sea_orm(nullable, column_type = "Text")]
    pub description: Option<String>,
    pub node_type: String,
    pub node_speed_limit: u64,
    pub traffic_rate: f64,
    pub sort: u64,
    #[sea_orm(default_value = "0")]
    pub maintenance_mode: bool,
    #[sea_orm(default_value = "0")]
    pub is_online: bool,
    #[sea_orm(nullable, column_type = "Timestamp")]
    pub last_seen_at: Option<DateTimeUtc>,
    // 实时状态字段
    #[sea_orm(nullable)]
    pub cpu_usage: Option<f64>,  // CPU 使用率，0.0-1.0（0.9 表示 90%）
    #[sea_orm(nullable)]
    pub mem_usage: Option<f64>,  // 内存使用率，0.0-1.0（0.6 表示 60%）
    #[sea_orm(nullable)]
    pub disk_usage: Option<f64>, // 磁盘使用率，0.0-1.0（0.4 表示 40%）
    #[sea_orm(nullable)]
    pub uptime: Option<u64>,        // 运行时间（秒）
    #[sea_orm(nullable)]
    pub online_user_count: Option<u64>, // 在线用户数
    // 节点硬件信息
    #[sea_orm(nullable)]
    pub cpu_threads: Option<u32>,  // CPU 线程数
    #[sea_orm(nullable)]
    pub mem_total: Option<u64>,    // 内存总容量（字节）
    #[sea_orm(nullable)]
    pub disk_total: Option<u64>,   // 磁盘总容量（字节）
    #[sea_orm(nullable)]
    pub public_ip: Option<String>, // 节点公网 IP
    // 网络接口信息（JSON 格式，存储各网卡的带宽使用情况）
    #[sea_orm(nullable, column_type = "Json")]
    pub network_interfaces: Option<JsonValue>,  // 网络接口信息数组
    #[sea_orm(nullable)]
    pub node_token: Option<String>,
    #[sea_orm(nullable)]
    pub node_shared_secret: Option<String>,
    #[sea_orm(default_value = "mqtt")]
    pub node_comm_mode: String,
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

