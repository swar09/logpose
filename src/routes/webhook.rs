use std::sync::Arc;

use axum::{Router, routing::post};

use crate::{AppState, billing::webhook::razorpay_webhook};

pub fn v1_routes_webhook() -> Router<Arc<AppState>> {
    Router::new().route("/razorpay", post(razorpay_webhook))
}
