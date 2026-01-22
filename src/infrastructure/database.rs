use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Schema};

use crate::infrastructure::persistence::{
    accelerator_game, accelerator_game_node_binding, accelerator_node, accelerator_profile, accelerator_user, account_user,
    accelerator_user_credential, accelerator_user_session,
    admin_chain, admin_inbound, admin_node_config, admin_outbound, admin_routing, admin_user, admin_user_mapping,
    cdk_code, config_entry, node_illegal_log, node_online_user_log, node_outbound_event_log,
    node_outbound_latency_log, node_status_log, node_traffic_log, wechat_ticket,
    acceleration_session, user_wallet,
};

pub async fn connect(url: &str) -> Result<DatabaseConnection, DbErr> {
    Database::connect(url).await
}

pub async fn init(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);

    for table in [
        schema.create_table_from_entity(accelerator_game::Entity),
        schema.create_table_from_entity(accelerator_game_node_binding::Entity),
        schema.create_table_from_entity(accelerator_node::Entity),
        schema.create_table_from_entity(accelerator_profile::Entity),
        schema.create_table_from_entity(accelerator_user::Entity),
        schema.create_table_from_entity(accelerator_user_credential::Entity),
        schema.create_table_from_entity(accelerator_user_session::Entity),
        schema.create_table_from_entity(account_user::Entity),
        schema.create_table_from_entity(user_wallet::Entity),
        schema.create_table_from_entity(acceleration_session::Entity),
        schema.create_table_from_entity(admin_node_config::Entity),
        schema.create_table_from_entity(admin_user::Entity),
        schema.create_table_from_entity(admin_inbound::Entity),
        schema.create_table_from_entity(admin_outbound::Entity),
        schema.create_table_from_entity(admin_chain::Entity),
        schema.create_table_from_entity(admin_routing::Entity),
        schema.create_table_from_entity(admin_user_mapping::Entity),
        schema.create_table_from_entity(cdk_code::Entity),
        schema.create_table_from_entity(config_entry::Entity),
        schema.create_table_from_entity(wechat_ticket::Entity),
        schema.create_table_from_entity(node_traffic_log::Entity),
        schema.create_table_from_entity(node_status_log::Entity),
        schema.create_table_from_entity(node_online_user_log::Entity),
        schema.create_table_from_entity(node_illegal_log::Entity),
        schema.create_table_from_entity(node_outbound_event_log::Entity),
        schema.create_table_from_entity(node_outbound_latency_log::Entity),
    ] {
        let mut stmt = table;
        stmt.if_not_exists();
        db.execute(backend.build(&stmt)).await?;
    }

    // 迁移：检查并添加缺失的字段
    migrate_add_process_name_field(db).await?;
    migrate_node_config_fields(db).await?;
    migrate_game_node_binding_fields(db).await?;
    migrate_acceleration_session_fields(db).await?;
    migrate_admin_outbound_fields(db).await?;

    Ok(())
}

async fn migrate_admin_outbound_fields(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();

    if matches!(backend, sea_orm::DatabaseBackend::Sqlite) {
        log::warn!("SQLite 不支持 admin_outbounds 字段迁移，需要手动迁移数据");
        return Ok(());
    }

    if matches!(backend, sea_orm::DatabaseBackend::MySql) {
        let alter_sqls = vec![
            "ALTER TABLE admin_outbounds ADD COLUMN IF NOT EXISTS send_through VARCHAR(255) NULL",
        ];
        for sql in alter_sqls {
            if let Err(e) =
                db.execute(sea_orm::Statement::from_string(backend, sql.to_string()))
                    .await
            {
                log::warn!(
                    "执行 admin_outbounds 迁移 SQL 失败（可能字段已存在或数据库版本不支持 IF NOT EXISTS）: {} - {}",
                    sql,
                    e
                );
            }
        }
    }

    if matches!(backend, sea_orm::DatabaseBackend::Postgres) {
        let alter_sqls = vec![
            "ALTER TABLE admin_outbounds ADD COLUMN IF NOT EXISTS send_through VARCHAR(255)",
        ];
        for sql in alter_sqls {
            if let Err(e) =
                db.execute(sea_orm::Statement::from_string(backend, sql.to_string()))
                    .await
            {
                log::warn!(
                    "执行 admin_outbounds 迁移 SQL 失败（可能字段已存在）: {} - {}",
                    sql,
                    e
                );
            }
        }
    }

    Ok(())
}

