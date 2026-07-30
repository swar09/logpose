use axum::{Router, routing::get};

use crate::AppState;

pub fn v1_routes_users() -> Router<AppState> {

    let users_router: Router<AppState> = Router::new().route("/urls", get(crate::handlers::users::get_urls));

    Router::new().nest("/{path_user_id}", users_router)
    // todo!()
    
}



