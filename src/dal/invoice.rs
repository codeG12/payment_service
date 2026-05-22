use crate::errors::AppError;
use crate::models::invoice::{Invoice, LineItem};
use sqlx::{PgPool, Postgres, Transaction};
use ulid::Ulid;

pub async fn create_with_items(
    pool: &PgPool,
    invoice: Invoice,
    items: Vec<(String, i32, i64)>,
) -> Result<Invoice, AppError> {
    let mut tx = pool.begin().await?;

    for (pos, (desc, qty, unit_amount)) in items.into_iter().enumerate() {
        let amount = qty as i64 * unit_amount;
        sqlx::query(r#"
                INSERT INTO line_items (id, invoice_id, description, quantity, unit_amount_cents, amount_cents, position) 
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#)
            .bind(Ulid::new().to_string())
            .bind(&invoice.id)
            .bind(desc)
            .bind(qty)
            .bind(unit_amount)
            .bind(amount)
            .bind(pos as i32)
            .execute(&mut *tx)
            .await?;
    }

    let created = sqlx::query_as::<_, Invoice>(r#"
            INSERT INTO invoices (id, business_id, customer_id, total_cents, due_date, idempotency_key) 
            VALUES ($1, $2, $3, $4, $5, $6) 
            RETURNING *
        "#)
        .bind(&invoice.id)
        .bind(&invoice.business_id)
        .bind(&invoice.customer_id)
        .bind(invoice.total_cents)
        .bind(invoice.due_date)
        .bind(invoice.idempotency_key)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(created)
}

pub async fn find_by_id(
    pool: &PgPool,
    id: &str,
    business_id: &str,
) -> Result<Option<Invoice>, AppError> {
    let invoice = sqlx::query_as::<_, Invoice>(
        r#"
            SELECT * 
            FROM invoices 
            WHERE id = $1 AND business_id = $2
        "#,
    )
    .bind(id)
    .bind(business_id)
    .fetch_optional(pool)
    .await?;

    Ok(invoice)
}

pub async fn find_items_by_invoice_id(
    pool: &PgPool,
    invoice_id: &str,
) -> Result<Vec<LineItem>, AppError> {
    let items = sqlx::query_as::<_, LineItem>(
        r#"
            SELECT * 
            FROM line_items 
            WHERE invoice_id = $1 
            ORDER BY position
        "#,
    )
    .bind(invoice_id)
    .fetch_all(pool)
    .await?;

    Ok(items)
}

pub async fn list(
    pool: &PgPool,
    business_id: &str,
    state: Option<String>,
) -> Result<Vec<Invoice>, AppError> {
    let invoices = if let Some(s) = state {
        sqlx::query_as::<_, Invoice>(
            r#"
                SELECT * 
                FROM invoices 
                WHERE business_id = $1 AND state = $2
            "#,
        )
        .bind(business_id)
        .bind(s)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Invoice>(
            r#"
                SELECT * 
                FROM invoices 
                WHERE business_id = $1
            "#,
        )
        .bind(business_id)
        .fetch_all(pool)
        .await?
    };

    Ok(invoices)
}

pub async fn update_state(pool: &PgPool, id: &str, state: &str) -> Result<Invoice, AppError> {
    let updated = sqlx::query_as::<_, Invoice>(
        r#"
            UPDATE invoices 
            SET state = $1, updated_at = NOW() 
            WHERE id = $2 
            RETURNING *
        "#,
    )
    .bind(state)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(updated)
}

pub async fn update_overdue(pool: &PgPool) -> Result<Vec<Invoice>, AppError> {
    let rows = sqlx::query_as::<_, Invoice>(
        r#"
            UPDATE invoices 
            SET state = $1, updated_at = NOW() 
            WHERE state = $2 AND due_date < NOW() 
            RETURNING *
        "#,
    )
    .bind("uncollectible")
    .bind("open")
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
