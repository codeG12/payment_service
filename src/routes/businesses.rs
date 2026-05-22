use crate::handlers::business;
use axum::{routing::post, Router};
use sqlx::PgPool;
pub fn routes() -> Router<PgPool> {
    Router::new().route("/", post(business::register_business))
}
