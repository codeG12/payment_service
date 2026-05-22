use crate::dal::customer as dal;
use crate::errors::AppError;
use crate::models::customer::Customer;
use crate::routes::auth::AuthenticatedBusiness;
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use sqlx::PgPool;
use ulid::Ulid;
#[derive(Deserialize)]
pub struct CreateCustomerRequest {
    pub name: String,
    pub email: String,
}
pub async fn create_customer(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Json(payload): Json<CreateCustomerRequest>,
) -> Result<Json<Customer>, AppError> {
    let id = Ulid::new().to_string();
    let customer = dal::insert(&pool, &id, &business.id, &payload.name, &payload.email).await?;
    Ok(Json(customer))
}
pub async fn list_customers(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
) -> Result<Json<Vec<Customer>>, AppError> {
    let customers = dal::list(&pool, &business.id).await?;
    Ok(Json(customers))
}
pub async fn get_customer(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Path(id): Path<String>,
) -> Result<Json<Customer>, AppError> {
    let customer = dal::find_by_id(&pool, &id, &business.id)
        .await?
        .ok_or_else(|| AppError::NotFound("Customer not found".into()))?;
    Ok(Json(customer))
}
