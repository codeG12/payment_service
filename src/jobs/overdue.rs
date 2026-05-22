use crate::services::webhook_service;
use serde_json::json;
use sqlx::PgPool;
pub async fn run_overdue_job(pool: PgPool) {
    let res = sqlx::query("UPDATE invoices SET state = $1, updated_at = NOW() WHERE state = $2 AND due_date < NOW() RETURNING id, business_id").bind("uncollectible").bind("open").fetch_all(&pool).await;
    if let Ok(rows) = res {
        for row in rows {
            use sqlx::Row;
            let id: String = row.get("id");
            let business_id: String = row.get("business_id");
            let _ = webhook_service::queue_webhook(
                &pool,
                &business_id,
                "invoice.payment_failed",
                json!({ "invoice_id": id, "reason": "overdue" }),
            )
            .await;
        }
    }
}
