use std::sync::Arc;

use axum::{Router, routing::post};

use crate::{
    AppState,
    handlers::{
        auth::{login, logout},
        user::signup,
    },
};

pub fn v1_routes_auth() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/signup", post(signup))
}
