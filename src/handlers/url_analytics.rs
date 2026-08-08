use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    AppState,
    error::AppError,
    models::auth::AuthUser,
    repository::{url::get_user_id_by_short_code, url_analytics::get_by_short_code},
};

pub async fn get_analytics_by_short_code(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(short_code): Path<String>,
) -> Result<Response, AppError> {
    let mut conn = state.pool.get()?;
    let user_id = get_user_id_by_short_code(short_code.clone(), &mut conn)?;

    if user_id != auth_user.user_id {
        return Err(AppError::Forbidden("Access denied".into()));
    }

    let vector = get_by_short_code(short_code, &mut conn)?;
    Ok((StatusCode::OK, Json(vector)).into_response())
}
