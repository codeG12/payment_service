use crate::dal::invoice as i_dal;
use crate::dal::payment as p_dal;
use crate::errors::AppError;
use crate::models::payment_attempt::PaymentAttempt;
use crate::services::webhook_service;
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
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
    let invoice = i_dal::find_by_id(pool, invoice_id, business_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Invoice not found".into()))?;
    if invoice.state != "open" {
        return Err(AppError::InvalidTransition {
            from: invoice.state,
            to: "paid".into(),
        });
    }
    if let Some(existing) = p_dal::find_by_idempotency_key(pool, idempotency_key).await? {
        return Ok(existing);
    }
    let attempt_id = Ulid::new().to_string();
    let attempt = PaymentAttempt {
        id: attempt_id.clone(),
        invoice_id: invoice_id.to_string(),
        idempotency_key: idempotency_key.to_string(),
        card_token: card_token.to_string(),
        amount_cents: invoice.total_cents,
        currency: invoice.currency.clone(),
        state: "pending".to_string(),
        psp_reference_id: None,
        error_message: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    p_dal::insert_pending(pool, attempt).await?;
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
    let (state, psp_ref, err_msg) = match psp_res {
        PspResult::Succeeded { reference } => ("succeeded", Some(reference), None),
        PspResult::Failed { error, reference } => ("failed", Some(reference), Some(error)),
    };
    let finalized =
        p_dal::finalize_payment(pool, &attempt_id, invoice_id, state, psp_ref, err_msg).await?;
    if state == "succeeded" {
        let _ =
            webhook_service::queue_webhook(pool, business_id, "invoice.paid", json!(invoice)).await;
    } else {
        let _ = webhook_service::queue_webhook(
            pool,
            business_id,
            "invoice.payment_failed",
            json!(finalized),
        )
        .await;
    }
    Ok(finalized)
}
