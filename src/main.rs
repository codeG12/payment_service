mod config;
mod db;
mod errors;
mod jobs;
mod models;
mod routes;
mod services;

use crate::config::AppConfig;
use crate::db::init_pool;
use axum::Router;
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = AppConfig::from_env();
    let pool = init_pool(&config).await?;

    // Run migrations
    info!("Running migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Set up app state
    let app = Router::new().nest("/v1", routes::routes(pool.clone(), config.clone()));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
