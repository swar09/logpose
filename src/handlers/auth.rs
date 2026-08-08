use std::sync::Arc;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::query_dsl::methods::{FilterDsl, OrFilterDsl, SelectDsl};
use diesel::{ExpressionMethods, RunQueryDsl};
use uuid::Uuid;

use axum::extract::State;
use jsonwebtoken::EncodingKey;

use crate::AppState;
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
) -> Response {
    let mut conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("pool error: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let user_id: Uuid = match users::table
        .filter(email.eq(&payload.login_id))
        .or_filter(username.eq(&payload.login_id))
        .select(users::id)
        .first(&mut conn)
    {
        Ok(id) => id,
        Err(_) => return unauthorized_response(),
    };

    let stored_hash = match get_hashed_password_by_id(user_id, &mut conn) {
        Ok(hash) => hash,
        Err(_) => return unauthorized_response(),
    };

    if !verify_password(&payload.password, &stored_hash) {
        return unauthorized_response();
    }

    let token = match genrate_jwt(
        UserRole::Client,
        user_id,
        EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("jwt error: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let data = LoginData {
        user: UserRole::Client,
        token,
        token_type: String::from("bearer"),
        expires_in: chrono::Utc::now().timestamp() as usize + DURATION,
    };

    (
        StatusCode::OK,
        Json(LoginResponse {
            success: true,
            data: Some(data),
        }),
    )
        .into_response()
}

// jwt expired , cookie free , redirect to home page
pub async fn logout(State(state): State<Arc<AppState>>, auth_user: AuthUser) -> Response {
    let exp_rsec = auth_user.exp - chrono::Utc::now().timestamp() as u64;

    let result = state
        .redis_store
        .clone()
        .blacklist(auth_user.jti, exp_rsec)
        .await;

    match result {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            eprintln!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
