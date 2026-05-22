use crate::errors::AppError;
use crate::models::business::Business;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use sqlx::PgPool;
pub struct AuthenticatedBusiness(pub Business);
#[async_trait]
impl FromRequestParts<PgPool> for AuthenticatedBusiness {
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, pool: &PgPool) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;
        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized);
        }
        let raw_key = &auth_header[7..];
        if raw_key.len() < 8 {
            return Err(AppError::Unauthorized);
        }
        let prefix = &raw_key[..8];
        let businesses = sqlx::query_as::<_, Business>(
            "SELECT * FROM businesses WHERE api_key_prefix = $1 AND is_active = TRUE",
        )
        .bind(prefix)
        .fetch_all(pool)
        .await?;
        for business in businesses {
            if let Ok(hash) = PasswordHash::new(&business.api_key_hash) {
                if Argon2::default()
                    .verify_password(raw_key.as_bytes(), &hash)
                    .is_ok()
                {
                    return Ok(AuthenticatedBusiness(business));
                }
            }
        }
        Err(AppError::Unauthorized)
    }
}
