use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use razorpay::webhooks::verify_webhook_signature;
use serde_json::Value as JsonValue;

use crate::{
    AppState,
    models::billing::{
        BillingInterval, NewUserSubscription, NewWebhookEvent, WebhookProcessingStatus,
    },
    repository::{
        billing::{
            create_user_subscription, get_payment_by_order_id, get_plan_by_id,
            update_payment_failed, update_payment_success,
        },
        webhook::{mark_webhook_processed, record_webhook_event},
    },
};

pub async fn razorpay_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let rzp_event_id = match headers
        .get("x-razorpay-event-id")
        .and_then(|v| v.to_str().ok())
    {
        Some(id) => id.to_string(),
        None => {
            tracing::warn!("webhook request missing x-razorpay-event-id");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let signature = match headers
        .get("X-Razorpay-Signature")
        .and_then(|v| v.to_str().ok())
    {
        Some(sig) => sig.to_string(),
        None => {
            tracing::warn!("webhook request missing X-Razorpay-Signature header");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let payload_str = match std::str::from_utf8(&body) {
        Ok(p) => p.to_string(),
        Err(_) => {
            tracing::warn!("webhook request body parsing failed");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    if verify_webhook_signature(&payload_str, &signature, &state.webhook_secret).is_err() {
        tracing::warn!("Error in verification of webhook signature");
        return StatusCode::BAD_REQUEST.into_response();
    }

    let payload_json: JsonValue = match serde_json::from_str(&payload_str) {
        Ok(j) => j,
        Err(_) => {
            tracing::warn!("Invalid JSON payload in webhook");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let state_clone = state.clone();
    let event_id = rzp_event_id.clone();

    tokio::spawn(async move {
        if let Err(e) = process_webhook_event(state_clone, event_id, payload_json).await {
            tracing::error!("error in processing webhook event: {e}");
        }
    });

    StatusCode::OK.into_response()
}

async fn process_webhook_event(
    state: Arc<AppState>,
    event_id: String,
    payload: JsonValue,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.pool.get()?;
    let event_type = payload
        .get("event")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let new_event = NewWebhookEvent {
        event_id: &event_id,
        event_type,
        status: WebhookProcessingStatus::Pending,
        payload: &payload,
    };

    let inserted_id = record_webhook_event(new_event, &mut conn)?;
    if inserted_id.is_none() {
        return Ok(());
    }

    match event_type {
        "payment.captured" | "order.paid" => {
            let payment_obj = payload
                .get("payload")
                .and_then(|p| p.get("payment"))
                .and_then(|p| p.get("entity"));

            if let Some(payment_entity) = payment_obj {
                let order_id = payment_entity
                    .get("order_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let payment_id = payment_entity
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if !order_id.is_empty()
                    && !payment_id.is_empty()
                    && let Ok(payment) = get_payment_by_order_id(order_id, &mut conn)
                {
                    let _ = update_payment_success(order_id, payment_id, None, &mut conn);

                    if let Ok(plan) = get_plan_by_id(payment.plan_id, &mut conn) {
                        let start_now = Utc::now();
                        let end_time = match plan.interval {
                            BillingInterval::Monthly => {
                                Some(start_now + chrono::Duration::days(30))
                            }
                            BillingInterval::Yearly => {
                                Some(start_now + chrono::Duration::days(365))
                            }
                            BillingInterval::Lifetime => None,
                            BillingInterval::OneTime => {
                                Some(start_now + chrono::Duration::days(30))
                            }
                        };

                        let new_sub = NewUserSubscription {
                            user_id: payment.user_id,
                            plan_id: plan.id,
                            status: crate::models::billing::SubscriptionStatus::Active,
                            razorpay_subscription_id: None,
                            razorpay_customer_id: None,
                            current_period_start: Some(start_now),
                            current_period_end: end_time,
                            cancel_at_period_end: false,
                        };
                        let _ = create_user_subscription(new_sub, &mut conn);
                    }
                }
            }
        }
        "payment.failed" => {
            let payment_obj = payload
                .get("payload")
                .and_then(|p| p.get("payment"))
                .and_then(|p| p.get("entity"));

            if let Some(payment_entity) = payment_obj {
                let order_id = payment_entity
                    .get("order_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let error_code = payment_entity.get("error_code").and_then(|v| v.as_str());
                let error_desc = payment_entity
                    .get("error_description")
                    .and_then(|v| v.as_str());

                if !order_id.is_empty() {
                    let _ = update_payment_failed(order_id, error_code, error_desc, &mut conn);
                }
            }
        }
        _ => {}
    }

    let _ = mark_webhook_processed(
        &event_id,
        WebhookProcessingStatus::Processed,
        None,
        &mut conn,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmac::{Hmac, KeyInit, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    fn generate_test_signature(body: &str, secret: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
        mac.update(body.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    #[test]
    fn test_valid_webhook_signature_passes() {
        let payload = r#"{"event":"payment.captured","payload":{"payment":{"entity":{"id":"pay_123","order_id":"order_456"}}}}"#;
        let secret = "test_webhook_secret_key";
        let signature = generate_test_signature(payload, secret);

        let result = verify_webhook_signature(payload, &signature, secret);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tampered_payload_fails_signature() {
        let payload = r#"{"event":"payment.captured","amount":500}"#;
        let secret = "test_webhook_secret_key";
        let signature = generate_test_signature(payload, secret);

        let tampered_payload = r#"{"event":"payment.captured","amount":50000}"#;
        let result = verify_webhook_signature(tampered_payload, &signature, secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_secret_fails_signature() {
        let payload = r#"{"event":"payment.captured","amount":500}"#;
        let secret = "test_webhook_secret_key";
        let wrong_secret = "attacker_secret_key";
        let signature = generate_test_signature(payload, wrong_secret);

        let result = verify_webhook_signature(payload, &signature, secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_payment_captured_payload_structure() {
        let payload_str = r#"{
            "entity": "event",
            "account_id": "acc_test",
            "event": "payment.captured",
            "contains": ["payment"],
            "payload": {
                "payment": {
                    "entity": {
                        "id": "pay_987654",
                        "entity": "payment",
                        "amount": 500,
                        "currency": "USD",
                        "status": "captured",
                        "order_id": "order_123456"
                    }
                }
            },
            "created_at": 1786960000
        }"#;

        let json: JsonValue = serde_json::from_str(payload_str).expect("Valid JSON expected");
        assert_eq!(
            json.get("event").and_then(|v| v.as_str()),
            Some("payment.captured")
        );

        let payment_entity = json
            .get("payload")
            .and_then(|p| p.get("payment"))
            .and_then(|p| p.get("entity"))
            .expect("payment entity expected");

        assert_eq!(
            payment_entity.get("id").and_then(|v| v.as_str()),
            Some("pay_987654")
        );
        assert_eq!(
            payment_entity.get("order_id").and_then(|v| v.as_str()),
            Some("order_123456")
        );
        assert_eq!(
            payment_entity.get("status").and_then(|v| v.as_str()),
            Some("captured")
        );
    }

    #[test]
    fn test_parse_payment_failed_payload_structure() {
        let payload_str = r#"{
            "entity": "event",
            "event": "payment.failed",
            "payload": {
                "payment": {
                    "entity": {
                        "id": "pay_failed_123",
                        "order_id": "order_fail_999",
                        "error_code": "BAD_REQUEST_ERROR",
                        "error_description": "Card was declined"
                    }
                }
            }
        }"#;

        let json: JsonValue = serde_json::from_str(payload_str).expect("Valid JSON expected");
        assert_eq!(
            json.get("event").and_then(|v| v.as_str()),
            Some("payment.failed")
        );

        let payment_entity = json
            .get("payload")
            .and_then(|p| p.get("payment"))
            .and_then(|p| p.get("entity"))
            .expect("payment entity expected");

        assert_eq!(
            payment_entity.get("error_code").and_then(|v| v.as_str()),
            Some("BAD_REQUEST_ERROR")
        );
        assert_eq!(
            payment_entity
                .get("error_description")
                .and_then(|v| v.as_str()),
            Some("Card was declined")
        );
    }
}
