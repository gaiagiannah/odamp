use axum::extract::{Request, Next};
use axum::http::StatusCode;
use axum::response::Response;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub sub: String,  // user ID
    pub exp: i64,
}

pub async fn jwt_auth(mut req: Request, next: Next) -> Result<Response, (StatusCode, String)> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".into()))?;

    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev_secret_change_me_min_32_chars_long".into());

    let data = decode::<Claims>(
        auth_header,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))?;

    // Inject user ID into request extensions
    req.extensions_mut().insert(data.claims.sub);

    Ok(next.run(req).await)
}   