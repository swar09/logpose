use axum::{Router, routing::get};
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use dotenvy::dotenv;
use std::env;
mod handlers;
mod models;
mod repository;
mod routes;
mod schema;
mod utils;

#[derive(Clone)]

pub struct AppState {
    pool: Pool<ConnectionManager<PgConnection>>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let addr = env::var("SERVER_URL").expect("SERVER_URL must be set");
    let pool = get_connection_pool();
    let state = AppState { pool };
    let app = Router::new()
        .route(
            "/api/health",
            get(|| async { println!("Server is live !") }),
        )
        .nest("/users", crate::routes::users::v1_routes_users())
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
