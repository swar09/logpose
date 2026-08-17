use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    AppState,
    billing::callback::UserSubscriptionResponse,
    error::AppError,
    models::{
        auth::AuthUser,
        user::{
            NewUser, NewUserRequest, UpdatePassword, UpdatePasswordRequest, UpdateRequest,
            UpdateUser, UserResponse,
        },
    },
    repository::{
        billing::{get_active_subscription_by_user_id, get_plan_by_code, get_plan_by_id},
        user::{
            create, delete_by_id, get_by_id, get_hashed_password_by_id, update_by_id,
            update_password_by_id,
        },
    },
    utils::auth::{hash_password, verify_password},
};

fn extract_guest_id(headers: &HeaderMap) -> Option<Uuid> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    for cookie_pair in cookie_header.split(';') {
        let parts: Vec<&str> = cookie_pair.trim().split('=').collect();
        if parts.len() == 2 && parts[0] == "guest_id" {
            return Uuid::parse_str(parts[1]).ok();
        }
    }
    None
}

pub async fn signup(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<NewUserRequest>,
) -> Result<Response, AppError> {
    let hashed_password = hash_password(payload.password)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {e}")))?;

    let new_user = NewUser {
        email: &payload.email,
        first_name: &payload.first_name,
        last_name: &payload.last_name,
        username: &payload.username,
        hashed_password: &hashed_password,
        avatar_url: None,
        google_id: None,
        auth_provider: "local",
    };

    let mut conn = state.pool.get()?;
    let user = create(&mut conn, &new_user)?;

    if let Some(guest_id) = extract_guest_id(&headers) {
        let _ = crate::repository::url::claim_guest_urls(guest_id, user.id, &mut conn);
    }

    let user_response = UserResponse::from(user);
    Ok((StatusCode::CREATED, Json(user_response)).into_response())
}

pub async fn get_urls(
    State(state): State<Arc<AppState>>,
    Path(path_id): Path<Uuid>,
    auth_user: AuthUser,
) -> Result<Response, AppError> {
    if path_id != auth_user.user_id {
        return Err(AppError::Forbidden("Access denied".into()));
    }
    let mut conn = state.pool.get()?;
    let urls = crate::repository::url::get_urls_by_user_id(path_id, &mut conn)?;

    Ok((StatusCode::OK, Json(urls)).into_response())
}

pub async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(path_id): Path<Uuid>,
    auth_user: AuthUser,
) -> Result<Response, AppError> {
    if path_id != auth_user.user_id {
        return Err(AppError::Forbidden("Access denied".into()));
    }
    let mut conn = state.pool.get()?;
    let user = get_by_id(path_id, &mut conn)?;

    Ok((StatusCode::OK, Json(UserResponse::from(user))).into_response())
}

pub async fn update_user_info_by_id(
    State(state): State<Arc<AppState>>,
    Path(path_id): Path<Uuid>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateRequest>,
) -> Result<Response, AppError> {
    if path_id != auth_user.user_id {
        return Err(AppError::Forbidden("Access denied".into()));
    }
    let update_user = UpdateUser {
        username: &payload.username,
        first_name: &payload.first_name,
        last_name: &payload.last_name,
    };
    let mut conn = state.pool.get()?;
    let user = update_by_id(&mut conn, path_id, &update_user)?;

    Ok((StatusCode::OK, Json(UserResponse::from(user))).into_response())
}

pub async fn delete_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(path_id): Path<Uuid>,
    auth_user: AuthUser,
) -> Result<Response, AppError> {
    if path_id != auth_user.user_id {
        return Err(AppError::Forbidden("Access denied".into()));
    }
    let mut conn = state.pool.get()?;
    delete_by_id(&mut conn, path_id)?;

    Ok(StatusCode::OK.into_response())
}

pub async fn update_user_password(
    State(state): State<Arc<AppState>>,
    Path(path_id): Path<Uuid>,
    auth_user: AuthUser,
    Json(payload): Json<UpdatePasswordRequest>,
) -> Result<Response, AppError> {
    if path_id != auth_user.user_id {
        return Err(AppError::Forbidden("Access denied".into()));
    }
    let mut conn = state.pool.get()?;

    let stored_hash = get_hashed_password_by_id(path_id, &mut conn)?;
    if !verify_password(&payload.old_password, &stored_hash) {
        return Err(AppError::BadRequest("Old password does not match".into()));
    }

    let hashed_password = hash_password(payload.new_password)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {e}")))?;

    let update_password = UpdatePassword {
        hashed_password: &hashed_password,
    };
    update_password_by_id(&mut conn, path_id, update_password)?;

    Ok(StatusCode::OK.into_response())
}

pub async fn get_subscription_by_id(
    State(state): State<Arc<AppState>>,
    Path(path_id): Path<Uuid>,
    auth_user: AuthUser,
) -> Result<Response, AppError> {
    if path_id != auth_user.user_id {
        return Err(AppError::Forbidden("Access denied".into()));
    }
    let mut conn = state.pool.get()?;
    let active_sub = get_active_subscription_by_user_id(path_id, &mut conn)?;

    let (plan, is_active) = match &active_sub {
        Some(sub) => {
            let p = get_plan_by_id(sub.plan_id, &mut conn)?;
            (p, true)
        }
        None => {
            let p = get_plan_by_code("plan_free", &mut conn)
                .or_else(|_| get_plan_by_id(1, &mut conn))?;
            (p, false)
        }
    };

    let response = UserSubscriptionResponse {
        subscription: active_sub,
        plan,
        is_active,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}
