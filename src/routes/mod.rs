pub mod auth;
pub mod businesses;
pub mod customers;
pub mod invoices;
pub mod mock_psp;
pub mod payments;
pub mod webhooks;
use crate::config::AppConfig;
use axum::Router;
use sqlx::PgPool;
pub fn routes(pool: PgPool, config: AppConfig) -> Router {
    Router::new()
        .nest("/customers", customers::routes())
        .nest("/invoices", invoices::routes())
        .merge(payments::routes())
        .nest("/webhooks", webhooks::routes())
        .nest("/businesses", businesses::routes())
        .nest("/mock_psp", mock_psp::routes())
        .with_state(pool)
        .layer(axum::Extension(config))
}
