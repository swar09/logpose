use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json,
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    AppState,
    error::AppError,
    models::{
        auth::AuthUser,
        url::{
            NewUrl, NewUrlRequest, PublicNewUrlRequest, UpdateCode, UpdateUrl, UpdateUrlRequest,
            Urls,
        },
    },
    repository::{
        billing::{get_active_subscription_by_user_id, get_plan_by_code, get_plan_by_id},
        url::{
            check_short_code_exists, count_urls_by_user_id, create, delete_by_short_code,
            get_active_urls_by_guest_id, get_by_short_code, get_long_url_by_id, modify_code_by_id,
            modify_url_by_id,
        },
    },
    utils::{alias::validate_custom_alias, base62::encode},
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

    let plan = match get_active_subscription_by_user_id(auth_user.user_id, &mut conn)? {
        Some(sub) => get_plan_by_id(sub.plan_id, &mut conn)?,
        None => {
            get_plan_by_code("plan_free", &mut conn).or_else(|_| get_plan_by_id(1, &mut conn))?
        }
    };

    let user_url_count = count_urls_by_user_id(auth_user.user_id, &mut conn)?;
    if user_url_count >= plan.max_urls_limit as i64 {
        return Err(AppError::Forbidden(
            "Plan URL limit reached. Upgrade to create more URLs".into(),
        ));
    }

    if let Some(ref alias) = payload.custom_alias {
        if !plan.custom_alias_allowed {
            return Err(AppError::Forbidden(
                "Custom aliases are not allowed on your current plan".into(),
            ));
        }

        let valid_alias = validate_custom_alias(alias)?;

        if check_short_code_exists(&valid_alias, &mut conn)? {
            return Err(AppError::BadRequest("Custom alias is already taken".into()));
        }

        let new_url = NewUrl {
            long_url: &payload.long_url,
            short_code: Some(&valid_alias),
            created_by: Some(payload.created_by),
            guest_id: None,
            expires_at: None,
        };

        let url = create(new_url, &mut conn)?;
        return Ok((StatusCode::CREATED, Json(url)).into_response());
    }

    let new_url = NewUrl {
        long_url: &payload.long_url,
        short_code: None,
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

pub async fn create_public_url(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<PublicNewUrlRequest>,
) -> Result<Response, AppError> {
    if payload.long_url.len() > 2048 {
        return Err(AppError::BadRequest(
            "URL exceeds maximum length of 2048 characters".into(),
        ));
    }

    let (guest_id, should_set_cookie) = match extract_guest_id(&headers) {
        Some(id) => (id, false),
        None => (Uuid::new_v4(), true),
    };

    let mut conn = state.pool.get()?;
    let expires_at_time = Utc::now() + chrono::Duration::hours(24);
    let new_url = NewUrl {
        long_url: &payload.long_url,
        short_code: None,
        created_by: None,
        guest_id: Some(guest_id),
        expires_at: Some(expires_at_time),
    };

    let mut url = create(new_url, &mut conn)?;
    let short_code_str = encode(url.database_id as u32, &state.ff)?;
    let updated_code = UpdateCode {
        short_code: &short_code_str,
    };

    modify_code_by_id(url.database_id, updated_code, &mut conn)?;
    url.short_code = Some(short_code_str);

    let mut response = (StatusCode::CREATED, Json(url)).into_response();
    if should_set_cookie {
        let cookie_val = format!(
            "guest_id={}; Path=/; Max-Age=86400; HttpOnly; SameSite=Lax",
            guest_id
        );
        if let Ok(header_val) = axum::http::HeaderValue::from_str(&cookie_val) {
            response
                .headers_mut()
                .insert(header::SET_COOKIE, header_val);
        }
    }

    Ok(response)
}

pub async fn get_my_guest_urls(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let guest_id = match extract_guest_id(&headers) {
        Some(id) => id,
        None => {
            return Ok((StatusCode::OK, Json(Vec::<Urls>::new())).into_response());
        }
    };

    let mut conn = state.pool.get()?;
    let urls = get_active_urls_by_guest_id(guest_id, &mut conn)?;
    Ok((StatusCode::OK, Json(urls)).into_response())
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

    if let Err(e) = state.redis_store.delete_url(short_code).await {
        tracing::warn!("Failed to delete cached URL from Redis: {e}");
    }

    Ok(StatusCode::OK.into_response())
}
