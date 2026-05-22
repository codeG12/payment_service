use crate::config::AppConfig;
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn init_pool(config: &AppConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
}
