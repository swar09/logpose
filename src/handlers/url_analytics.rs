use std::sync::Arc;


use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};


use crate::{
    AppState,
    repository::{url_analytics::get_by_short_code, urls::get_user_id_by_short_code},
    utils::auth::AuthUser,
};


pub async fn get_analytics_by_short_code(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(short_code): Path<String>,
) -> Response {
    let mut conn = state.pool.get().unwrap();
    let user_id = match get_user_id_by_short_code(short_code.clone(), &mut conn) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("{e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if user_id != auth_user.user_id {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let vector = match get_by_short_code(short_code, &mut conn) {
        Ok(vec) => vec,
        Err(e) => {
            eprintln!("{e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    (StatusCode::OK, Json(vector)).into_response()
}

// pub async fn delete_analytics_by_id(State(state): State<Arc<AppState>>, auth_user: AuthUser) {}
