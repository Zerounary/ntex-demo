use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Schema};

use crate::infrastructure::persistence::{
    accelerator_game, accelerator_node, accelerator_profile, accelerator_user, account_user,
    admin_node_config, admin_outbound, admin_routing, admin_user, admin_user_mapping,
    cdk_code, config_entry, wechat_ticket,
};

pub async fn connect(url: &str) -> Result<DatabaseConnection, DbErr> {
    Database::connect(url).await
}

pub async fn init(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);

    for table in [
        schema.create_table_from_entity(accelerator_game::Entity),
        schema.create_table_from_entity(accelerator_node::Entity),
        schema.create_table_from_entity(accelerator_profile::Entity),
        schema.create_table_from_entity(accelerator_user::Entity),
        schema.create_table_from_entity(account_user::Entity),
        schema.create_table_from_entity(admin_node_config::Entity),
        schema.create_table_from_entity(admin_user::Entity),
        schema.create_table_from_entity(admin_outbound::Entity),
        schema.create_table_from_entity(admin_routing::Entity),
        schema.create_table_from_entity(admin_user_mapping::Entity),
        schema.create_table_from_entity(cdk_code::Entity),
        schema.create_table_from_entity(config_entry::Entity),
        schema.create_table_from_entity(wechat_ticket::Entity),
    ] {
        let mut stmt = table;
        stmt.if_not_exists();
        db.execute(backend.build(&stmt)).await?;
    }

    // 迁移：检查并添加缺失的字段
    migrate_add_process_name_field(db).await?;

    Ok(())
}

/// 迁移：为 accelerator_games 表添加 process_name 字段（如果不存在）
async fn migrate_add_process_name_field(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    
    // 添加 process_name 字段（如果已存在会失败，但我们可以忽略）
    let alter_sql = match backend {
        sea_orm::DatabaseBackend::MySql => {
            "ALTER TABLE accelerator_games ADD COLUMN IF NOT EXISTS process_name VARCHAR(255) NOT NULL DEFAULT ''"
        }
        sea_orm::DatabaseBackend::Postgres => {
            "ALTER TABLE accelerator_games ADD COLUMN IF NOT EXISTS process_name VARCHAR(255) NOT NULL DEFAULT ''"
        }
        sea_orm::DatabaseBackend::Sqlite => {
            // SQLite 不支持 IF NOT EXISTS 和直接添加 NOT NULL 列
            // 对于 SQLite，如果表已存在，需要重建表，这里简化处理：跳过
            return Ok(());
        }
    };

    if matches!(backend, sea_orm::DatabaseBackend::MySql | sea_orm::DatabaseBackend::Postgres) {
        // MySQL 5.7+ 和 PostgreSQL 9.5+ 支持 IF NOT EXISTS
        // 如果数据库版本较旧，可能会失败，但我们可以忽略
        if let Err(e) = db.execute(sea_orm::Statement::from_string(
            backend,
            alter_sql.to_string(),
        ))
        .await
        {
            // 如果字段已存在或其他错误，记录但不失败
            log::warn!("添加 process_name 字段时出错（可能已存在或数据库版本不支持 IF NOT EXISTS）: {}", e);
            
            // 尝试不使用 IF NOT EXISTS 的版本（对于旧版 MySQL）
            let alter_sql_fallback = match backend {
                sea_orm::DatabaseBackend::MySql => {
                    "ALTER TABLE accelerator_games ADD COLUMN process_name VARCHAR(255) NOT NULL DEFAULT ''"
                }
                _ => return Ok(()),
            };
            
            if let Err(e2) = db.execute(sea_orm::Statement::from_string(
                backend,
                alter_sql_fallback.to_string(),
            ))
            .await
            {
                // 如果还是失败，可能是字段已存在，记录但不失败
                log::warn!("添加 process_name 字段失败（可能已存在）: {}", e2);
            }
        }
    }

    Ok(())
}
