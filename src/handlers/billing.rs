use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use razorpay::{models::CreateOrderRequest, webhooks::verify_payment_signature};
use uuid::Uuid;

use crate::{
    AppState,
    billing::callback::{
        CreateOrderPayload, CreateOrderResponse, UserSubscriptionResponse, VerifyPaymentPayload, VerifyPaymentResponse,
    },
    error::AppError,
    models::{
        auth::AuthUser,
        billing::{BillingInterval, NewPayment, NewUserSubscription, PaymentStatus, SubscriptionStatus},
    },
    repository::billing::{
        cancel_user_subscription, create_payment, create_user_subscription, get_active_plans,
        get_active_subscription_by_user_id, get_payment_by_order_id, get_plan_by_code, get_plan_by_id,
        update_payment_failed, update_payment_success,
    },
};

pub async fn list_plans(State(state): State<Arc<AppState>>) -> Result<Response, AppError> {
    let mut conn = state.pool.get()?;
    let plans = get_active_plans(&mut conn)?;
    Ok((StatusCode::OK, Json(plans)).into_response())
}

pub async fn create_checkout_order(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(payload): Json<CreateOrderPayload>,
) -> Result<Response, AppError> {
    let mut conn = state.pool.get()?;
    let plan = get_plan_by_id(payload.plan_id, &mut conn)?;

    let receipt_str = format!("rcpt_{}", Uuid::new_v4().simple());
    let req = CreateOrderRequest {
        amount: plan.amount as u64,
        currency: plan.currency.clone(),
        receipt: Some(receipt_str),
        partial_payment: Some(false),
        first_payment_min_amount: None,
        transfers: None,
        notes: None,
    };

    let order_id = match state.billing.create_order(req, None).await {
        Ok(order) => order.id,
        Err(e) => {
            if state.razorpay_key_id.starts_with("rzp_test") {
                tracing::warn!(
                    "Razorpay live API returned error with test key ({e}), generating development test order"
                );
                format!("order_{}", Uuid::new_v4().simple())
            } else {
                return Err(AppError::from(e));
            }
        },
    };

    let new_payment = NewPayment {
        user_id: auth_user.user_id,
        plan_id: plan.id,
        subscription_id: None,
        amount: plan.amount,
        currency: &plan.currency,
        status: PaymentStatus::Created,
        razorpay_order_id: &order_id,
        razorpay_payment_id: None,
        razorpay_signature: None,
    };

    create_payment(new_payment, &mut conn)?;

    let response = CreateOrderResponse {
        order_id,
        amount: plan.amount,
        currency: plan.currency,
        razorpay_key_id: state.razorpay_key_id.clone(),
        plan_id: plan.id,
        plan_name: plan.name,
    };

    Ok((StatusCode::CREATED, Json(response)).into_response())
}

pub async fn verify_checkout_payment(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(payload): Json<VerifyPaymentPayload>,
) -> Result<Response, AppError> {
    let mut conn = state.pool.get()?;

    let existing_payment = get_payment_by_order_id(&payload.razorpay_order_id, &mut conn)?;
    if existing_payment.user_id != auth_user.user_id {
        return Err(AppError::Forbidden("Unauthorized payment verification".into()));
    }

    let is_valid = verify_payment_signature(
        &payload.razorpay_order_id,
        &payload.razorpay_payment_id,
        &payload.razorpay_signature,
        &state.razorpay_key_secret,
    );

    if let Err(e) = is_valid {
        update_payment_failed(
            &payload.razorpay_order_id,
            Some("SIGNATURE_VERIFICATION_FAILED"),
            Some(&e.to_string()),
            &mut conn,
        )?;
        return Err(AppError::BadRequest("Invalid payment signature".into()));
    }

    let updated_payment = update_payment_success(
        &payload.razorpay_order_id,
        &payload.razorpay_payment_id,
        Some(&payload.razorpay_signature),
        &mut conn,
    )?;

    let plan = get_plan_by_id(updated_payment.plan_id, &mut conn)?;
    let start_now = Utc::now();
    let end_time = match plan.interval {
        BillingInterval::Monthly => Some(start_now + chrono::Duration::days(30)),
        BillingInterval::Yearly => Some(start_now + chrono::Duration::days(365)),
        BillingInterval::Lifetime => None,
        BillingInterval::OneTime => Some(start_now + chrono::Duration::days(30)),
    };

    let new_sub = NewUserSubscription {
        user_id: auth_user.user_id,
        plan_id: plan.id,
        status: SubscriptionStatus::Active,
        razorpay_subscription_id: None,
        razorpay_customer_id: None,
        current_period_start: Some(start_now),
        current_period_end: end_time,
        cancel_at_period_end: false,
    };

    let subscription = create_user_subscription(new_sub, &mut conn)?;

    let response = VerifyPaymentResponse {
        success: true,
        message: "Payment verified and plan activated successfully".into(),
        payment: updated_payment,
        subscription,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

pub async fn get_current_subscription(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<Response, AppError> {
    let mut conn = state.pool.get()?;
    let active_sub = get_active_subscription_by_user_id(auth_user.user_id, &mut conn)?;

    let (plan, is_active) = match &active_sub {
        Some(sub) => {
            let p = get_plan_by_id(sub.plan_id, &mut conn)?;
            (p, true)
        },
        None => {
            let p = get_plan_by_code("plan_free", &mut conn).or_else(|_| get_plan_by_id(1, &mut conn))?;
            (p, false)
        },
    };

    let response = UserSubscriptionResponse {
        subscription: active_sub,
        plan,
        is_active,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

pub async fn cancel_current_subscription(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<Response, AppError> {
    let mut conn = state.pool.get()?;
    let active_sub = match get_active_subscription_by_user_id(auth_user.user_id, &mut conn)? {
        Some(sub) => sub,
        None => {
            return Err(AppError::BadRequest("No active subscription found to cancel".into()));
        },
    };

    if let Some(ref rzp_sub_id) = active_sub.razorpay_subscription_id {
        let _ = state.billing.stop_subscription(rzp_sub_id, true, None).await;
    }

    cancel_user_subscription(active_sub.id, true, &mut conn)?;

    let response = serde_json::json!({
        "success": true,
        "message": "Subscription cancelled. Access remains active until the end of the current billing cycle."
    });

    Ok((StatusCode::OK, Json(response)).into_response())
}
