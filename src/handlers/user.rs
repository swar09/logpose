use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    AppState,
    error::AppError,
    models::{
        auth::AuthUser,
        user::{
            NewUser, NewUserRequest, UpdatePassword, UpdatePasswordRequest, UpdateRequest,
            UpdateUser, UserResponse,
        },
    },
    repository::user::{
        create, delete_by_id, get_by_id, get_hashed_password_by_id, update_by_id,
        update_password_by_id,
    },
    utils::auth::{hash_password, verify_password},
};

pub async fn signup(
    State(state): State<Arc<AppState>>,
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
    };

    let mut conn = state.pool.get()?;
    let user = create(&mut conn, &new_user)?;
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
    let user = get_by_id(auth_user.user_id, &mut conn)?;
    let user_response = UserResponse::from(user);

    Ok((StatusCode::OK, Json(user_response)).into_response())
}

pub async fn get_subscription_by_id(
    State(_state): State<Arc<AppState>>,
    Path(_path_id): Path<Uuid>,
) -> Response {
    StatusCode::NOT_IMPLEMENTED.into_response()
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

    let mut conn = state.pool.get()?;
    let updated_user_data = UpdateUser {
        username: &payload.username,
        last_name: &payload.last_name,
        first_name: &payload.first_name,
    };

    let user_data = update_by_id(&mut conn, auth_user.user_id, &updated_user_data)?;
    let user_response = UserResponse::from(user_data);
    Ok((StatusCode::OK, Json(user_response)).into_response())
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
    let old_hashed_password = get_hashed_password_by_id(auth_user.user_id, &mut conn)?;

    if !verify_password(&payload.old_password, &old_hashed_password) {
        return Err(AppError::Unauthorized);
    }

    let new_hashed_password = hash_password(payload.new_password)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {e}")))?;

    let updated_password = UpdatePassword {
        hashed_password: &new_hashed_password,
    };
    update_password_by_id(&mut conn, auth_user.user_id, updated_password)?;

    Ok(StatusCode::OK.into_response())
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
    delete_by_id(&mut conn, auth_user.user_id)?;

    Ok(StatusCode::OK.into_response())
}
