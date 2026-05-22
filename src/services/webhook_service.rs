use crate::dal::webhook as dal;
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
    let endpoint = WebhookEndpoint {
        id,
        business_id: business_id.to_string(),
        target_url: target_url.to_string(),
        secret,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    dal::insert_endpoint(pool, endpoint).await
}
pub async fn queue_webhook(
    pool: &PgPool,
    business_id: &str,
    event_type: &str,
    payload: Value,
) -> Result<(), AppError> {
    let endpoints = dal::list_endpoints(pool, business_id).await?;
    for endpoint in endpoints {
        if !endpoint.is_active {
            continue;
        }
        let delivery_id = Ulid::new().to_string();
        let event_payload =
            json!({ "event": event_type, "timestamp": Utc::now(), "data": payload });
        let delivery = WebhookDelivery {
            id: delivery_id.clone(),
            webhook_endpoint_id: endpoint.id,
            event_type: event_type.to_string(),
            payload: event_payload,
            state: "pending".to_string(),
            attempt_count: 0,
            next_retry_at: Utc::now(),
            last_http_status: None,
            last_error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        dal::insert_delivery(pool, delivery).await?;
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            let _ = deliver_webhook(pool_clone, delivery_id).await;
        });
    }
    Ok(())
}
pub async fn deliver_webhook(pool: PgPool, delivery_id: String) -> Result<(), AppError> {
    let delivery = dal::find_delivery_by_id(&pool, &delivery_id).await?;
    if delivery.state == "delivered" || delivery.state == "failed" {
        return Ok(());
    }
    let endpoint = dal::find_endpoint_by_id(&pool, &delivery.webhook_endpoint_id).await?;
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
            dal::update_delivery_status(
                &pool,
                &delivery_id,
                "delivered",
                Some(response.status().as_u16() as i32),
            )
            .await?;
        }
        _ => {
            let next_attempt = delivery.attempt_count + 1;
            if next_attempt >= 5 {
                dal::update_delivery_retry(
                    &pool,
                    &delivery_id,
                    next_attempt,
                    Utc::now(),
                    Some("failed".to_string()),
                )
                .await?;
            } else {
                let delay = match next_attempt {
                    1 => 30,
                    2 => 300,
                    3 => 1800,
                    4 => 7200,
                    _ => 0,
                };
                let next_retry = Utc::now() + ChronoDuration::seconds(delay);
                dal::update_delivery_retry(&pool, &delivery_id, next_attempt, next_retry, None)
                    .await?;
            }
        }
    }
    Ok(())
}
pub async fn start_webhook_retry_loop(pool: PgPool) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        if let Ok(deliveries) = dal::list_pending_retries(&pool).await {
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
