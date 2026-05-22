use crate::errors::AppError;
use crate::models::invoice::Invoice;
use crate::models::payment_attempt::PaymentAttempt;
use crate::services::webhook_service;
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};
use std::time::Duration;
use ulid::Ulid;
pub enum PspResult {
    Succeeded { reference: String },
    Failed { error: String, reference: String },
}
pub async fn call_mock_psp(card_token: &str, _amount_cents: i64) -> PspResult {
    tokio::time::sleep(Duration::from_millis(200)).await;
    match card_token {
        t if t.starts_with("tok_fail") => PspResult::Failed {
            error: "card_declined".into(),
            reference: Ulid::new().to_string(),
        },
        t if t.starts_with("tok_timeout") => {
            tokio::time::sleep(Duration::from_secs(30)).await;
            PspResult::Failed {
                error: "timeout".into(),
                reference: String::new(),
            }
        }
        _ => PspResult::Succeeded {
            reference: Ulid::new().to_string(),
        },
    }
}
pub async fn process_payment(
    pool: &PgPool,
    business_id: &str,
    invoice_id: &str,
    card_token: &str,
    idempotency_key: &str,
) -> Result<PaymentAttempt, AppError> {
    let invoice =
        sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE id = $1 AND business_id = $2")
            .bind(invoice_id)
            .bind(business_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Invoice not found".into()))?;
    if invoice.state != "open" {
        return Err(AppError::InvalidTransition {
            from: invoice.state,
            to: "paid".into(),
        });
    }
    if let Some(existing) = sqlx::query_as::<_, PaymentAttempt>(
        "SELECT * FROM payment_attempts WHERE idempotency_key = $1",
    )
    .bind(idempotency_key)
    .fetch_optional(pool)
    .await?
    {
        return Ok(existing);
    }
    let attempt_id = Ulid::new().to_string();
    sqlx::query("INSERT INTO payment_attempts (id, invoice_id, idempotency_key, card_token, amount_cents, currency, state) VALUES ($1, $2, $3, $4, $5, $6, $7)").bind(&attempt_id).bind(invoice_id).bind(idempotency_key).bind(card_token).bind(invoice.total_cents).bind(&invoice.currency).bind("pending").execute(pool).await?;
    let psp_res = match tokio::time::timeout(
        Duration::from_secs(10),
        call_mock_psp(card_token, invoice.total_cents),
    )
    .await
    {
        Ok(res) => res,
        Err(_) => PspResult::Failed {
            error: "psp_timeout".into(),
            reference: String::new(),
        },
    };
    let mut tx = pool.begin().await?;
    let (state, psp_ref, err_msg) = match psp_res {
        PspResult::Succeeded { reference } => ("succeeded", Some(reference), None),
        PspResult::Failed { error, reference } => ("failed", Some(reference), Some(error)),
    };
    let attempt = sqlx::query_as::<_, PaymentAttempt>("UPDATE payment_attempts SET state = $1, psp_reference_id = $2, error_message = $3, updated_at = NOW() WHERE id = $4 RETURNING *").bind(state).bind(psp_ref).bind(err_msg).bind(&attempt_id).fetch_one(&mut *tx).await?;
    if state == "succeeded" {
        sqlx::query("UPDATE invoices SET state = $1, updated_at = NOW() WHERE id = $2")
            .bind("paid")
            .bind(invoice_id)
            .execute(&mut *tx)
            .await?;
        let _ =
            webhook_service::queue_webhook(pool, business_id, "invoice.paid", json!(invoice)).await;
    } else {
        let _ = webhook_service::queue_webhook(
            pool,
            business_id,
            "invoice.payment_failed",
            json!(attempt),
        )
        .await;
    }
    tx.commit().await?;
    Ok(attempt)
}
