use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::{
    AppState,
    handlers::{
        url::{
            create_public_url, create_url, delete_url, generate_generic_qr, get_my_guest_urls,
            get_short_code_qr, get_url_data_by_shortcode, redirect_url_by_short_code, update_url,
        },
        url_analytics::get_analytics_by_short_code,
    },
};

pub fn v1_routes_urls() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_url))
        .route("/qr", get(generate_generic_qr))
        .route("/public/shorten", post(create_public_url))
        .route("/public/my-urls", get(get_my_guest_urls))
        .route("/{short_code}", get(get_url_data_by_shortcode))
        .route("/{short_code}", post(update_url))
        .route("/{short_code}", delete(delete_url))
        .route("/{short_code}/analytics", get(get_analytics_by_short_code))
        .route("/{short_code}/qr", get(get_short_code_qr))
}

pub fn v1_routes_public() -> Router<Arc<AppState>> {
    Router::new()
        .route("/shorten", post(create_public_url))
        .route("/my-urls", get(get_my_guest_urls))
        .route("/qr", get(generate_generic_qr))
}

pub fn redirect_url_routes() -> Router<Arc<AppState>> {
    Router::new().route("/{short_code}", get(redirect_url_by_short_code))
}
