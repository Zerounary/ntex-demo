use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("PORT must be a number");

        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://todo.db?mode=rwc".to_string());

        Self { port, database_url }
    }
}

