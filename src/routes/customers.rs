use crate::errors::AppError;
use crate::models::customer::Customer;
use crate::routes::auth::AuthenticatedBusiness;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::PgPool;
use ulid::Ulid;
#[derive(Deserialize)]
pub struct CreateCustomerRequest {
    pub name: String,
    pub email: String,
}
pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/", post(create_customer).get(list_customers))
        .route("/:id", get(get_customer))
}
async fn create_customer(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Json(payload): Json<CreateCustomerRequest>,
) -> Result<Json<Customer>, AppError> {
    let id = Ulid::new().to_string();
    let customer = sqlx::query_as::<_, Customer>(
        "INSERT INTO customers (id, business_id, name, email) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(&id)
    .bind(&business.id)
    .bind(&payload.name)
    .bind(&payload.email)
    .fetch_one(&pool)
    .await?;
    Ok(Json(customer))
}
async fn list_customers(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
) -> Result<Json<Vec<Customer>>, AppError> {
    let customers = sqlx::query_as::<_, Customer>("SELECT * FROM customers WHERE business_id = $1")
        .bind(&business.id)
        .fetch_all(&pool)
        .await?;
    Ok(Json(customers))
}
async fn get_customer(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Path(id): Path<String>,
) -> Result<Json<Customer>, AppError> {
    let customer =
        sqlx::query_as::<_, Customer>("SELECT * FROM customers WHERE id = $1 AND business_id = $2")
            .bind(&id)
            .bind(&business.id)
            .fetch_optional(&pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Customer not found".into()))?;
    Ok(Json(customer))
}
