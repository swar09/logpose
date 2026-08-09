use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{AppState, handlers::url_analytics::get_analytics_by_short_code};

#[allow(dead_code)]
pub fn v1_routes_url_analytics() -> Router<Arc<AppState>> {
    Router::new().route("/{short_code}", get(get_analytics_by_short_code))
}
