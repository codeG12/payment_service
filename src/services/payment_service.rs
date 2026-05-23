use crate::dal::invoice as i_dal;
use crate::dal::payment as p_dal;
use crate::errors::AppError;
use crate::models::payment_attempt::PaymentAttempt;
use crate::services::webhook_service;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::time::Duration;
use ulid::Ulid;
#[derive(Serialize)]
struct PspRequest {
    token: String,
    amount_cents: i64,
}
#[derive(Deserialize)]
struct PspResponse {
    status: String,
    psp_ref: Option<String>,
    code: Option<String>,
}
pub async fn process_payment(
    pool: &PgPool,
    psp_url: &str,
    business_id: &str,
    invoice_id: &str,
    card_token: &str,
    idempotency_key: &str,
) -> Result<PaymentAttempt, AppError> {
    if let Some(existing) = p_dal::find_by_idempotency_key(pool, idempotency_key).await? {
        return Ok(existing);
    }
    let invoice = i_dal::find_by_id(pool, invoice_id, business_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Invoice not found".into()))?;
    if invoice.state != "open" {
        return Err(AppError::InvalidTransition {
            from: invoice.state,
            to: "paid".into(),
        });
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
    let client = reqwest::Client::new();
    let psp_res = client
        .post(psp_url)
        .json(&PspRequest {
            token: card_token.to_string(),
            amount_cents: invoice.total_cents,
        })
        .timeout(Duration::from_secs(10))
        .send()
        .await;
    let (state, psp_ref, err_msg) = match psp_res {
        Ok(resp) if resp.status().is_success() => {
            let body: PspResponse = resp
                .json()
                .await
                .map_err(|e| AppError::Internal(e.into()))?;
            if body.status == "succeeded" {
                ("succeeded", body.psp_ref, None)
            } else {
                ("failed", body.psp_ref, body.code)
            }
        }
        Ok(resp) => (
            "failed",
            None,
            Some(format!("PSP Error: {}", resp.status())),
        ),
        Err(e) if e.is_timeout() => ("pending", None, Some("psp_timeout".into())),
        Err(e) => ("failed", None, Some(format!("Network Error: {}", e))),
    };
    let res = p_dal::finalize_payment(pool, &attempt_id, invoice_id, state, psp_ref, err_msg).await;
    match res {
        Ok(finalized) => {
            if state == "succeeded" {
                let _ = webhook_service::queue_webhook(
                    pool,
                    business_id,
                    "invoice.paid",
                    json!(invoice),
                )
                .await;
            } else if state == "failed" {
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
        Err(e) => Err(e),
    }
}
