use crate::errors::AppError;
use crate::models::webhook::{WebhookDelivery, WebhookEndpoint};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub async fn insert_endpoint(
    pool: &PgPool,
    endpoint: WebhookEndpoint,
) -> Result<WebhookEndpoint, AppError> {
    let created = sqlx::query_as::<_, WebhookEndpoint>(
        r#"
            INSERT INTO webhook_endpoints (id, business_id, target_url, secret) 
            VALUES ($1, $2, $3, $4) 
            RETURNING *
        "#,
    )
    .bind(endpoint.id)
    .bind(endpoint.business_id)
    .bind(endpoint.target_url)
    .bind(endpoint.secret)
    .fetch_one(pool)
    .await?;

    Ok(created)
}

pub async fn list_endpoints(
    pool: &PgPool,
    business_id: &str,
) -> Result<Vec<WebhookEndpoint>, AppError> {
    let endpoints = sqlx::query_as::<_, WebhookEndpoint>(
        r#"
            SELECT * 
            FROM webhook_endpoints 
            WHERE business_id = $1
        "#,
    )
    .bind(business_id)
    .fetch_all(pool)
    .await?;

    Ok(endpoints)
}

pub async fn deactivate_endpoint(
    pool: &PgPool,
    id: &str,
    business_id: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
            UPDATE webhook_endpoints 
            SET is_active = FALSE, updated_at = NOW() 
            WHERE id = $1 AND business_id = $2
        "#,
    )
    .bind(id)
    .bind(business_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_endpoint_by_id(pool: &PgPool, id: &str) -> Result<WebhookEndpoint, AppError> {
    let endpoint = sqlx::query_as::<_, WebhookEndpoint>(
        r#"
            SELECT * 
            FROM webhook_endpoints 
            WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(endpoint)
}

pub async fn insert_delivery(pool: &PgPool, delivery: WebhookDelivery) -> Result<(), AppError> {
    sqlx::query(
        r#"
            INSERT INTO webhook_deliveries (id, webhook_endpoint_id, event_type, payload) 
            VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(delivery.id)
    .bind(delivery.webhook_endpoint_id)
    .bind(delivery.event_type)
    .bind(delivery.payload)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_delivery_by_id(pool: &PgPool, id: &str) -> Result<WebhookDelivery, AppError> {
    let delivery = sqlx::query_as::<_, WebhookDelivery>(
        r#"
            SELECT * 
            FROM webhook_deliveries 
            WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(delivery)
}

pub async fn update_delivery_status(
    pool: &PgPool,
    id: &str,
    state: &str,
    status: Option<i32>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
            UPDATE webhook_deliveries 
            SET state = $1, last_http_status = $2, updated_at = NOW() 
            WHERE id = $3
        "#,
    )
    .bind(state)
    .bind(status)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_delivery_retry(
    pool: &PgPool,
    id: &str,
    count: i32,
    next_retry: DateTime<Utc>,
    state: Option<String>,
) -> Result<(), AppError> {
    if let Some(s) = state {
        sqlx::query(
            r#"
                UPDATE webhook_deliveries 
                SET state = $1, attempt_count = $2, updated_at = NOW() 
                WHERE id = $3
            "#,
        )
        .bind(s)
        .bind(count)
        .bind(id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            r#"
                UPDATE webhook_deliveries 
                SET attempt_count = $1, next_retry_at = $2, updated_at = NOW() 
                WHERE id = $3
            "#,
        )
        .bind(count)
        .bind(next_retry)
        .bind(id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn list_pending_retries(pool: &PgPool) -> Result<Vec<WebhookDelivery>, AppError> {
    let deliveries = sqlx::query_as::<_, WebhookDelivery>(
        r#"
            SELECT * 
            FROM webhook_deliveries 
            WHERE state = $1 AND next_retry_at <= NOW()
        "#,
    )
    .bind("pending")
    .fetch_all(pool)
    .await?;

    Ok(deliveries)
}
