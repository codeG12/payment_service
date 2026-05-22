use crate::dal::invoice as dal;
use crate::errors::AppError;
use crate::models::invoice::Invoice;
use crate::services::webhook_service;
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use ulid::Ulid;
pub async fn create_invoice(
    pool: &PgPool,
    business_id: &str,
    customer_id: &str,
    line_items: Vec<(String, i32, i64)>,
    idempotency_key: Option<String>,
) -> Result<Invoice, AppError> {
    let invoice_id = Ulid::new().to_string();
    let mut total_cents = 0;
    for (_, qty, unit_amount) in &line_items {
        total_cents += *qty as i64 * *unit_amount;
    }
    let invoice = Invoice {
        id: invoice_id,
        business_id: business_id.to_string(),
        customer_id: customer_id.to_string(),
        state: "draft".to_string(),
        currency: "USD".to_string(),
        total_cents,
        due_date: Utc::now() + Duration::days(30),
        idempotency_key,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let created = dal::create_with_items(pool, invoice, line_items).await?;
    let _ =
        webhook_service::queue_webhook(pool, business_id, "invoice.created", json!(created)).await;
    Ok(created)
}
pub async fn transition_state(
    pool: &PgPool,
    business_id: &str,
    invoice_id: &str,
    to_state: &str,
) -> Result<Invoice, AppError> {
    let invoice = dal::find_by_id(pool, invoice_id, business_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Invoice not found".into()))?;
    let allowed = match (invoice.state.as_str(), to_state) {
        ("draft", "open") => true,
        ("draft", "void") => true,
        ("open", "void") => true,
        ("open", "paid") => true,
        ("open", "uncollectible") => true,
        _ => false,
    };
    if !allowed {
        return Err(AppError::InvalidTransition {
            from: invoice.state,
            to: to_state.to_string(),
        });
    }
    let updated = dal::update_state(pool, invoice_id, to_state).await?;
    Ok(updated)
}
