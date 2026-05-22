use crate::dal::webhook as dal;
use crate::errors::AppError;
use crate::models::webhook::WebhookEndpoint;
use crate::routes::auth::AuthenticatedBusiness;
use crate::services::webhook_service;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
#[derive(Deserialize)]
pub struct RegisterWebhookRequest {
    pub target_url: String,
}
pub async fn register_webhook(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Json(payload): Json<RegisterWebhookRequest>,
) -> Result<Json<WebhookEndpoint>, AppError> {
    let endpoint =
        webhook_service::register_endpoint(&pool, &business.id, &payload.target_url).await?;
    Ok(Json(endpoint))
}
pub async fn list_webhooks(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
) -> Result<Json<Vec<WebhookEndpoint>>, AppError> {
    let endpoints = dal::list_endpoints(&pool, &business.id).await?;
    Ok(Json(endpoints))
}
pub async fn deactivate_webhook(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Path(id): Path<String>,
) -> Result<(), AppError> {
    dal::deactivate_endpoint(&pool, &id, &business.id).await?;
    Ok(())
}
