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
    models::users::{NewUser, NewUserRequest},
    repository::user::create,
    utils::auth::{AuthUser, hash_password},
};

pub async fn test() -> Json<String> {
    eprintln!("handler is called ");

    Json::from(String::from("Test"))
}

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
    let mut conn = state.clone().pool.get().unwrap();

    let urls_result = crate::repository::urls::get_urls_by_user_id(path_id, &mut conn);
    match urls_result {
        Ok(urls) => (StatusCode::OK, Json(urls)).into_response(),
        Err(e) => {
            println!("DATABASE ERROR : {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_user_by_id() {}
pub async fn get_subscription_by_id(
    State(_state): State<Arc<AppState>>,
    Path(_path_id): Path<Uuid>,
) -> Response {
    StatusCode::NOT_IMPLEMENTED.into_response()
}
