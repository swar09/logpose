use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::query_dsl::methods::{FilterDsl, OrFilterDsl, SelectDsl};
use diesel::{ExpressionMethods, RunQueryDsl};
use jsonwebtoken::EncodingKey;
use uuid::Uuid;

use crate::AppState;
use crate::error::AppError;
use crate::models::auth::{AuthUser, LoginData, LoginRequest, LoginResponse, UserRole};
use crate::repository::user::get_hashed_password_by_id;
use crate::schema::users::{self, email, username};
use crate::utils::auth::{genrate_jwt, verify_password};

const DURATION: usize = 3600;

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

    if !verify_password(&payload.password, &stored_hash) {
        return Ok(unauthorized_response());
    }

    let token = genrate_jwt(
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
