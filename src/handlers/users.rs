use axum::{
    Error, Json, extract::{Path, State}, http::StatusCode, response::Response, response::IntoResponse
};
use diesel::{PgConnection, r2d2::Pool};
use uuid::Uuid;

use crate::{AppState, models::urls::Urls};

pub async fn test() -> Json<String> {
    eprintln!("handler is called ");

    Json::from(String::from("Test"))
}
use axum::debug_handler;

#[debug_handler(state = crate::AppState)] 
pub async fn get_urls(
    State(state): State<AppState>,
    Path(path_id): Path<Uuid>,
) -> Response {
    // TODO : Middleware checks
    let mut conn = state.pool.get().unwrap();

    let urls_result = crate::repository::urls::get_urls_by_user_id(path_id, &mut conn);
    match urls_result {
        Ok(urls) => {
            (StatusCode::OK, Json(urls)).into_response()
        }
        Err(e) => {
            println!("DATABASE ERROR : {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
