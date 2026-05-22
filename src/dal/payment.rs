use crate::errors::AppError;
use crate::models::payment_attempt::PaymentAttempt;
use sqlx::{PgPool, Postgres, Transaction};

pub async fn find_by_idempotency_key(
    pool: &PgPool,
    key: &str,
) -> Result<Option<PaymentAttempt>, AppError> {
    let attempt = sqlx::query_as::<_, PaymentAttempt>(
        r#"
            SELECT * 
            FROM payment_attempts 
            WHERE idempotency_key = $1
        "#,
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;

    Ok(attempt)
}

pub async fn insert_pending(pool: &PgPool, attempt: PaymentAttempt) -> Result<(), AppError> {
    sqlx::query(r#"
            INSERT INTO payment_attempts (id, invoice_id, idempotency_key, card_token, amount_cents, currency, state) 
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#)
        .bind(attempt.id)
        .bind(attempt.invoice_id)
        .bind(attempt.idempotency_key)
        .bind(attempt.card_token)
        .bind(attempt.amount_cents)
        .bind(attempt.currency)
        .bind("pending")
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn finalize_payment(
    pool: &PgPool,
    attempt_id: &str,
    invoice_id: &str,
    state: &str,
    psp_ref: Option<String>,
    err_msg: Option<String>,
) -> Result<PaymentAttempt, AppError> {
    let mut tx = pool.begin().await?;

    let attempt = sqlx::query_as::<_, PaymentAttempt>(
        r#"
            UPDATE payment_attempts 
            SET state = $1, psp_reference_id = $2, error_message = $3, updated_at = NOW() 
            WHERE id = $4 
            RETURNING *
        "#,
    )
    .bind(state)
    .bind(psp_ref)
    .bind(err_msg)
    .bind(attempt_id)
    .fetch_one(&mut *tx)
    .await?;

    if state == "succeeded" {
        sqlx::query(
            r#"
                UPDATE invoices 
                SET state = $1, updated_at = NOW() 
                WHERE id = $2
            "#,
        )
        .bind("paid")
        .bind(invoice_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(attempt)
}
