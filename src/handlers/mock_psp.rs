use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use ulid::Ulid;
#[derive(Deserialize)]
pub struct MockPspRequest {
    pub token: String,
    pub amount_cents: i64,
}
#[derive(Serialize)]
pub struct MockPspResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psp_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}
pub async fn charge(Json(payload): Json<MockPspRequest>) -> impl IntoResponse {
    match payload.token.as_str() {
        "tok_success" => {
            tokio::time::sleep(Duration::from_millis(100)).await;
            (
                StatusCode::OK,
                Json(MockPspResponse {
                    status: "succeeded".into(),
                    psp_ref: Some(Ulid::new().to_string()),
                    code: None,
                }),
            )
                .into_response()
        }
        "tok_insufficient_funds" => {
            tokio::time::sleep(Duration::from_millis(100)).await;
            (
                StatusCode::OK,
                Json(MockPspResponse {
                    status: "failed".into(),
                    psp_ref: None,
                    code: Some("insufficient_funds".into()),
                }),
            )
                .into_response()
        }
        "tok_card_declined" => {
            tokio::time::sleep(Duration::from_millis(100)).await;
            (
                StatusCode::OK,
                Json(MockPspResponse {
                    status: "failed".into(),
                    psp_ref: None,
                    code: Some("card_declined".into()),
                }),
            )
                .into_response()
        }
        "tok_timeout" => {
            tokio::time::sleep(Duration::from_secs(30)).await;
            (
                StatusCode::OK,
                Json(MockPspResponse {
                    status: "succeeded".into(),
                    psp_ref: Some(Ulid::new().to_string()),
                    code: None,
                }),
            )
                .into_response()
        }
        "tok_network_error" => (StatusCode::INTERNAL_SERVER_ERROR, "Network Error").into_response(),
        _ => (StatusCode::BAD_REQUEST, "Invalid Token").into_response(),
    }
}
