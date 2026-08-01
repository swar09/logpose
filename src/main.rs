use aes::Aes256;
use axum::{Router, routing::get};
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use dotenvy::dotenv;
use fpe::ff1::FF1;
use std::env;
use std::sync::Arc;
mod handlers;
mod models;
mod repository;
mod routes;
mod schema;
mod utils;

// #[derive(Clone)]

pub struct AppState {
    pool: Pool<ConnectionManager<PgConnection>>,
    ff: FF1<Aes256>,
}
const RADIX: u32 = 3812;
#[tokio::main]
async fn main() {
    dotenv().ok();

    let binding = env::var("ID_AES_KEY").expect("AES KEY NOT PROVIDED");
    let key = binding.as_bytes();
    let addr = env::var("SERVER_URL").expect("SERVER_URL must be set");
    let pool = get_connection_pool();
    let ff = FF1::<Aes256>::new(key, RADIX).unwrap();

    let state = Arc::new(AppState { pool, ff });
    let app = Router::new()
        .route(
            "/api/health",
            get(|| async { println!("Server is live !") }),
        )
        .nest("/api/v1/users", crate::routes::users::v1_routes_users())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

pub fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(url);

    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}
