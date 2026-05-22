use crate::handlers::payment;
use axum::{routing::post, Router};
use sqlx::PgPool;
pub fn routes() -> Router<PgPool> {
    Router::new().route("/invoices/:id/pay", post(payment::pay_invoice))
}
