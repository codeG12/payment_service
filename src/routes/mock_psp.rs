use crate::handlers::mock_psp;
use axum::{Router, routing::post};
pub fn routes() -> Router<sqlx::PgPool> {
    Router::new().route("/charge", post(mock_psp::charge))
}
