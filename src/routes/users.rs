use axum::{Router, routing::get};

use crate::{
    AppState,
    handlers::users::{get_subscription_by_id, get_urls},
};
use std::sync::Arc;

pub fn v1_routes_users() -> Router<Arc<AppState>> {
    let users_router: Router<Arc<AppState>> = Router::new()
        .route("/urls", get(get_urls))
        .route("/subscription", get(get_subscription_by_id));

    Router::new().nest("/{path_user_id}", users_router)
    // todo!()
}
