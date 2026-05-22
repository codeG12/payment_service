use crate::dal::business as dal;
use crate::errors::AppError;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};
use axum::{Json, extract::State};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use ulid::Ulid;
#[derive(Deserialize)]
pub struct CreateBusinessRequest {
    pub name: String,
}
#[derive(Serialize)]
pub struct CreateBusinessResponse {
    pub id: String,
    pub name: String,
    pub api_key: String,
}
pub async fn register_business(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateBusinessRequest>,
) -> Result<Json<CreateBusinessResponse>, AppError> {
    let id = Ulid::new().to_string();
    let random_part = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
    let raw_key = format!("sk_{}", random_part);
    let prefix = &raw_key[..8];
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let api_key_hash = argon2
        .hash_password(raw_key.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Hashing failed: {}", e))?
        .to_string();
    dal::insert(&pool, &id, &payload.name, &api_key_hash, prefix).await?;

    Ok(Json(CreateBusinessResponse {
        id,
        name: payload.name,
        api_key: raw_key,
    }))
}
