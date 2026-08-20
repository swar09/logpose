use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use crate::{
    AppState,
    handlers::user::{
        delete_user_by_id, get_subscription_by_id, get_urls, get_user_by_id, signup, update_user_info_by_id,
        update_user_password,
    },
};

pub fn v1_routes_users() -> Router<Arc<AppState>> {
    let users_router: Router<Arc<AppState>> = Router::new()
        .route("/", patch(update_user_info_by_id))
        .route("/urls", get(get_urls))
        .route("/subscription", get(get_subscription_by_id))
        .route("/", get(get_user_by_id))
        .route("/", delete(delete_user_by_id))
        .route("/updatepassword", patch(update_user_password));

    Router::new()
        .route("/", post(signup))
        .nest("/{path_user_id}", users_router)
}
