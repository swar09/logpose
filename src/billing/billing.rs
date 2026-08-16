use async_trait::async_trait;
use axum::http::HeaderMap;
use razorpay::{
    Creatable, Fetchable, RazorpayClient,
    models::{CapturePaymentRequest, CreateOrderRequest, Order, Payment},
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

    // TODO : Add more fuctions
}

#[async_trait]
impl PaymentsGateway for RazorpayClient {
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

    // TODO : impl them all
}
