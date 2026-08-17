use async_trait::async_trait;
use axum::http::HeaderMap;
use razorpay::{
    Creatable, Fetchable, RazorpayClient,
    models::{
        CapturePaymentRequest, CreateOrderRequest, CreatePlanRequest, CreateSubscriptionRequest,
        Order, Payment, Plan, Subscription,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum BillingError {
    #[error("payment not found: {0}")]
    NotFound(String),
    #[error("razorpay error: {0}")]
    Provider(#[from] razorpay::RazorpayError),
}

#[async_trait]
pub trait PaymentsGateway: Send + Sync {
    // verify payment signature
    async fn verify_payment_signature(&self) -> Result<bool, BillingError>;

    // verify subscription signature
    async fn verify_subscriptionsignature(&self) -> Result<bool, BillingError>;

    // get payment
    async fn get_payment(
        &self,
        payment_id: &str,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Payment, BillingError>;

    // get order
    async fn get_order(
        &self,
        order_id: &str,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Order, BillingError>;

    // create order
    async fn create_order(
        &self,
        request: CreateOrderRequest,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Order, BillingError>;

    // verify payment
    async fn verify_payment(
        &self,
        payment_id: &str,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Payment, BillingError>;

    // capture payment
    async fn capture_payment(
        &self,
        payment_id: &str,
        request: CapturePaymentRequest,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Payment, BillingError>;

    // create plan (daily, weekly, monthly, annual) default is monthly
    async fn create_plan(
        &self,
        request: CreatePlanRequest,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Plan, BillingError>;

    // start subscription
    async fn start_subscription(
        &self,
        request: CreateSubscriptionRequest,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Subscription, BillingError>;

    // stop subscription
    async fn stop_subscription(
        &self,
        subscription_id: &str,
        cancel_at_cycle_end: bool,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Subscription, BillingError>;

    // pause subscription
    async fn pause_subscription(
        &self,
        subscription_id: &str,
        pause_at: Option<&str>,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Subscription, BillingError>;

    // resume subscription
    async fn resume_subscription(
        &self,
        subscription_id: &str,
        resume_at: Option<&str>,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Subscription, BillingError>;

    // TODO : Add more fuctions
}

#[async_trait]
impl PaymentsGateway for RazorpayClient {
    // verify payment signature
    async fn verify_payment_signature(&self) -> Result<bool, BillingError> {
        todo!()
    }

    // verify subscription signature
    async fn verify_subscriptionsignature(&self) -> Result<bool, BillingError> {
        todo!()
    }

    // get payment
    async fn get_payment(
        &self,
        payment_id: &str,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Payment, BillingError> {
        self.payments()
            .fetch(payment_id, extra_headers)
            .await
            .map_err(BillingError::from)
    }

    // get order
    async fn get_order(
        &self,
        order_id: &str,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Order, BillingError> {
        self.orders()
            .fetch(order_id, extra_headers)
            .await
            .map_err(BillingError::from)
    }

    // create order
    async fn create_order(
        &self,
        request: CreateOrderRequest,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Order, BillingError> {
        self.orders()
            .create(request, extra_headers)
            .await
            .map_err(BillingError::from)
    }

    // verify payment
    async fn verify_payment(
        &self,
        payment_id: &str,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Payment, BillingError> {
        self.payments()
            .fetch(payment_id, extra_headers)
            .await
            .map_err(BillingError::from)
    }

    // capture payment
    async fn capture_payment(
        &self,
        payment_id: &str,
        request: CapturePaymentRequest,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Payment, BillingError> {
        self.payments()
            .capture(payment_id, request, extra_headers)
            .await
            .map_err(BillingError::from)
    }

    // create monthly plan
    async fn create_plan(
        &self,
        request: CreatePlanRequest,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Plan, BillingError> {
        self.plans()
            .create(request, extra_headers)
            .await
            .map_err(BillingError::from)
    }

    // start subscription
    async fn start_subscription(
        &self,
        request: CreateSubscriptionRequest,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Subscription, BillingError> {
        self.subscriptions()
            .create(request, extra_headers)
            .await
            .map_err(BillingError::from)
    }

    // stop subscription
    async fn stop_subscription(
        &self,
        subscription_id: &str,
        cancel_at_cycle_end: bool,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Subscription, BillingError> {
        self.subscriptions()
            .cancel(subscription_id, cancel_at_cycle_end, extra_headers)
            .await
            .map_err(BillingError::from)
    }

    // pause subscription
    async fn pause_subscription(
        &self,
        subscription_id: &str,
        pause_at: Option<&str>,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Subscription, BillingError> {
        self.subscriptions()
            .pause(subscription_id, pause_at, extra_headers)
            .await
            .map_err(BillingError::from)
    }

    // resume subscription
    async fn resume_subscription(
        &self,
        subscription_id: &str,
        resume_at: Option<&str>,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Subscription, BillingError> {
        self.subscriptions()
            .resume(subscription_id, resume_at, extra_headers)
            .await
            .map_err(BillingError::from)
    }

    // TODO : impl them all
}
