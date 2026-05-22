use crate::errors::AppError;
use crate::models::invoice::{Invoice, LineItem};
use crate::routes::auth::AuthenticatedBusiness;
use crate::services::invoice_service;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
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
pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/", post(create_invoice).get(list_invoices))
        .route("/:id", get(get_invoice))
        .route("/:id/finalize", post(finalize_invoice))
        .route("/:id/void", post(void_invoice))
        .route("/:id/mark-uncollectible", post(mark_uncollectible))
}
async fn create_invoice(
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
async fn list_invoices(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Query(query): Query<ListInvoicesQuery>,
) -> Result<Json<Vec<Invoice>>, AppError> {
    let invoices = if let Some(state) = query.state {
        sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE business_id = $1 AND state = $2")
            .bind(&business.id)
            .bind(state)
            .fetch_all(&pool)
            .await?
    } else {
        sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE business_id = $1")
            .bind(&business.id)
            .fetch_all(&pool)
            .await?
    };
    Ok(Json(invoices))
}
async fn get_invoice(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let invoice =
        sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE id = $1 AND business_id = $2")
            .bind(&id)
            .bind(&business.id)
            .fetch_optional(&pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Invoice not found".into()))?;
    let items = sqlx::query_as::<_, LineItem>(
        "SELECT * FROM line_items WHERE invoice_id = $1 ORDER BY position",
    )
    .bind(&id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(
        serde_json::json!({ "invoice": invoice, "line_items": items }),
    ))
}
async fn finalize_invoice(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Path(id): Path<String>,
) -> Result<Json<Invoice>, AppError> {
    let invoice = invoice_service::transition_state(&pool, &business.id, &id, "open").await?;
    Ok(Json(invoice))
}
async fn void_invoice(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Path(id): Path<String>,
) -> Result<Json<Invoice>, AppError> {
    let invoice = invoice_service::transition_state(&pool, &business.id, &id, "void").await?;
    Ok(Json(invoice))
}
async fn mark_uncollectible(
    State(pool): State<PgPool>,
    AuthenticatedBusiness(business): AuthenticatedBusiness,
    Path(id): Path<String>,
) -> Result<Json<Invoice>, AppError> {
    let invoice =
        invoice_service::transition_state(&pool, &business.id, &id, "uncollectible").await?;
    Ok(Json(invoice))
}
