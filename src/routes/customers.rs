use crate::handlers::customer;
use axum::{
    routing::{get, post},
    Router,
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
