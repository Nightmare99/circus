use chrono::{Duration, Utc};
use domain::InstanceRole;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub typ: TokenType,
    pub role: Option<InstanceRole>,
    pub iat: i64,
    pub exp: i64,
}

pub fn issue_access_token(
    user_id: Uuid,
    role: InstanceRole,
    secret: &str,
    ttl_minutes: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        typ: TokenType::Access,
        role: Some(role),
        iat: now.timestamp(),
        exp: (now + Duration::minutes(ttl_minutes)).timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn issue_refresh_token(
    user_id: Uuid,
    secret: &str,
    ttl_days: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        typ: TokenType::Refresh,
        role: None,
        iat: now.timestamp(),
        exp: (now + Duration::days(ttl_days)).timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}
