use crate::AppState;
use axum::Json;
use axum::http::{Error, StatusCode};
use axum::response::IntoResponse;
use axum::{body::Bytes, extract::State, http::HeaderMap, response::Response};
use razorpay::webhooks::verify_webhook_signature;

pub async fn razorpay_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let _rzp_event_id = match headers
        .get("x-razorpay-event-id")
        .and_then(|v| v.to_str().ok())
    {
        Some(id) => id,
        None => {
            tracing::warn!("webhook request missing x-razorpay-event-id ");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };
    let signature = match headers
        .get("X-Razorpay-Signature")
        .and_then(|v| v.to_str().ok())
    {
        Some(sig) => sig,
        None => {
            tracing::warn!("webhook request missing X-Razorpay-Signature header");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let payload = match std::str::from_utf8(&body) {
        Ok(p) => p.to_owned(),
        Err(_) => {
            tracing::warn!("webhook request body parsing failed !");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let verified = verify_webhook_signature(&payload, signature, &state.webhook_secret);

    if verified.is_err() {
        tracing::warn!("Error in verification of webhook signature ");
        return StatusCode::BAD_REQUEST.into_response();
    }

    let event = Json::from(payload).clone();

    let state = state;

    tokio::spawn(async move {
        if process_webhook_event(state, event).await.is_err() {
            tracing::error!("error in processing webhook event");
        }
    });

    StatusCode::OK.into_response()
}

async fn process_webhook_event(_state: AppState, _event: Json<String>) -> Result<bool, Error> {
    // verification done
    // parse
    // db queries
    // return some result
    // trigger some service or background job
    // diesel repo layer must be ready for this first
    todo!()
}
