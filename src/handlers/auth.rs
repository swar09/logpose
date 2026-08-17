use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use diesel::query_dsl::methods::{FilterDsl, OrFilterDsl, SelectDsl};
use diesel::{ExpressionMethods, RunQueryDsl};
use jsonwebtoken::EncodingKey;
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use crate::error::AppError;
use crate::models::auth::{AuthUser, LoginData, LoginRequest, LoginResponse, UserRole};
use crate::models::user::{NewUser, UpdateOAuthProfile};
use crate::repository::user::{
    create, get_by_email, get_by_google_id, get_hashed_password_by_id, update_oauth_profile,
};
use crate::schema::users::{self, email, username};
use crate::utils::auth::{generate_jwt, verify_password};

const DURATION: usize = 3600;

#[derive(Deserialize)]
pub struct GoogleCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub email_verified: Option<bool>,
}

fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(LoginResponse {
            success: false,
            data: None,
        }),
    )
        .into_response()
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Response, AppError> {
    let mut conn = state.pool.get()?;

    let user_id: Uuid = match users::table
        .filter(email.eq(&payload.login_id))
        .or_filter(username.eq(&payload.login_id))
        .select(users::id)
        .first(&mut conn)
    {
        Ok(id) => id,
        Err(_) => return Ok(unauthorized_response()),
    };

    let stored_hash = match get_hashed_password_by_id(user_id, &mut conn) {
        Ok(hash) => hash,
        Err(_) => return Ok(unauthorized_response()),
    };

    if stored_hash.is_empty() || !verify_password(&payload.password, &stored_hash) {
        return Ok(unauthorized_response());
    }

    let token = generate_jwt(
        UserRole::Client,
        user_id,
        EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )?;

    let data = LoginData {
        user: UserRole::Client,
        token,
        token_type: String::from("bearer"),
        expires_in: chrono::Utc::now().timestamp() as usize + DURATION,
    };

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            success: true,
            data: Some(data),
        }),
    )
        .into_response())
}

pub async fn google_login(State(state): State<Arc<AppState>>) -> Result<Redirect, AppError> {
    let (auth_url, csrf_token) = state
        .google_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_extra_param("access_type", "offline")
        .add_extra_param("prompt", "select_account")
        .url();

    state
        .redis_store
        .set_oauth_state(csrf_token.secret(), 300)
        .await?;

    Ok(Redirect::temporary(auth_url.as_str()))
}

pub async fn google_callback(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<GoogleCallbackQuery>,
) -> Result<Response, AppError> {
    if let Some(err) = query.error {
        return Err(AppError::BadRequest(format!("Google OAuth error: {err}")));
    }

    let state_param = query
        .state
        .ok_or_else(|| AppError::BadRequest("Missing OAuth state parameter".into()))?;
    let code_param = query
        .code
        .ok_or_else(|| AppError::BadRequest("Missing OAuth authorization code".into()))?;

    let is_valid_state = state
        .redis_store
        .verify_and_consume_oauth_state(&state_param)
        .await?;
    if !is_valid_state {
        return Err(AppError::BadRequest(
            "Invalid or expired OAuth state parameter".into(),
        ));
    }

    let token_res = state
        .google_client
        .exchange_code(AuthorizationCode::new(code_param))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to exchange authorization code: {e}")))?;

    let access_token = token_res.access_token().secret();

    let client = reqwest::Client::new();
    let user_info: GoogleUserInfo = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch user profile from Google: {e}")))?
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse Google user profile: {e}")))?;

    let mut conn = state.pool.get()?;

    let existing_user = get_by_google_id(&user_info.sub, &mut conn)
        .ok()
        .or_else(|| get_by_email(&user_info.email, &mut conn).ok());

    let user = match existing_user {
        Some(existing) => {
            let first = user_info
                .given_name
                .as_deref()
                .unwrap_or(&existing.first_name);
            let last = user_info
                .family_name
                .as_deref()
                .unwrap_or(&existing.last_name);
            let update_profile = UpdateOAuthProfile {
                first_name: first,
                last_name: last,
                avatar_url: user_info.picture.as_deref(),
                google_id: Some(&user_info.sub),
            };
            update_oauth_profile(existing.id, &update_profile, &mut conn)?
        }
        None => {
            let first = user_info.given_name.as_deref().unwrap_or("User");
            let last = user_info.family_name.as_deref().unwrap_or("");
            let email_prefix = user_info.email.split('@').next().unwrap_or("user");
            let raw_username = format!("{}_{}", email_prefix, Uuid::new_v4().simple());
            let username_val = &raw_username[..raw_username.len().min(30)];
            let new_user = NewUser {
                email: &user_info.email,
                first_name: first,
                last_name: last,
                username: username_val,
                hashed_password: "",
                avatar_url: user_info.picture.as_deref(),
                google_id: Some(&user_info.sub),
                auth_provider: "google",
            };
            create(&mut conn, &new_user)?
        }
    };

    if let Some(cookie_header) = headers.get(header::COOKIE)
        && let Ok(cookie_str) = cookie_header.to_str()
    {
        for cookie_pair in cookie_str.split(';') {
            let parts: Vec<&str> = cookie_pair.trim().split('=').collect();
            if parts.len() == 2
                && parts[0] == "guest_id"
                && let Ok(guest_uuid) = Uuid::parse_str(parts[1])
            {
                let _ = crate::repository::url::claim_guest_urls(guest_uuid, user.id, &mut conn);
            }
        }
    }

    let token = generate_jwt(
        UserRole::Client,
        user.id,
        EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )?;

    let data = LoginData {
        user: UserRole::Client,
        token,
        token_type: String::from("bearer"),
        expires_in: chrono::Utc::now().timestamp() as usize + DURATION,
    };

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            success: true,
            data: Some(data),
        }),
    )
        .into_response())
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<Response, AppError> {
    let now = chrono::Utc::now().timestamp() as u64;
    let exp_rsec = auth_user.exp.saturating_sub(now);

    state
        .redis_store
        .clone()
        .blacklist(auth_user.jti, exp_rsec)
        .await?;

    Ok(StatusCode::OK.into_response())
}
