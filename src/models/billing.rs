use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(DbEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::BillingInterval"]
pub enum BillingInterval {
    Monthly,
    Yearly,
    Lifetime,
    OneTime,
}

#[derive(DbEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::SubscriptionStatus"]
pub enum SubscriptionStatus {
    Created,
    Active,
    PastDue,
    Canceled,
    Expired,
}

#[derive(DbEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::PaymentStatus"]
pub enum PaymentStatus {
    Created,
    Authorized,
    Captured,
    Failed,
    Refunded,
}

#[derive(DbEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::WebhookProcessingStatus"]
pub enum WebhookProcessingStatus {
    Pending,
    Processed,
    Failed,
    Ignored,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::plans)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Plan {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub amount: i32,
    pub currency: String,
    pub interval: BillingInterval,
    pub max_urls_limit: i32,
    pub custom_alias_allowed: bool,
    pub analytics_retention_days: i32,
    pub features: JsonValue,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::plans)]
pub struct NewPlan<'a> {
    pub name: &'a str,
    pub code: &'a str,
    pub description: Option<&'a str>,
    pub amount: i32,
    pub currency: &'a str,
    pub interval: BillingInterval,
    pub max_urls_limit: i32,
    pub custom_alias_allowed: bool,
    pub analytics_retention_days: i32,
    pub features: JsonValue,
    pub is_active: bool,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::user_subscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: i32,
    pub status: SubscriptionStatus,
    pub razorpay_subscription_id: Option<String>,
    pub razorpay_customer_id: Option<String>,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub canceled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::user_subscriptions)]
pub struct NewUserSubscription<'a> {
    pub user_id: Uuid,
    pub plan_id: i32,
    pub status: SubscriptionStatus,
    pub razorpay_subscription_id: Option<&'a str>,
    pub razorpay_customer_id: Option<&'a str>,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::payments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Payment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: i32,
    pub subscription_id: Option<Uuid>,
    pub amount: i32,
    pub currency: String,
    pub status: PaymentStatus,
    pub razorpay_order_id: String,
    pub razorpay_payment_id: Option<String>,
    pub razorpay_signature: Option<String>,
    pub error_code: Option<String>,
    pub error_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::payments)]
pub struct NewPayment<'a> {
    pub user_id: Uuid,
    pub plan_id: i32,
    pub subscription_id: Option<Uuid>,
    pub amount: i32,
    pub currency: &'a str,
    pub status: PaymentStatus,
    pub razorpay_order_id: &'a str,
    pub razorpay_payment_id: Option<&'a str>,
    pub razorpay_signature: Option<&'a str>,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::webhook_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WebhookEvent {
    pub id: Uuid,
    pub event_id: String,
    pub event_type: String,
    pub status: WebhookProcessingStatus,
    pub payload: JsonValue,
    pub error_log: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::webhook_events)]
pub struct NewWebhookEvent<'a> {
    pub event_id: &'a str,
    pub event_type: &'a str,
    pub status: WebhookProcessingStatus,
    pub payload: &'a JsonValue,
}
