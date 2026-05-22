use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PaymentAttempt {
    pub id: String,
    pub invoice_id: String,
    pub idempotency_key: String,
    pub card_token: String,
    pub amount_cents: i64,
    pub currency: String,
    pub state: String,
    pub psp_reference_id: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
