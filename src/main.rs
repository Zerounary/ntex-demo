mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

use dotenv::dotenv;
use infrastructure::database;
use presentation::web::{self, AppState};

use crate::config::AppConfig;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = AppConfig::from_env();

    let db = database::connect(&config.database_url)
        .await
        .map_err(to_io_error)?;
    database::init(&db).await.map_err(to_io_error)?;

    let state = AppState::new(db);
    web::serve(config.port, state).await
}

fn to_io_error(err: impl std::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
}