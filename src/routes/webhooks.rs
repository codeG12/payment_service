use crate::handlers::webhook;
use axum::{
    Router,
    routing::{delete, get, post},
};
use sqlx::PgPool;
pub fn routes() -> Router<PgPool> {
    Router::new()
        .route(
            "/",
            post(webhook::register_webhook).get(webhook::list_webhooks),
        )
        .route("/:id", delete(webhook::deactivate_webhook))
}
