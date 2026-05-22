use crate::handlers::customer;
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
pub fn routes() -> Router<PgPool> {
    Router::new()
        .route(
            "/",
            post(customer::create_customer).get(customer::list_customers),
        )
        .route("/:id", get(customer::get_customer))
}
