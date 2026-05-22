use crate::errors::AppError;
use crate::models::webhook::{WebhookDelivery, WebhookEndpoint};
use chrono::{Duration as ChronoDuration, Utc};
use hex;
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha2::Sha256;
use sqlx::PgPool;
use std::time::Duration;
use ulid::Ulid;
pub async fn register_endpoint(
    pool: &PgPool,
    business_id: &str,
    target_url: &str,
) -> Result<WebhookEndpoint, AppError> {
    let id = Ulid::new().to_string();
    let secret = hex::encode(rand::random::<[u8; 16]>());
    let endpoint = sqlx::query_as::<_, WebhookEndpoint>("INSERT INTO webhook_endpoints (id, business_id, target_url, secret) VALUES ($1, $2, $3, $4) RETURNING *").bind(&id).bind(business_id).bind(target_url).bind(secret).fetch_one(pool).await?;
    Ok(endpoint)
}
pub async fn queue_webhook(
    pool: &PgPool,
    business_id: &str,
    event_type: &str,
    payload: Value,
) -> Result<(), AppError> {
    let endpoints = sqlx::query_as::<_, WebhookEndpoint>(
        "SELECT * FROM webhook_endpoints WHERE business_id = $1 AND is_active = TRUE",
    )
    .bind(business_id)
    .fetch_all(pool)
    .await?;
    for endpoint in endpoints {
        let delivery_id = Ulid::new().to_string();
        let event_payload =
            json!({ "event": event_type, "timestamp": Utc::now(), "data": payload });
        sqlx::query("INSERT INTO webhook_deliveries (id, webhook_endpoint_id, event_type, payload) VALUES ($1, $2, $3, $4)").bind(&delivery_id).bind(&endpoint.id).bind(event_type).bind(&event_payload).execute(pool).await?;
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            let _ = deliver_webhook(pool_clone, delivery_id).await;
        });
    }
    Ok(())
}
pub async fn deliver_webhook(pool: PgPool, delivery_id: String) -> Result<(), AppError> {
    let delivery =
        sqlx::query_as::<_, WebhookDelivery>("SELECT * FROM webhook_deliveries WHERE id = $1")
            .bind(&delivery_id)
            .fetch_one(&pool)
            .await?;
    if delivery.state == "delivered" || delivery.state == "failed" {
        return Ok(());
    }
    let endpoint =
        sqlx::query_as::<_, WebhookEndpoint>("SELECT * FROM webhook_endpoints WHERE id = $1")
            .bind(&delivery.webhook_endpoint_id)
            .fetch_one(&pool)
            .await?;
    let payload_bytes = serde_json::to_vec(&delivery.payload).unwrap();
    let mut mac = Hmac::<Sha256>::new_from_slice(endpoint.secret.as_bytes()).unwrap();
    mac.update(&payload_bytes);
    let signature = hex::encode(mac.finalize().into_bytes());
    let client = reqwest::Client::new();
    let res = client
        .post(&endpoint.target_url)
        .header("X-Webhook-Signature", format!("sha256={}", signature))
        .header("X-Webhook-Event", &delivery.event_type)
        .json(&delivery.payload)
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    match res {
        Ok(response) if response.status().is_success() => {
            sqlx::query("UPDATE webhook_deliveries SET state = $1, last_http_status = $2, updated_at = NOW() WHERE id = $3").bind("delivered").bind(response.status().as_u16() as i32).bind(&delivery_id).execute(&pool).await?;
        }
        _ => {
            let next_attempt = delivery.attempt_count + 1;
            if next_attempt >= 5 {
                sqlx::query("UPDATE webhook_deliveries SET state = $1, attempt_count = $2, updated_at = NOW() WHERE id = $3").bind("failed").bind(next_attempt).bind(&delivery_id).execute(&pool).await?;
            } else {
                let delay = match next_attempt {
                    1 => 30,
                    2 => 300,
                    3 => 1800,
                    4 => 7200,
                    _ => 0,
                };
                let next_retry = Utc::now() + ChronoDuration::seconds(delay);
                sqlx::query("UPDATE webhook_deliveries SET attempt_count = $1, next_retry_at = $2, updated_at = NOW() WHERE id = $3").bind(next_attempt).bind(next_retry).bind(&delivery_id).execute(&pool).await?;
            }
        }
    }
    Ok(())
}
pub async fn start_webhook_retry_loop(pool: PgPool) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        let deliveries = sqlx::query_as::<_, WebhookDelivery>(
            "SELECT * FROM webhook_deliveries WHERE state = $1 AND next_retry_at <= NOW()",
        )
        .bind("pending")
        .fetch_all(&pool)
        .await;
        if let Ok(deliveries) = deliveries {
            for delivery in deliveries {
                let pool_clone = pool.clone();
                let delivery_id = delivery.id;
                tokio::spawn(async move {
                    let _ = deliver_webhook(pool_clone, delivery_id).await;
                });
            }
        }
    }
}
