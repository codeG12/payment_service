use crate::errors::AppError;
use crate::models::webhook::WebhookEndpoint;
use crate::routes::auth::AuthenticatedBusiness;
use crate::services::webhook_service;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post},
};
use serde::Deserialize;
use sqlx::PgPool;
#[derive(Deserialize)]
pub struct RegisterWebhookRequest {
    pub target_url: String,
}
pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/", post(register_webhook).get(list_webhooks))
        .route("/:id", delete(deactivate_webhook))
}
async fn register_webhook(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Json(payload): Json<RegisterWebhookRequest>,
) -> Result<Json<WebhookEndpoint>, AppError> {
    let endpoint =
        webhook_service::register_endpoint(&pool, &business.id, &payload.target_url).await?;
    Ok(Json(endpoint))
}
async fn list_webhooks(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
) -> Result<Json<Vec<WebhookEndpoint>>, AppError> {
    let endpoints = sqlx::query_as::<_, WebhookEndpoint>(
        "SELECT * FROM webhook_endpoints WHERE business_id = $1",
    )
    .bind(&business.id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(endpoints))
}
async fn deactivate_webhook(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Path(id): Path<String>,
) -> Result<(), AppError> {
    sqlx::query("UPDATE webhook_endpoints SET is_active = FALSE, updated_at = NOW() WHERE id = $1 AND business_id = $2").bind(id).bind(business.id).execute(&pool).await?;
    Ok(())
}
