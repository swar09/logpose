use crate::{AppState, utils::auth::AuthUser};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn test() -> Json<String> {
    eprintln!("handler is called ");

    Json::from(String::from("Test"))
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

pub async fn get_subscription_by_id(
    State(_state): State<Arc<AppState>>,
    Path(_path_id): Path<Uuid>,
) -> Response {
    StatusCode::NOT_IMPLEMENTED.into_response()
}
