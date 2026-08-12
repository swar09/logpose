use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::fmt::{self};

use crate::service::rate_limiting::RateLimitError;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug)]
pub enum AppError {
    Database(diesel::result::Error),
    Pool(diesel::r2d2::PoolError),
    Redis(redis::RedisError),
    Jwt(jsonwebtoken::errors::Error),
    #[allow(dead_code)]
    NotFound(String),
    Unauthorized,
    Forbidden(String),
    BadRequest(String),
    Internal(String),
    RateLimitError(RateLimitError),
}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "Database error: {e}"),
            AppError::Pool(e) => write!(f, "Pool error: {e}"),
            AppError::Redis(e) => write!(f, "Redis error: {e}"),
            AppError::Jwt(e) => write!(f, "JWT error: {e}"),
            AppError::NotFound(msg) => write!(f, "Not found: {msg}"),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {msg}"),
            AppError::BadRequest(msg) => write!(f, "Bad request: {msg}"),
            AppError::Internal(msg) => write!(f, "Internal error: {msg}"),
            AppError::RateLimitError(e) => write!(f, "Rate limit error {0}", e.msg),
        }
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<diesel::r2d2::PoolError> for AppError {
    fn from(err: diesel::r2d2::PoolError) -> Self {
        AppError::Pool(err)
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::Redis(err)
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::Jwt(err)
    }
}

impl From<RateLimitError> for AppError {
    fn from(err: RateLimitError) -> Self {
        AppError::RateLimitError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(e) => {
                tracing::error!("Database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            }
            AppError::Pool(e) => {
                tracing::error!("Connection pool error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Service temporarily unavailable".to_string(),
                )
            }
            AppError::Redis(e) => {
                tracing::error!("Redis error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Cache error".to_string())
            }
            AppError::Jwt(e) => {
                tracing::error!("JWT error: {e}");
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid authentication token".to_string(),
                )
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized access".to_string()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::RateLimitError(e) => (StatusCode::TOO_MANY_REQUESTS, e.msg),
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_status_codes() {
        let not_found_res = AppError::NotFound("URL not found".into()).into_response();
        assert_eq!(not_found_res.status(), StatusCode::NOT_FOUND);

        let unauthorized_res = AppError::Unauthorized.into_response();
        assert_eq!(unauthorized_res.status(), StatusCode::UNAUTHORIZED);

        let forbidden_res = AppError::Forbidden("Denied".into()).into_response();
        assert_eq!(forbidden_res.status(), StatusCode::FORBIDDEN);

        let bad_req_res = AppError::BadRequest("Invalid payload".into()).into_response();
        assert_eq!(bad_req_res.status(), StatusCode::BAD_REQUEST);

        let internal_res = AppError::Internal("Something broke".into()).into_response();
        assert_eq!(internal_res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
