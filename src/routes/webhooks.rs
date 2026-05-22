use axum::{Router, routing::get};
use sqlx::PgPool;
pub fn routes() -> Router<PgPool> {
    Router::new().route("/", get(|| async { "webhooks" }))
}
