use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    AppState,
    handlers::user::{get_subscription_by_id, get_urls, update_user_info_by_id},
};

pub fn v1_routes_users() -> Router<Arc<AppState>> {
    let users_router: Router<Arc<AppState>> = Router::new()
        .route("/", post(update_user_info_by_id))
        .route("/urls", get(get_urls))
        .route("/subscription", get(get_subscription_by_id));

    Router::new().nest("/{path_user_id}", users_router)
    // todo!()
}
