use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Invoice {
    pub id: String,
    pub business_id: String,
    pub customer_id: String,
    pub state: String,
    pub currency: String,
    pub total_cents: i64,
    pub due_date: DateTime<Utc>,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct LineItem {
    pub id: String,
    pub invoice_id: String,
    pub description: String,
    pub quantity: i32,
    pub unit_amount_cents: i64,
    pub amount_cents: i64,
    pub position: i32,
}
