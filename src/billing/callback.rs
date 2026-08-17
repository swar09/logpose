use serde::{Deserialize, Serialize};

use crate::models::billing::{Payment, Plan, UserSubscription};

#[derive(Deserialize)]
pub struct CreateOrderPayload {
    pub plan_id: i32,
}

#[derive(Serialize)]
pub struct CreateOrderResponse {
    pub order_id: String,
    pub amount: i32,
    pub currency: String,
    pub razorpay_key_id: String,
    pub plan_id: i32,
    pub plan_name: String,
}

#[derive(Deserialize)]
pub struct VerifyPaymentPayload {
    pub razorpay_order_id: String,
    pub razorpay_payment_id: String,
    pub razorpay_signature: String,
}

#[derive(Serialize)]
pub struct VerifyPaymentResponse {
    pub success: bool,
    pub message: String,
    pub payment: Payment,
    pub subscription: UserSubscription,
}

#[derive(Serialize)]
pub struct UserSubscriptionResponse {
    pub subscription: Option<UserSubscription>,
    pub plan: Plan,
    pub is_active: bool,
}
