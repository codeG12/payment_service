mod config;
mod db;
mod errors;
mod jobs;
mod models;
mod routes;
mod services;
use crate::config::AppConfig;
use crate::db::init_pool;
use crate::jobs::overdue;
use crate::services::webhook_service;
use axum::Router;
use std::net::SocketAddr;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let config = AppConfig::from_env();
    let pool = init_pool(&config).await?;
    info!("Running migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        webhook_service::start_webhook_retry_loop(pool_clone).await;
    });
    let pool_clone = pool.clone();
    let sched = JobScheduler::new().await?;
    sched
        .add(Job::new_async("0 0 * * * *", move |_uuid, _l| {
            let pool = pool_clone.clone();
            Box::pin(async move {
                overdue::run_overdue_job(pool).await;
            })
        })?)
        .await?;
    sched.start().await?;
    let app = Router::new().nest("/v1", routes::routes(pool.clone(), config.clone()));
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
