use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub async fn not_found() -> Response {
    StatusCode::NOT_FOUND.into_response()
}
