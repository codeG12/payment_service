use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct WebhookEndpoint {
    pub id: String,
    pub business_id: String,
    pub target_url: String,
    pub secret: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct WebhookDelivery {
    pub id: String,
    pub webhook_endpoint_id: String,
    pub event_type: String,
    pub payload: Value,
    pub state: String,
    pub attempt_count: i32,
    pub next_retry_at: DateTime<Utc>,
    pub last_http_status: Option<i32>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
