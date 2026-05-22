use crate::dal::invoice as dal;
use crate::errors::AppError;
use crate::models::invoice::Invoice;
use crate::routes::auth::AuthenticatedBusiness;
use crate::services::invoice_service;
use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use sqlx::PgPool;
#[derive(Deserialize)]
pub struct CreateInvoiceRequest {
    pub customer_id: String,
    pub line_items: Vec<CreateLineItemRequest>,
    pub idempotency_key: Option<String>,
}
#[derive(Deserialize)]
pub struct CreateLineItemRequest {
    pub description: String,
    pub quantity: i32,
    pub unit_amount_cents: i64,
}
#[derive(Deserialize)]
pub struct ListInvoicesQuery {
    pub state: Option<String>,
}
pub async fn create_invoice(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Json(payload): Json<CreateInvoiceRequest>,
) -> Result<Json<Invoice>, AppError> {
    let items = payload
        .line_items
        .into_iter()
        .map(|i| (i.description, i.quantity, i.unit_amount_cents))
        .collect();
    let invoice = invoice_service::create_invoice(
        &pool,
        &business.id,
        &payload.customer_id,
        items,
        payload.idempotency_key,
    )
    .await?;
    Ok(Json(invoice))
}
pub async fn list_invoices(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Query(query): Query<ListInvoicesQuery>,
) -> Result<Json<Vec<Invoice>>, AppError> {
    let invoices = dal::list(&pool, &business.id, query.state).await?;
    Ok(Json(invoices))
}
pub async fn get_invoice(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let invoice = dal::find_by_id(&pool, &id, &business.id)
        .await?
        .ok_or_else(|| AppError::NotFound("Invoice not found".into()))?;
    let items = dal::find_items_by_invoice_id(&pool, &id).await?;
    Ok(Json(
        serde_json::json!({ "invoice": invoice, "line_items": items }),
    ))
}
pub async fn finalize_invoice(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Path(id): Path<String>,
) -> Result<Json<Invoice>, AppError> {
    let invoice = invoice_service::transition_state(&pool, &business.id, &id, "open").await?;
    Ok(Json(invoice))
}
pub async fn void_invoice(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Path(id): Path<String>,
) -> Result<Json<Invoice>, AppError> {
    let invoice = invoice_service::transition_state(&pool, &business.id, &id, "void").await?;
    Ok(Json(invoice))
}
pub async fn mark_uncollectible(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Path(id): Path<String>,
) -> Result<Json<Invoice>, AppError> {
    let invoice =
        invoice_service::transition_state(&pool, &business.id, &id, "uncollectible").await?;
    Ok(Json(invoice))
}
