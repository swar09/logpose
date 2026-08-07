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
    models::{
        auth::AuthUser,
        user::{
            NewUser, NewUserRequest, UpdatePassword, UpdatePasswordRequest, UpdateRequest,
            UpdateUser,
        },
    },
    repository::user::{
        create, get_by_id, get_hashed_password_by_id, update_by_id, update_password_by_id,
    },
    utils::auth::hash_password,
};

pub async fn signup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NewUserRequest>,
) -> Response {
    let hashed_password = match hash_password(payload.password) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("{e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let new_user = NewUser {
        email: &payload.email,
        first_name: &payload.first_name,
        last_name: &payload.last_name,
        username: &payload.username,
        hashed_password: &hashed_password,
    };

    let mut conn = state.clone().pool.get().unwrap();
    match create(&mut conn, &new_user) {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn get_urls(
    State(state): State<Arc<AppState>>,
    Path(path_id): Path<Uuid>,
    auth_user: AuthUser,
) -> Response {
    if path_id != auth_user.user_id {
        return StatusCode::FORBIDDEN.into_response();
    }
    let mut conn = state.pool.get().unwrap();

    let urls_result = crate::repository::url::get_urls_by_user_id(path_id, &mut conn);
    match urls_result {
        Ok(urls) => (StatusCode::OK, Json(urls)).into_response(),
        Err(e) => {
            println!("DATABASE ERROR : {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(path_id): Path<Uuid>,
    auth_user: AuthUser,
) -> Response {
    if path_id != auth_user.user_id {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let mut conn = state.pool.get().unwrap();
    let user = match get_by_id(auth_user.user_id, &mut conn) {
        Ok(user) => user,
        Err(e) => {
            eprintln!("{e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    (StatusCode::OK, Json(user)).into_response()
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
) -> Response {
    if path_id != auth_user.user_id {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let mut conn = state.pool.get().unwrap();

    let updated_user_data = UpdateUser {
        username: &payload.username,
        last_name: &payload.last_name,
        first_name: &payload.first_name,
    };

    match update_by_id(&mut conn, auth_user.user_id, &updated_user_data) {
        Ok(user_data) => (StatusCode::OK, Json(user_data)).into_response(),
        Err(e) => {
            eprintln!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
pub async fn update_user_password(
    State(state): State<Arc<AppState>>,
    Path(path_id): Path<Uuid>,
    auth_user: AuthUser,
    Json(payload): Json<UpdatePasswordRequest>,
) -> Response {
    if path_id != auth_user.user_id {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let mut conn = state.pool.get().unwrap();

    let given_hashed_password = match hash_password(payload.old_password) {
        Ok(hashed_password) => hashed_password,
        Err(e) => {
            eprintln!("{e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let old_hashed_password = match get_hashed_password_by_id(auth_user.user_id, &mut conn) {
        Ok(hashed_password) => hashed_password,
        Err(e) => {
            eprintln!("{e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if old_hashed_password == given_hashed_password {
        let new_hashed_password = match hash_password(payload.new_password) {
            Ok(hashed_password) => hashed_password,
            Err(_e) => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

        let updated_password = UpdatePassword {
            hashed_password: &new_hashed_password,
        };
        let result = update_password_by_id(&mut conn, auth_user.user_id, updated_password);

        match result {
            Ok(_i) => {
                return StatusCode::OK.into_response();
            }
            Err(e) => {
                eprintln!("{e}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }

    StatusCode::UNAUTHORIZED.into_response()
}
