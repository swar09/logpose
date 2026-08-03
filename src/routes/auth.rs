use std::sync::Arc;

use axum::{Router, routing::post};

use crate::{
    AppState,
    handlers::auth::{login, logout, refresh},
};

pub fn v1_routes_auth() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh))
    // .route("/signup", post(signup))
}
