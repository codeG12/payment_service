use crate::errors::AppError;
use crate::models::customer::Customer;
use sqlx::PgPool;
pub async fn insert(
    pool: &PgPool,
    id: &str,
    business_id: &str,
    name: &str,
    email: &str,
) -> Result<Customer, AppError> {
    let customer = sqlx::query_as::<_, Customer>(
        "INSERT INTO customers (id, business_id, name, email) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(id)
    .bind(business_id)
    .bind(name)
    .bind(email)
    .fetch_one(pool)
    .await?;
    Ok(customer)
}
pub async fn list(pool: &PgPool, business_id: &str) -> Result<Vec<Customer>, AppError> {
    let customers = sqlx::query_as::<_, Customer>("SELECT * FROM customers WHERE business_id = $1")
        .bind(business_id)
        .fetch_all(pool)
        .await?;
    Ok(customers)
}
pub async fn find_by_id(
    pool: &PgPool,
    id: &str,
    business_id: &str,
) -> Result<Option<Customer>, AppError> {
    let customer =
        sqlx::query_as::<_, Customer>("SELECT * FROM customers WHERE id = $1 AND business_id = $2")
            .bind(id)
            .bind(business_id)
            .fetch_optional(pool)
            .await?;
    Ok(customer)
}
