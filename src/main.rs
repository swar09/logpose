mod handlers;
mod models;
mod repository;
mod routes;
mod schema;
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

use crate::routes::auth::v1_routes_auth;
use crate::routes::url::redirect_url_routes;
use crate::routes::url::v1_routes_urls;

const RADIX: u32 = 3812;
pub struct AppState {
    ff: FF1<Aes256>,

    pool: Pool<ConnectionManager<PgConnection>>,

    jwt_secret: String,
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

    let binding = env::var("ID_AES_KEY").expect("AES KEY NOT PROVIDED");
    let key_bytes = hex::decode(&binding).expect("ID_AES_KEY must be a valid 64-character hex string");
    let addr = env::var("SERVER_URL").expect("SERVER_URL must be set");
    let pool = get_connection_pool();
    let ff = FF1::<Aes256>::new(&key_bytes, RADIX).unwrap();
    let jwt_secret = env::var("JWT_ENCODING_KEY").expect("JWT_ENCODING_KEY must be set");
    let state = Arc::new(AppState {
        pool,
        ff,
        jwt_secret,
    });
    let app = Router::new()
        .route(
            "/api/health",
            get(|| async { println!("Server is live !") }),
        )
        .nest("/api/v1/users", crate::routes::user::v1_routes_users())
        .nest("/api/v1/urls", v1_routes_urls())
        .nest("/api/v1/auth", v1_routes_auth())
        .merge(redirect_url_routes())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        println!("http-server started at http://{}", std::env::var("SERVER_URL").unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
