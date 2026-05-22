use crate::handlers::payment;
use axum::{Router, routing::post};
use sqlx::PgPool;
pub fn routes() -> Router<PgPool> {
    Router::new().route("/invoices/:id/pay", post(payment::pay_invoice))
}
