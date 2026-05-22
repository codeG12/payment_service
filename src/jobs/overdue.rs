use crate::dal::invoice as dal;
use crate::services::webhook_service;
use serde_json::json;
use sqlx::PgPool;
pub async fn run_overdue_job(pool: PgPool) {
    if let Ok(invoices) = dal::update_overdue(&pool).await {
        for inv in invoices {
            let _ = webhook_service::queue_webhook(
                &pool,
                &inv.business_id,
                "invoice.payment_failed",
                json!({ "invoice_id": inv.id, "reason": "overdue" }),
            )
            .await;
        }
    }
}
