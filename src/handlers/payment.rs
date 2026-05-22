use crate::config::AppConfig;
use crate::errors::AppError;
use crate::models::payment_attempt::PaymentAttempt;
use crate::routes::auth::AuthenticatedBusiness;
use crate::services::payment_service;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Extension, Json,
};
use serde::Deserialize;
use sqlx::PgPool;
#[derive(Deserialize)]
pub struct PayInvoiceRequest {
    pub card_token: String,
}
pub async fn pay_invoice(
    State(pool): State<PgPool>,
    Extension(config): Extension<AppConfig>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<PayInvoiceRequest>,
) -> Result<Json<PaymentAttempt>, AppError> {
    let idempotency_key = headers
        .get("Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::ValidationError("Missing Idempotency-Key header".into()))?;
    let attempt = payment_service::process_payment(
        &pool,
        &config.psp_url,
        &business.id,
        &id,
        &payload.card_token,
        idempotency_key,
    )
    .await?;
    Ok(Json(attempt))
}
