mod billing;
mod error;
mod handlers;
mod models;
mod repository;
mod routes;
mod schema;
mod service;
mod utils;

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use aes::Aes256;
use axum::{Router, routing::get};
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use dotenvy::dotenv;
use fpe::ff1::FF1;
use razorpay::RazorpayClient;
use tower_http::cors::AllowOrigin;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::billing::billing::PaymentsGateway;
use crate::handlers::notfound::not_found;
use crate::routes::auth::v1_routes_auth;
use crate::routes::billing::v1_routes_billing;
use crate::routes::url::redirect_url_routes;
use crate::routes::url::v1_routes_public;
use crate::routes::url::v1_routes_urls;
use crate::routes::user::v1_routes_users;
use crate::routes::webhook::v1_routes_webhook;
use crate::service::rate_limiting::RateLimiterLayer;
use crate::service::rate_limiting::TokenBucket;
use crate::service::url_service::UrlService;
use crate::utils::redis::RedisStore;

const RADIX: u32 = 3812;

pub struct AppState {
    pub ff: Arc<FF1<Aes256>>,
    pub pool: Pool<ConnectionManager<PgConnection>>,
    pub jwt_secret: String,
    pub redis_store: RedisStore,
    pub url_service: UrlService,
    pub billing: Arc<dyn PaymentsGateway>,
    pub webhook_secret: String,
    pub razorpay_key_id: String,
    pub razorpay_key_secret: String,
}

pub fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(url);

    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .without_time()
        .init();

    println!("   \x1b[32m\x1b[1mInitializing\x1b[0m Logpose services...");

    let binding = env::var("ID_AES_KEY").expect("AES KEY NOT PROVIDED");
    let key_bytes =
        hex::decode(&binding).expect("ID_AES_KEY must be a valid 64-character hex string");
    let addr = env::var("SERVER_URL").expect("SERVER_URL must be set");

    let pool = get_connection_pool();
    println!("      \x1b[32m\x1b[1mConnected\x1b[0m to PostgreSQL database");

    let ff = Arc::new(FF1::<Aes256>::new(&key_bytes, RADIX).expect("Failed to initialize FF1"));
    let jwt_secret = env::var("JWT_ENCODING_KEY").expect("JWT_ENCODING_KEY must be set");

    let redis_addr = env::var("REDIS_ADDR").expect("REDIS_ADDR must be set");
    let redis_store = RedisStore::new(&redis_addr)
        .await
        .expect("Failed to connect to Redis");
    println!("      \x1b[32m\x1b[1mConnected\x1b[0m to Redis cache");

    let url_service = UrlService::new(redis_store.clone(), pool.clone(), ff.clone());
    println!("     \x1b[32m\x1b[1mRegistered\x1b[0m URL Shortener services");

    let razorpay_key_id = env::var("RAZORPAY_API_KEY").expect("razorpay_api_key not found");
    let razorpay_key_secret = env::var("RAZORPAY_API_SECRET").expect("razorpay_secret not found");
    let client = RazorpayClient::new(razorpay_key_id.clone(), razorpay_key_secret.clone())
        .expect("Razorpay API_KEY & SECRET must be set properly");
    let billing = Arc::new(client);
    println!("      \x1b[32m\x1b[1mCreated\x1b[0m Billing Client");

    let webhook_secret =
        env::var("RAZORPAY_WEBHOOK_SECRET").expect("RAZORPAY_WEBHOOK_SECRET must be set");

    let cleanup_pool = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Ok(mut conn) = cleanup_pool.get() {
                let _ = crate::repository::url::cleanup_expired_guest_urls(&mut conn);
            }
        }
    });

    let state = Arc::new(AppState {
        pool,
        ff,
        jwt_secret,
        redis_store,
        url_service,
        billing,
        webhook_secret,
        razorpay_key_id,
        razorpay_key_secret,
    });

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::COOKIE,
        ])
        .allow_credentials(true);

    let bucket = TokenBucket::new(100, 10).expect("rate limiting must be set properly");
    let rate_limiting_layer = RateLimiterLayer::new(bucket);

    let app = Router::new()
        .route(
            "/api/health",
            get(|| async {
                info!("Health check");
                "OK"
            }),
        )
        .nest("/api/v1/users", v1_routes_users())
        .nest("/api/v1/urls", v1_routes_urls())
        .nest("/api/v1/public", v1_routes_public())
        .nest("/api/v1/auth", v1_routes_auth())
        .nest("/api/v1/billing", v1_routes_billing())
        .nest("/api/v1/webhooks", v1_routes_webhook())
        .merge(redirect_url_routes())
        .fallback(not_found)
        .with_state(state)
        .layer(cors)
        .layer(rate_limiting_layer);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind server");

    println!(
        "       \x1b[32m\x1b[1mListening\x1b[0m HTTP server on http://{}",
        addr
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Server failed");
}
