use super::{jwt, password, AuthUser};
use crate::{error::ApiError, state::AppState};
use axum::{extract::State, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use db::models::UserRow;
use domain::InstanceRole;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub instance_role: InstanceRole,
}

impl From<UserRow> for UserPublic {
    fn from(u: UserRow) -> Self {
        Self {
            id: u.id,
            email: u.email,
            display_name: u.display_name,
            instance_role: u.instance_role,
        }
    }
}

#[derive(Serialize)]
pub struct SessionResponse {
    pub access_token: String,
    pub user: UserPublic,
}

const REFRESH_COOKIE: &str = "refresh_token";
const REFRESH_COOKIE_PATH: &str = "/api/auth";

fn refresh_cookie(state: &AppState, token: String) -> Cookie<'static> {
    let mut cookie = Cookie::new(REFRESH_COOKIE, token);
    cookie.set_path(REFRESH_COOKIE_PATH);
    cookie.set_http_only(true);
    cookie.set_secure(state.cookie_secure);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_max_age(time::Duration::days(state.refresh_token_ttl_days));
    cookie
}

fn issue_session(
    state: &AppState,
    jar: CookieJar,
    user: UserRow,
) -> Result<(CookieJar, SessionResponse), ApiError> {
    let access = jwt::issue_access_token(
        user.id,
        user.instance_role,
        &state.jwt_secret,
        state.access_token_ttl_minutes,
    )
    .map_err(|e| ApiError::Internal(e.into()))?;
    let refresh =
        jwt::issue_refresh_token(user.id, &state.jwt_secret, state.refresh_token_ttl_days)
            .map_err(|e| ApiError::Internal(e.into()))?;
    let jar = jar.add(refresh_cookie(state, refresh));
    Ok((
        jar,
        SessionResponse {
            access_token: access,
            user: user.into(),
        },
    ))
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<RegisterRequest>,
) -> Result<(CookieJar, Json<SessionResponse>), ApiError> {
    let email = normalize_email(&req.email);
    if !email.contains('@') || email.len() > 320 {
        return Err(ApiError::BadRequest("invalid email".into()));
    }
    if req.password.len() < 8 {
        return Err(ApiError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }
    let display_name = req.display_name.trim();
    if display_name.is_empty() {
        return Err(ApiError::BadRequest("display name is required".into()));
    }

    let hash = password::hash(&req.password).map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;
    let user =
        db::users::create(&state.pool, &email, &hash, display_name, InstanceRole::User).await?;

    let (jar, resp) = issue_session(&state, jar, user)?;
    Ok((jar, Json(resp)))
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, Json<SessionResponse>), ApiError> {
    let email = normalize_email(&req.email);
    let user = db::users::find_by_email(&state.pool, &email)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    if !password::verify(&req.password, &user.password_hash) {
        return Err(ApiError::Unauthorized);
    }
    let (jar, resp) = issue_session(&state, jar, user)?;
    Ok((jar, Json(resp)))
}

pub async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<SessionResponse>), ApiError> {
    let token = jar
        .get(REFRESH_COOKIE)
        .map(|c| c.value().to_string())
        .ok_or(ApiError::Unauthorized)?;
    let claims =
        jwt::decode_token(&token, &state.jwt_secret).map_err(|_| ApiError::Unauthorized)?;
    if claims.typ != jwt::TokenType::Refresh {
        return Err(ApiError::Unauthorized);
    }
    let user = db::users::find_by_id(&state.pool, claims.sub)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    let (jar, resp) = issue_session(&state, jar, user)?;
    Ok((jar, Json(resp)))
}

pub async fn logout(jar: CookieJar) -> CookieJar {
    let mut cookie = Cookie::from(REFRESH_COOKIE);
    cookie.set_path(REFRESH_COOKIE_PATH);
    jar.remove(cookie)
}

pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserPublic>, ApiError> {
    let user = db::users::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(user.into()))
}
