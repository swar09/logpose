use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json,
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use base62::decode;

use crate::{
    AppState,
    models::urls::{NewUrl, NewUrlRequest, UpdateCode},
    repository::urls::{create, get_by_short_code, get_long_url_by_id, modify_code_by_id},
    utils::{analytics::create_analytics, auth::AuthUser, base62::encode},
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Redirect {
    let mut conn = state.pool.get().unwrap();
    let id = match decode(&short_code) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("{e}");
            return Redirect::to("/");
        }
    };
    let long_url = match get_long_url_by_id(id as i32, &mut conn) {
        Ok(long_url) => long_url,
        Err(e) => {
            eprintln!("{e}");
            return Redirect::to("/");
        }
    };

    // fire and forget , user will be redirected without any delay of analytics
    tokio::spawn(async move {
        if !create_analytics(addr, &headers, &mut conn, short_code) {
            eprintln!("Analytics Error")
        }
    });

    Redirect::temporary(&long_url)
}

