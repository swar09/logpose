use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    models::billing::{
        NewPayment, NewUserSubscription, Payment, PaymentStatus, Plan, SubscriptionStatus, UserSubscription,
    },
    schema::{payments, plans, user_subscriptions},
};

pub fn get_active_plans(conn: &mut PgConnection) -> Result<Vec<Plan>, diesel::result::Error> {
    plans::table
        .filter(plans::is_active.eq(true))
        .order(plans::amount.asc())
        .select(Plan::as_select())
        .load(conn)
}

pub fn get_plan_by_id(plan_id: i32, conn: &mut PgConnection) -> Result<Plan, diesel::result::Error> {
    plans::table.find(plan_id).select(Plan::as_select()).first(conn)
}

pub fn get_plan_by_code(code_str: &str, conn: &mut PgConnection) -> Result<Plan, diesel::result::Error> {
    plans::table
        .filter(plans::code.eq(code_str))
        .select(Plan::as_select())
        .first(conn)
}

pub fn create_payment(new_payment: NewPayment, conn: &mut PgConnection) -> Result<Payment, diesel::result::Error> {
    diesel::insert_into(payments::table)
        .values(&new_payment)
        .returning(Payment::as_returning())
        .get_result(conn)
}

pub fn get_payment_by_order_id(order_id_str: &str, conn: &mut PgConnection) -> Result<Payment, diesel::result::Error> {
    payments::table
        .filter(payments::razorpay_order_id.eq(order_id_str))
        .select(Payment::as_select())
        .first(conn)
}

pub fn get_payment_by_id(payment_id: Uuid, conn: &mut PgConnection) -> Result<Payment, diesel::result::Error> {
    payments::table
        .find(payment_id)
        .select(Payment::as_select())
        .first(conn)
}

pub fn get_payments_by_user_id(
    user_id_val: Uuid,
    conn: &mut PgConnection,
) -> Result<Vec<Payment>, diesel::result::Error> {
    payments::table
        .filter(payments::user_id.eq(user_id_val))
        .order(payments::created_at.desc())
        .select(Payment::as_select())
        .load(conn)
}

pub fn update_payment_success(
    order_id_str: &str,
    payment_id_str: &str,
    signature_str: Option<&str>,
    conn: &mut PgConnection,
) -> Result<Payment, diesel::result::Error> {
    diesel::update(payments::table.filter(payments::razorpay_order_id.eq(order_id_str)))
        .set((
            payments::status.eq(PaymentStatus::Captured),
            payments::razorpay_payment_id.eq(Some(payment_id_str)),
            payments::razorpay_signature.eq(signature_str),
        ))
        .returning(Payment::as_returning())
        .get_result(conn)
}

pub fn update_payment_failed(
    order_id_str: &str,
    err_code: Option<&str>,
    err_desc: Option<&str>,
    conn: &mut PgConnection,
) -> Result<Payment, diesel::result::Error> {
    diesel::update(payments::table.filter(payments::razorpay_order_id.eq(order_id_str)))
        .set((
            payments::status.eq(PaymentStatus::Failed),
            payments::error_code.eq(err_code),
            payments::error_description.eq(err_desc),
        ))
        .returning(Payment::as_returning())
        .get_result(conn)
}

pub fn create_user_subscription(
    new_sub: NewUserSubscription,
    conn: &mut PgConnection,
) -> Result<UserSubscription, diesel::result::Error> {
    diesel::insert_into(user_subscriptions::table)
        .values(&new_sub)
        .returning(UserSubscription::as_returning())
        .get_result(conn)
}

pub fn get_active_subscription_by_user_id(
    user_id_val: Uuid,
    conn: &mut PgConnection,
) -> Result<Option<UserSubscription>, diesel::result::Error> {
    user_subscriptions::table
        .filter(user_subscriptions::user_id.eq(user_id_val))
        .filter(user_subscriptions::status.eq(SubscriptionStatus::Active))
        .order(user_subscriptions::created_at.desc())
        .select(UserSubscription::as_select())
        .first(conn)
        .optional()
}

pub fn get_subscription_by_razorpay_id(
    rzp_sub_id: &str,
    conn: &mut PgConnection,
) -> Result<UserSubscription, diesel::result::Error> {
    user_subscriptions::table
        .filter(user_subscriptions::razorpay_subscription_id.eq(rzp_sub_id))
        .select(UserSubscription::as_select())
        .first(conn)
}

pub fn update_subscription_period(
    sub_id: Uuid,
    status_val: SubscriptionStatus,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    conn: &mut PgConnection,
) -> Result<UserSubscription, diesel::result::Error> {
    diesel::update(user_subscriptions::table.find(sub_id))
        .set((
            user_subscriptions::status.eq(status_val),
            user_subscriptions::current_period_start.eq(start),
            user_subscriptions::current_period_end.eq(end),
        ))
        .returning(UserSubscription::as_returning())
        .get_result(conn)
}

pub fn cancel_user_subscription(
    sub_id: Uuid,
    at_period_end: bool,
    conn: &mut PgConnection,
) -> Result<UserSubscription, diesel::result::Error> {
    let now = Utc::now();
    let status_val = if at_period_end {
        SubscriptionStatus::Active
    } else {
        SubscriptionStatus::Canceled
    };

    diesel::update(user_subscriptions::table.find(sub_id))
        .set((
            user_subscriptions::status.eq(status_val),
            user_subscriptions::cancel_at_period_end.eq(at_period_end),
            user_subscriptions::canceled_at.eq(Some(now)),
        ))
        .returning(UserSubscription::as_returning())
        .get_result(conn)
}
