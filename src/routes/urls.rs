use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    AppState, handlers::urls::{create_url, get_url_data_by_shortcode, redirect_url_by_short_code},
};

pub fn redirect_url_routes() -> Router<Arc<AppState>> {
    Router::new().route("/{:short_code}", get(redirect_url_by_short_code))
}

pub fn v1_routes_urls() -> Router<Arc<AppState>> {
    Router::new().route("/", post(create_url))
    .route("/{:short_code}", get(get_url_data_by_shortcode))
    // .route("", post(create_url))
    // .route("", post(create_url))
}
