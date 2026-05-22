use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Business {
    pub id: String,
    pub name: String,
    pub api_key_hash: String,
    pub api_key_prefix: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
