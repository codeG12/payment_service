use crate::handlers::mock_psp;
use axum::{routing::post, Router};
pub fn routes() -> Router<sqlx::PgPool> {
    Router::new().route("/charge", post(mock_psp::charge))
}
