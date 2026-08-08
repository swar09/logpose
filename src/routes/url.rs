use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::{
    AppState,
    handlers::{
        url::{
            create_url, delete_url, get_url_data_by_shortcode, redirect_url_by_short_code,
            update_url,
        },
        url_analytics::get_analytics_by_short_code,
    },
};

pub fn v1_routes_urls() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_url))
        .route("/{short_code}", get(get_url_data_by_shortcode))
        .route("/{short_code}", post(update_url))
        .route("/{short_code}", delete(delete_url))
        .route("/{short_code}/analytics", get(get_analytics_by_short_code))
}

pub fn redirect_url_routes() -> Router<Arc<AppState>> {
    Router::new().route("/{short_code}", get(redirect_url_by_short_code))
}
