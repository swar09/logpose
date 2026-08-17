use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    AppState,
    handlers::{
        auth::{google_callback, google_login, login, logout},
        user::signup,
    },
};

pub fn v1_routes_auth() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/signup", post(signup))
        .route("/google", get(google_login))
        .route("/google/callback", get(google_callback))
}
