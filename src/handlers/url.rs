use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json,
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    AppState,
    error::AppError,
    models::{
        auth::AuthUser,
        url::{NewUrl, NewUrlRequest, UpdateCode, UpdateUrl, UpdateUrlRequest},
    },
    repository::url::{
        create, delete_by_short_code, get_by_short_code, get_long_url_by_id, modify_code_by_id,
        modify_url_by_id,
    },
    utils::base62::encode,
};

pub async fn create_url(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(payload): Json<NewUrlRequest>,
) -> Result<Response, AppError> {
    if auth_user.user_id != payload.created_by {
        return Err(AppError::Forbidden(
            "Cannot create URL for another user".into(),
        ));
    }
    if payload.long_url.len() > 2048 {
        return Err(AppError::BadRequest(
            "URL exceeds maximum length of 2048 characters".into(),
        ));
    }
    let mut conn = state.pool.get()?;
    let new_url = NewUrl {
        long_url: &payload.long_url,
        created_by: Some(payload.created_by),
        guest_id: None,
        expires_at: None,
    };

    let mut url = create(new_url, &mut conn)?;
    let short_code_str = encode(url.database_id as u32, &state.ff)?;
    let updated_code = UpdateCode {
        short_code: &short_code_str,
    };

    modify_code_by_id(url.database_id, updated_code, &mut conn)?;
    url.short_code = Some(short_code_str);

    Ok((StatusCode::CREATED, Json(url)).into_response())
}

pub async fn get_url_data_by_shortcode(
    State(state): State<Arc<AppState>>,
    Path(short_code): Path<String>,
) -> Result<Response, AppError> {
    let mut conn = state.pool.get()?;
    let url = get_by_short_code(short_code, &mut conn)?;
    Ok((StatusCode::OK, Json(url)).into_response())
}

pub async fn redirect_url_by_short_code(
    State(state): State<Arc<AppState>>,
    Path(short_code): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Redirect {
    let long_url = match state.url_service.get_url(short_code.clone()).await {
        Ok(url) => url,
        Err(e) => {
            tracing::error!("{e}");
            return Redirect::temporary("/404");
        }
    };

    let pool = state.pool.clone();
    let sc = short_code.clone();
    tokio::task::spawn_blocking(move || {
        if let Ok(mut conn) = pool.get() {
            crate::utils::analytics::create_analytics(addr, &headers, &mut conn, sc);
        }
    });

    Redirect::temporary(&long_url)
}

pub async fn update_url(
    State(state): State<Arc<AppState>>,
    Path(short_code): Path<String>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateUrlRequest>,
) -> Result<Response, AppError> {
    if payload.long_url.len() > 2048 {
        return Err(AppError::BadRequest(
            "URL exceeds maximum length of 2048 characters".into(),
        ));
    }
    let mut conn = state.pool.get()?;

    let existing_url = get_by_short_code(short_code.clone(), &mut conn)?;
    if existing_url.created_by != Some(auth_user.user_id) {
        return Err(AppError::Forbidden(
            "You are not allowed to update this URL".into(),
        ));
    }

    let update_url = UpdateUrl {
        long_url: &payload.long_url,
    };
    modify_url_by_id(payload.database_id, update_url, &mut conn)?;
    let long_url = get_long_url_by_id(payload.database_id, &mut conn)?;

    // Invalidate the cache
    if let Err(e) = state.redis_store.delete_url(short_code).await {
        tracing::warn!("Failed to delete cached URL from Redis: {e}");
    }

    Ok((StatusCode::OK, Json(long_url)).into_response())
}

pub async fn delete_url(
    State(state): State<Arc<AppState>>,
    Path(short_code): Path<String>,
    auth_user: AuthUser,
) -> Result<Response, AppError> {
    let mut conn = state.pool.get()?;

    let existing_url = get_by_short_code(short_code.clone(), &mut conn)?;
    if existing_url.created_by != Some(auth_user.user_id) {
        return Err(AppError::Forbidden(
            "You are not allowed to delete this URL".into(),
        ));
    }

    delete_by_short_code(short_code.clone(), &mut conn)?;

    // Invalidate the cache
    if let Err(e) = state.redis_store.delete_url(short_code).await {
        tracing::warn!("Failed to delete cached URL from Redis: {e}");
    }

    Ok(StatusCode::OK.into_response())
}
