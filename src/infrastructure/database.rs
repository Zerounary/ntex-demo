use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Schema};

use crate::domain::todo;

pub async fn connect(url: &str) -> Result<DatabaseConnection, DbErr> {
    Database::connect(url).await
}

pub async fn init(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);
    let mut stmt = schema.create_table_from_entity(todo::Entity);
    stmt.if_not_exists();
    db.execute(backend.build(&stmt)).await?;
    Ok(())
}

