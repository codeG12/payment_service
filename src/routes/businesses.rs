use crate::handlers::business;
use axum::{Router, routing::post};
use sqlx::PgPool;
pub fn routes() -> Router<PgPool> {
    Router::new().route("/", post(business::register_business))
}
