use crate::errors::AppError;
use crate::models::business::Business;
use sqlx::PgPool;

pub async fn find_by_prefix(pool: &PgPool, prefix: &str) -> Result<Vec<Business>, AppError> {
    let businesses = sqlx::query_as::<_, Business>(
        r#"
            SELECT * 
            FROM businesses 
            WHERE api_key_prefix = $1 
              AND is_active = TRUE
        "#,
    )
    .bind(prefix)
    .fetch_all(pool)
    .await?;

    Ok(businesses)
}

pub async fn insert(
    pool: &PgPool,
    id: &str,
    name: &str,
    hash: &str,
    prefix: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
            INSERT INTO businesses (id, name, api_key_hash, api_key_prefix) 
            VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(hash)
    .bind(prefix)
    .execute(pool)
    .await?;

    Ok(())
}
