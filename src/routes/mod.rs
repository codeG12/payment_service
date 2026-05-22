pub mod auth;
pub mod businesses;
pub mod customers;
pub mod invoices;
pub mod payments;
pub mod webhooks;
use crate::config::AppConfig;
use axum::Router;
use sqlx::PgPool;
pub fn routes(pool: PgPool, config: AppConfig) -> Router {
    Router::new()
        .nest("/customers", customers::routes())
        .nest("/invoices", invoices::routes())
        .nest("/webhooks", webhooks::routes())
        .nest("/businesses", businesses::routes())
        .with_state(pool)
        .layer(axum::Extension(config))
}
