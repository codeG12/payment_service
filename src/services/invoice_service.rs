use crate::errors::AppError;
use crate::models::invoice::{Invoice, LineItem};
use crate::services::webhook_service;
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};
use ulid::Ulid;
pub async fn create_invoice(
    pool: &PgPool,
    business_id: &str,
    customer_id: &str,
    line_items: Vec<(String, i32, i64)>,
    idempotency_key: Option<String>,
) -> Result<Invoice, AppError> {
    let mut tx = pool.begin().await?;
    let invoice_id = Ulid::new().to_string();
    let mut total_cents = 0;
    for (pos, (desc, qty, unit_amount)) in line_items.into_iter().enumerate() {
        let amount = qty as i64 * unit_amount;
        total_cents += amount;
        sqlx::query("INSERT INTO line_items (id, invoice_id, description, quantity, unit_amount_cents, amount_cents, position) VALUES ($1, $2, $3, $4, $5, $6, $7)").bind(Ulid::new().to_string()).bind(&invoice_id).bind(desc).bind(qty).bind(unit_amount).bind(amount).bind(pos as i32).execute(&mut *tx).await?;
    }
    let invoice = sqlx::query_as::<_, Invoice>("INSERT INTO invoices (id, business_id, customer_id, total_cents, due_date, idempotency_key) VALUES ($1, $2, $3, $4, NOW() + INTERVAL '30 days', $5) RETURNING *").bind(&invoice_id).bind(business_id).bind(customer_id).bind(total_cents).bind(idempotency_key).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    let _ =
        webhook_service::queue_webhook(pool, business_id, "invoice.created", json!(invoice)).await;
    Ok(invoice)
}
pub async fn transition_state(
    pool: &PgPool,
    business_id: &str,
    invoice_id: &str,
    to_state: &str,
) -> Result<Invoice, AppError> {
    let invoice =
        sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE id = $1 AND business_id = $2")
            .bind(invoice_id)
            .bind(business_id)
            .fetch_optional(pool)
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
    let updated = sqlx::query_as::<_, Invoice>(
        "UPDATE invoices SET state = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
    )
    .bind(to_state)
    .bind(invoice_id)
    .fetch_one(pool)
    .await?;
    Ok(updated)
}
