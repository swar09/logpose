use axum::{Router, routing::get};
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
mod repository;
mod models;
mod schema;

#[tokio::main]

async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let conn  = PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    let addr = env::var("SERVER_URL").expect("SERVER_URL must be set");
    let app = Router::new().route("/api/health", get(|| async {println!("Server is live !")}));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}                   
