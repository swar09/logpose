// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "billing_interval"))]
    pub struct BillingInterval;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "payment_status"))]
    pub struct PaymentStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "subscription_status"))]
    pub struct SubscriptionStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "transaction_status"))]
    pub struct TransactionStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "webhook_processing_status"))]
    pub struct WebhookProcessingStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PaymentStatus;

    payments (id) {
        id -> Uuid,
        user_id -> Uuid,
        plan_id -> Int4,
        subscription_id -> Nullable<Uuid>,
        amount -> Int4,
        #[max_length = 3]
        currency -> Varchar,
        status -> PaymentStatus,
        #[max_length = 100]
        razorpay_order_id -> Varchar,
        #[max_length = 100]
        razorpay_payment_id -> Nullable<Varchar>,
        razorpay_signature -> Nullable<Text>,
        #[max_length = 100]
        error_code -> Nullable<Varchar>,
        error_description -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::BillingInterval;

    plans (id) {
        id -> Int4,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 50]
        code -> Varchar,
        description -> Nullable<Text>,
        amount -> Int4,
        #[max_length = 3]
        currency -> Varchar,
        interval -> BillingInterval,
        max_urls_limit -> Int4,
        custom_alias_allowed -> Bool,
        analytics_retention_days -> Int4,
        features -> Jsonb,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TransactionStatus;

    transactions (id) {
        id -> Uuid,
        user_id -> Uuid,
        amount -> Int4,
        #[max_length = 5]
        currency_code -> Varchar,
        status -> TransactionStatus,
        reference_id -> Nullable<Uuid>,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    url_analytics (id) {
        id -> Uuid,
        #[max_length = 4]
        short_code -> Nullable<Varchar>,
        clicked_at -> Timestamptz,
        #[max_length = 45]
        ip_address -> Varchar,
        #[max_length = 1024]
        user_agent -> Nullable<Varchar>,
        #[max_length = 100]
        browser -> Nullable<Varchar>,
        #[max_length = 100]
        device -> Nullable<Varchar>,
        #[max_length = 10]
        country_code -> Nullable<Varchar>,
        #[max_length = 2048]
        referer -> Nullable<Varchar>,
    }
}

diesel::table! {
    urls (database_id) {
        database_id -> Int4,
        #[max_length = 4]
        short_code -> Nullable<Varchar>,
        #[max_length = 2048]
        long_url -> Varchar,
        created_by -> Nullable<Uuid>,
        guest_id -> Nullable<Uuid>,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SubscriptionStatus;

    user_subscriptions (id) {
        id -> Uuid,
        user_id -> Uuid,
        plan_id -> Int4,
        status -> SubscriptionStatus,
        #[max_length = 100]
        razorpay_subscription_id -> Nullable<Varchar>,
        #[max_length = 100]
        razorpay_customer_id -> Nullable<Varchar>,
        current_period_start -> Nullable<Timestamptz>,
        current_period_end -> Nullable<Timestamptz>,
        cancel_at_period_end -> Bool,
        canceled_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 100]
        first_name -> Varchar,
        #[max_length = 100]
        last_name -> Varchar,
        #[max_length = 30]
        username -> Varchar,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 128]
        hashed_password -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::WebhookProcessingStatus;

    webhook_events (id) {
        id -> Uuid,
        #[max_length = 100]
        event_id -> Varchar,
        #[max_length = 100]
        event_type -> Varchar,
        status -> WebhookProcessingStatus,
        payload -> Jsonb,
        error_log -> Nullable<Text>,
        processed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(payments -> plans (plan_id));
diesel::joinable!(payments -> user_subscriptions (subscription_id));
diesel::joinable!(payments -> users (user_id));
diesel::joinable!(urls -> users (created_by));
diesel::joinable!(user_subscriptions -> plans (plan_id));
diesel::joinable!(user_subscriptions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    payments,
    plans,
    transactions,
    url_analytics,
    urls,
    user_subscriptions,
    users,
    webhook_events,
);
