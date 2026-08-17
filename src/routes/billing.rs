use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    AppState,
    handlers::billing::{
        cancel_current_subscription, create_checkout_order, get_current_subscription, list_plans,
        verify_checkout_payment,
    },
};

pub fn v1_routes_billing() -> Router<Arc<AppState>> {
    Router::new()
        .route("/plans", get(list_plans))
        .route("/order", post(create_checkout_order))
        .route("/verify", post(verify_checkout_payment))
        .route("/subscription", get(get_current_subscription))
        .route("/cancel", post(cancel_current_subscription))
}