async fn migrate_acceleration_session_fields(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();

    if matches!(backend, sea_orm::DatabaseBackend::Sqlite) {
        log::warn!("SQLite 不支持 acceleration_sessions 字段迁移，需要手动迁移数据");
        return Ok(());
    }

    if matches!(backend, sea_orm::DatabaseBackend::MySql) {
        let alter_sqls = vec![
            "ALTER TABLE acceleration_sessions ADD COLUMN IF NOT EXISTS tcp_node_id BIGINT NULL",
            "ALTER TABLE acceleration_sessions ADD COLUMN IF NOT EXISTS tcp_admin_user_id BIGINT NULL",
            "ALTER TABLE acceleration_sessions ADD COLUMN IF NOT EXISTS tcp_outbound_tag VARCHAR(128) NULL",
            "ALTER TABLE acceleration_sessions ADD COLUMN IF NOT EXISTS udp_node_id BIGINT NULL",
            "ALTER TABLE acceleration_sessions ADD COLUMN IF NOT EXISTS udp_admin_user_id BIGINT NULL",
            "ALTER TABLE acceleration_sessions ADD COLUMN IF NOT EXISTS udp_outbound_tag VARCHAR(128) NULL",
        ];

        for sql in alter_sqls {
            if let Err(e) =
                db.execute(sea_orm::Statement::from_string(backend, sql.to_string()))
                    .await
            {
                log::warn!(
                    "执行 acceleration_sessions 迁移 SQL 失败（可能字段已存在或数据库版本不支持 IF NOT EXISTS）: {} - {}",
                    sql,
                    e
                );
            }
        }
    }

    if matches!(backend, sea_orm::DatabaseBackend::Postgres) {
        let alter_sqls = vec![
            "ALTER TABLE acceleration_sessions ADD COLUMN IF NOT EXISTS tcp_node_id BIGINT",
            "ALTER TABLE acceleration_sessions ADD COLUMN IF NOT EXISTS tcp_admin_user_id BIGINT",
            "ALTER TABLE acceleration_sessions ADD COLUMN IF NOT EXISTS tcp_outbound_tag VARCHAR(128)",
            "ALTER TABLE acceleration_sessions ADD COLUMN IF NOT EXISTS udp_node_id BIGINT",
            "ALTER TABLE acceleration_sessions ADD COLUMN IF NOT EXISTS udp_admin_user_id BIGINT",
            "ALTER TABLE acceleration_sessions ADD COLUMN IF NOT EXISTS udp_outbound_tag VARCHAR(128)",
        ];

        for sql in alter_sqls {
            if let Err(e) =
                db.execute(sea_orm::Statement::from_string(backend, sql.to_string()))
                    .await
            {
                log::warn!(
                    "执行 acceleration_sessions 迁移 SQL 失败（可能字段已存在）: {} - {}",
                    sql,
                    e
                );
            }
        }
    }

    Ok(())
}

