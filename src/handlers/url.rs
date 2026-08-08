use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json,
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};

use crate::repository::url::{delete_by_short_code, get_long_url_by_id, modify_url_by_id};
use crate::{
    AppState,
    models::{
        auth::AuthUser,
        url::{NewUrl, NewUrlRequest, UpdateCode, UpdateUrl, UpdateUrlRequest},
    },
    repository::url::{create, get_by_short_code, modify_code_by_id},
    utils::base62::encode,
};
pub async fn create_url(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(payload): Json<NewUrlRequest>,
) -> Response {
    if auth_user.user_id != payload.created_by {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let mut conn = state.pool.get().unwrap();
    let new_url = NewUrl {
        // short_code: &payload.short_code,
        long_url: &payload.long_url,
        created_by: payload.created_by,
    };

    let mut url = match create(new_url, &mut conn) {
        Ok(url) => url,
        Err(e) => {
            eprint!("{e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let short_code_str = encode(url.database_id as u32, &state.ff);
    let updated_code = UpdateCode {
        short_code: &short_code_str,
    };

    let result = modify_code_by_id(url.database_id, updated_code, &mut conn);

    match result {
        Ok(_) => {
            url.short_code = Some(short_code_str);
            (StatusCode::CREATED, Json(url)).into_response()
        }
        Err(e) => {
            eprintln!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_url_data_by_shortcode(
    State(state): State<Arc<AppState>>,
    Path(short_code): Path<String>,
) -> Response {
    let mut conn = state.pool.get().unwrap();
    let url = match get_by_short_code(short_code, &mut conn) {
        Ok(url) => url,
        Err(e) => {
            eprintln!("{e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    (StatusCode::OK, Json(url)).into_response()
}
pub async fn redirect_url_by_short_code(
    State(state): State<Arc<AppState>>,
    Path(short_code): Path<String>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    _headers: HeaderMap,
) -> Redirect {
    let long_url = match state.url_service.get_url(short_code).await {
        Ok(url) => url,
        Err(e) => {
            eprintln!("{e}");
            return Redirect::temporary("/404");
        }
    };
    Redirect::temporary(&long_url)
}

pub async fn update_url(
    State(state): State<Arc<AppState>>,
    Path(_short_code): Path<String>,
    _auth_user: AuthUser,
    Json(payload): Json<UpdateUrlRequest>,
) -> Response {
    let mut conn = state.pool.get().unwrap();

    let update_url = UpdateUrl {
        long_url: &payload.long_url,
    };
    let result = modify_url_by_id(payload.database_id, update_url, &mut conn);
    match result {
        Ok(_) => {
            let _updated_url = match get_long_url_by_id(payload.database_id, &mut conn) {
                Ok(long_url) => {
                    return (StatusCode::OK, Json(long_url)).into_response();
                }
                Err(_) => {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            };
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn delete_url(
    State(state): State<Arc<AppState>>,
    Path(short_code): Path<String>,
    _auth_user: AuthUser,
) -> Response {
    let mut conn = state.pool.get().unwrap();
    match delete_by_short_code(short_code, &mut conn) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
