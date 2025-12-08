use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Schema};

use crate::infrastructure::persistence::{
    accelerator_game, accelerator_node, accelerator_profile, accelerator_user, account_user,
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
        schema.create_table_from_entity(cdk_code::Entity),
        schema.create_table_from_entity(config_entry::Entity),
        schema.create_table_from_entity(wechat_ticket::Entity),
    ] {
        let mut stmt = table;
        stmt.if_not_exists();
        db.execute(backend.build(&stmt)).await?;
    }

    Ok(())
}