/// 迁移：为 accelerator_game_node_bindings 表添加链路与展示字段
async fn migrate_game_node_binding_fields(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();

    // SQLite 迁移（SQLite 不支持 ALTER TABLE ADD COLUMN IF NOT EXISTS/ALTER COLUMN，需要重建表）
    if matches!(backend, sea_orm::DatabaseBackend::Sqlite) {
        log::warn!("SQLite 不支持 accelerator_game_node_bindings 字段迁移，需要手动迁移数据");
        return Ok(());
    }

    // MySQL 迁移
    if matches!(backend, sea_orm::DatabaseBackend::MySql) {
        let alter_sqls = vec![
            // 新主键与类型字段
            // 说明：MySQL 对 auto_increment 约束较严格，这里尽量一次 ALTER 完成（失败会记录但不终止）
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY FIRST",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS type VARCHAR(16) NOT NULL DEFAULT 'node' AFTER game_id",
            // node_id 允许为空（type=chain 时）
            "ALTER TABLE accelerator_game_node_bindings MODIFY COLUMN node_id VARCHAR(128) NULL",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS tcp_chain_id BIGINT NULL",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS udp_chain_id BIGINT NULL",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS display_name VARCHAR(255) NULL",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS region VARCHAR(255) NULL",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS mode VARCHAR(255) NULL",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS ping INT NULL",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS status VARCHAR(255) NULL",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS remark TEXT NULL",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP",
        ];

        for sql in alter_sqls {
            if let Err(e) =
                db.execute(sea_orm::Statement::from_string(backend, sql.to_string()))
                    .await
            {
                log::warn!(
                    "执行 accelerator_game_node_bindings 迁移 SQL 失败（可能字段已存在或数据库版本不支持 IF NOT EXISTS）: {} - {}",
                    sql,
                    e
                );
            }
        }
    }

    // PostgreSQL 迁移
    if matches!(backend, sea_orm::DatabaseBackend::Postgres) {
        let alter_sqls = vec![
            // 新主键与类型字段
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS id BIGINT GENERATED BY DEFAULT AS IDENTITY",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS type VARCHAR(16) NOT NULL DEFAULT 'node'",
            // node_id 允许为空（type=chain 时）
            "ALTER TABLE accelerator_game_node_bindings ALTER COLUMN node_id DROP NOT NULL",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS tcp_chain_id BIGINT",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS udp_chain_id BIGINT",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS display_name VARCHAR(255)",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS region VARCHAR(255)",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS mode VARCHAR(255)",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS ping INTEGER",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS status VARCHAR(255)",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS remark TEXT",
            "ALTER TABLE accelerator_game_node_bindings ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP",
        ];

        for sql in alter_sqls {
            if let Err(e) =
                db.execute(sea_orm::Statement::from_string(backend, sql.to_string()))
                    .await
            {
                log::warn!(
                    "执行 accelerator_game_node_bindings 迁移 SQL 失败（可能字段已存在）: {} - {}",
                    sql,
                    e
                );
            }
        }

        // 尝试将主键切换到 id（如果仍然是旧的复合主键，这一步会失败；失败只记录 warning）
        let pk_sqls = vec![
            "ALTER TABLE accelerator_game_node_bindings DROP CONSTRAINT IF EXISTS accelerator_game_node_bindings_pkey",
            "ALTER TABLE accelerator_game_node_bindings ADD PRIMARY KEY (id)",
        ];
        for sql in pk_sqls {
            if let Err(e) =
                db.execute(sea_orm::Statement::from_string(backend, sql.to_string()))
                    .await
            {
                log::warn!(
                    "执行 accelerator_game_node_bindings 主键迁移 SQL 失败（可能已是新主键或约束名不同）: {} - {}",
                    sql,
                    e
                );
            }
        }
    }

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

/// 迁移：为 admin_node_configs 表添加/修改字段
async fn migrate_node_config_fields(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    
    // MySQL 迁移
    if matches!(backend, sea_orm::DatabaseBackend::MySql) {
        // 添加新字段（如果不存在）
        let alter_sqls = vec![
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS name VARCHAR(255) NULL",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS region VARCHAR(255) NULL",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS description TEXT NULL",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS is_online TINYINT(1) NOT NULL DEFAULT 0",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS last_seen_at TIMESTAMP NULL",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS cpu_threads INT UNSIGNED NULL",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS mem_total BIGINT UNSIGNED NULL",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS disk_total BIGINT UNSIGNED NULL",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS public_ip VARCHAR(255) NULL",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS network_interfaces JSON NULL",
        ];
        
        for sql in alter_sqls {
            if let Err(e) = db.execute(sea_orm::Statement::from_string(backend, sql.to_string())).await {
                log::warn!("执行迁移 SQL 失败（可能字段已存在）: {} - {}", sql, e);
            }
        }
        
        // 修改使用率字段类型（从 VARCHAR 改为 DOUBLE）
        // 注意：如果字段已存在且是 VARCHAR，需要先转换数据，这里简化处理
        // 实际使用时，如果字段已存在且类型不同，可能需要手动迁移数据
        let alter_type_sqls = vec![
            "ALTER TABLE admin_node_configs MODIFY COLUMN cpu_usage DOUBLE NULL",
            "ALTER TABLE admin_node_configs MODIFY COLUMN mem_usage DOUBLE NULL",
            "ALTER TABLE admin_node_configs MODIFY COLUMN disk_usage DOUBLE NULL",
        ];
        
        for sql in alter_type_sqls {
            if let Err(e) = db.execute(sea_orm::Statement::from_string(backend, sql.to_string())).await {
                log::warn!("修改字段类型失败（可能类型已正确或字段不存在）: {} - {}", sql, e);
            }
        }
        
        // 修改 node_status_logs 表的字段类型
        let alter_log_type_sqls = vec![
            "ALTER TABLE node_status_logs MODIFY COLUMN cpu DOUBLE NOT NULL",
            "ALTER TABLE node_status_logs MODIFY COLUMN mem DOUBLE NOT NULL",
            "ALTER TABLE node_status_logs MODIFY COLUMN disk DOUBLE NOT NULL",
        ];
        
        for sql in alter_log_type_sqls {
            if let Err(e) = db.execute(sea_orm::Statement::from_string(backend, sql.to_string())).await {
                log::warn!("修改 node_status_logs 字段类型失败: {} - {}", sql, e);
            }
        }

        // 移除遗留字段：inbounds（已由 admin_inbound 子表替代）
        let drop_inbounds_sqls = vec![
            "ALTER TABLE admin_node_configs DROP COLUMN IF EXISTS inbounds",
            "ALTER TABLE admin_node_configs DROP COLUMN inbounds",
        ];
        for sql in drop_inbounds_sqls {
            if let Err(e) = db.execute(sea_orm::Statement::from_string(backend, sql.to_string())).await {
                log::warn!("移除 admin_node_configs.inbounds 字段失败（可能已移除或数据库版本不支持）: {} - {}", sql, e);
            }
        }

        // accelerator_game_node_bindings.node_id 由 u64 -> String（绑定 accelerator_nodes.id）
        let alter_bind_node_id_sqls = vec![
            "ALTER TABLE accelerator_game_node_bindings MODIFY COLUMN node_id VARCHAR(128) NOT NULL",
            "ALTER TABLE accelerator_game_node_bindings MODIFY COLUMN node_id TEXT NOT NULL",
        ];
        for sql in alter_bind_node_id_sqls {
            if let Err(e) = db.execute(sea_orm::Statement::from_string(backend, sql.to_string())).await {
                log::warn!("修改 accelerator_game_node_bindings.node_id 字段类型失败（可能已是字符串或表不存在）: {} - {}", sql, e);
            }
        }
    }
    
    // PostgreSQL 迁移
    if matches!(backend, sea_orm::DatabaseBackend::Postgres) {
        let alter_sqls = vec![
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS name VARCHAR(255)",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS region VARCHAR(255)",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS description TEXT",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS is_online BOOLEAN NOT NULL DEFAULT FALSE",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS last_seen_at TIMESTAMPTZ",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS cpu_threads INTEGER",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS mem_total BIGINT",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS disk_total BIGINT",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS public_ip VARCHAR(255)",
            "ALTER TABLE admin_node_configs ADD COLUMN IF NOT EXISTS network_interfaces JSONB",
        ];
        
        for sql in alter_sqls {
            if let Err(e) = db.execute(sea_orm::Statement::from_string(backend, sql.to_string())).await {
                log::warn!("执行迁移 SQL 失败（可能字段已存在）: {} - {}", sql, e);
            }
        }
        
        // PostgreSQL 修改字段类型
        let alter_type_sqls = vec![
            "ALTER TABLE admin_node_configs ALTER COLUMN cpu_usage TYPE DOUBLE PRECISION USING CASE WHEN cpu_usage ~ '^[0-9.]+$' THEN cpu_usage::DOUBLE PRECISION ELSE NULL END",
            "ALTER TABLE admin_node_configs ALTER COLUMN mem_usage TYPE DOUBLE PRECISION USING CASE WHEN mem_usage ~ '^[0-9.]+$' THEN mem_usage::DOUBLE PRECISION ELSE NULL END",
            "ALTER TABLE admin_node_configs ALTER COLUMN disk_usage TYPE DOUBLE PRECISION USING CASE WHEN disk_usage ~ '^[0-9.]+$' THEN disk_usage::DOUBLE PRECISION ELSE NULL END",
        ];
        
        for sql in alter_type_sqls {
            if let Err(e) = db.execute(sea_orm::Statement::from_string(backend, sql.to_string())).await {
                log::warn!("修改字段类型失败: {} - {}", sql, e);
            }
        }
        
        // 修改 node_status_logs 表的字段类型
        let alter_log_type_sqls = vec![
            "ALTER TABLE node_status_logs ALTER COLUMN cpu TYPE DOUBLE PRECISION USING CASE WHEN cpu ~ '^[0-9.]+$' THEN cpu::DOUBLE PRECISION ELSE 0.0 END",
            "ALTER TABLE node_status_logs ALTER COLUMN mem TYPE DOUBLE PRECISION USING CASE WHEN mem ~ '^[0-9.]+$' THEN mem::DOUBLE PRECISION ELSE 0.0 END",
            "ALTER TABLE node_status_logs ALTER COLUMN disk TYPE DOUBLE PRECISION USING CASE WHEN disk ~ '^[0-9.]+$' THEN disk::DOUBLE PRECISION ELSE 0.0 END",
        ];
        
        for sql in alter_log_type_sqls {
            if let Err(e) = db.execute(sea_orm::Statement::from_string(backend, sql.to_string())).await {
                log::warn!("修改 node_status_logs 字段类型失败: {} - {}", sql, e);
            }
        }

        // 移除遗留字段：inbounds（已由 admin_inbound 子表替代）
        let drop_inbounds_sql = "ALTER TABLE admin_node_configs DROP COLUMN IF EXISTS inbounds";
        if let Err(e) = db
            .execute(sea_orm::Statement::from_string(
                backend,
                drop_inbounds_sql.to_string(),
            ))
            .await
        {
            log::warn!(
                "移除 admin_node_configs.inbounds 字段失败（可能已移除）: {} - {}",
                drop_inbounds_sql,
                e
            );
        }

        // accelerator_game_node_bindings.node_id 由 u64 -> String（绑定 accelerator_nodes.id）
        let alter_bind_node_id_sqls = vec![
            "ALTER TABLE accelerator_game_node_bindings ALTER COLUMN node_id TYPE VARCHAR(128) USING node_id::text",
            "ALTER TABLE accelerator_game_node_bindings ALTER COLUMN node_id TYPE TEXT USING node_id::text",
        ];
        for sql in alter_bind_node_id_sqls {
            if let Err(e) =
                db.execute(sea_orm::Statement::from_string(backend, sql.to_string()))
                    .await
            {
                log::warn!("修改 accelerator_game_node_bindings.node_id 字段类型失败（可能已是字符串或表不存在）: {} - {}", sql, e);
            }
        }
    }
    
    // SQLite 迁移（SQLite 不支持 ALTER COLUMN，需要重建表，这里简化处理）
    if matches!(backend, sea_orm::DatabaseBackend::Sqlite) {
        // SQLite 不支持 IF NOT EXISTS 和 ALTER COLUMN，这里跳过
        // 如果需要支持 SQLite，需要重建表
        log::warn!("SQLite 不支持 ALTER COLUMN，需要手动迁移数据");
    }
    
    Ok(())
}
