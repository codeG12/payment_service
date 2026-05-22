use crate::handlers::invoice;
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
pub fn routes() -> Router<PgPool> {
    Router::new()
        .route(
            "/",
            post(invoice::create_invoice).get(invoice::list_invoices),
        )
        .route("/:id", get(invoice::get_invoice))
        .route("/:id/finalize", post(invoice::finalize_invoice))
        .route("/:id/void", post(invoice::void_invoice))
        .route("/:id/mark-uncollectible", post(invoice::mark_uncollectible))
}
