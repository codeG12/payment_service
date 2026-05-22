use payment_service::dal::{business, customer, invoice as i_dal};
use payment_service::models::invoice::Invoice;
use payment_service::services::payment_service;
use sqlx::PgPool;
use ulid::Ulid;
use chrono::Utc;
use std::sync::Arc;
use futures::future::join_all;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

async fn setup_test_data(pool: &PgPool) -> (String, String) {
    let b_id = Ulid::new().to_string();
    let _ = business::insert(pool, &b_id, "Test Biz", "hash", "prefix").await;
    
    let c_id = Ulid::new().to_string();
    let _ = customer::insert(pool, &c_id, &b_id, "Customer", "c@test.com").await;
    
    (b_id, c_id)
}

#[sqlx::test(migrations = "./migrations")]
async fn test_concurrency_payment(pool: PgPool) {
    let (b_id, c_id) = setup_test_data(&pool).await;
    let i_id = Ulid::new().to_string();
    
    sqlx::query("INSERT INTO invoices (id, business_id, customer_id, state, total_cents, due_date) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(&i_id).bind(&b_id).bind(&c_id).bind("open").bind(1000).bind(Utc::now())
        .execute(&pool).await.unwrap();

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/charge"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "succeeded",
            "psp_ref": "ref_123"
        })))
        .mount(&mock_server)
        .await;

    let psp_url = format!("{}/charge", mock_server.uri());
    let n = 5;
    let mut handles = vec![];
    let pool_arc = Arc::new(pool.clone());

    for i in 0..n {
        let p = pool_arc.clone();
        let b = b_id.clone();
        let inv = i_id.clone();
        let url = psp_url.clone();
        let i_key = format!("key_{}", i);
        
        handles.push(tokio::spawn(async move {
            payment_service::services::payment_service::process_payment(&p, &url, &b, &inv, "tok_success", &i_key).await
        }));
    }

    let results = join_all(handles).await;
    let mut success_count = 0;
    for res in results {
        if let Ok(Ok(attempt)) = res {
            if attempt.state == "succeeded" {
                success_count += 1;
            }
        }
    }

    assert_eq!(success_count, 1, "Only one payment should succeed for the same invoice");
    
    let final_invoice = i_dal::find_by_id(&pool, &i_id, &b_id).await.unwrap().unwrap();
    assert_eq!(final_invoice.state, "paid");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_idempotency(pool: PgPool) {
    let (b_id, c_id) = setup_test_data(&pool).await;
    let i_id = Ulid::new().to_string();
    
    sqlx::query("INSERT INTO invoices (id, business_id, customer_id, state, total_cents, due_date) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(&i_id).bind(&b_id).bind(&c_id).bind("open").bind(1000).bind(Utc::now())
        .execute(&pool).await.unwrap();

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/charge"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "succeeded",
            "psp_ref": "ref_idemp"
        })))
        .expect(1) // Should only be called once
        .mount(&mock_server)
        .await;

    let psp_url = format!("{}/charge", mock_server.uri());
    let i_key = "idemp_123";

    let res1 = payment_service::services::payment_service::process_payment(&pool, &psp_url, &b_id, &i_id, "tok_success", i_key).await.unwrap();
    let res2 = payment_service::services::payment_service::process_payment(&pool, &psp_url, &b_id, &i_id, "tok_success", i_key).await.unwrap();

    assert_eq!(res1.id, res2.id);
    assert_eq!(res1.state, res2.state);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_psp_failure_behavior(pool: PgPool) {
    let (b_id, c_id) = setup_test_data(&pool).await;
    let i_id = Ulid::new().to_string();
    
    sqlx::query("INSERT INTO invoices (id, business_id, customer_id, state, total_cents, due_date) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(&i_id).bind(&b_id).bind(&c_id).bind("open").bind(1000).bind(Utc::now())
        .execute(&pool).await.unwrap();

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/charge"))
        .respond_with(ResponseTemplate::new(500)) // Network/Server error
        .mount(&mock_server)
        .await;

    let psp_url = format!("{}/charge", mock_server.uri());

    let res = payment_service::services::payment_service::process_payment(&pool, &psp_url, &b_id, &i_id, "tok_network_error", "key_fail").await.unwrap();
    
    assert_eq!(res.state, "failed");
    let inv = i_dal::find_by_id(&pool, &i_id, &b_id).await.unwrap().unwrap();
    assert_eq!(inv.state, "open", "Invoice should remain open after PSP failure");
}
