use axum::routing::{get, put};
use mimalloc::MiMalloc;
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, sync::Arc, time::Duration};

use crate::{
    articles::get_posts,
    common::AppState,
    search::{SearchService, get_search_results},
};

mod articles;
mod common;
mod config;
mod db;
mod search;
mod some_errors;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const DEFAULT_PORT: u16 = 8124;
const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_INDEX_NAME: &str = "articles";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let config = config::Config::from_env()
        .expect("Failed to load configuration. Check your environment variables.");

    let host = config.host.as_deref().unwrap_or(DEFAULT_HOST);
    let port = config.port.unwrap_or(DEFAULT_PORT);
    let addr = format!("{}:{}", host, port);
    let address: SocketAddr = addr.parse().expect("Invalid HOST:PORT format");

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Failed to bind to address. Port may be in use.");

    println!("Listening on http://{}", address);

    let jwt_secret = config.jwt_secret.clone();
    let search_service = SearchService::new(&config, DEFAULT_INDEX_NAME.to_string()).await?;

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database.");

    let state = AppState {
        db_pool: pool,
        jwt_secret: jwt_secret,
        search_service: search_service,
    };

    let state = Arc::new(state);

    let api_router = axum::Router::new()
        .route("/search", get(get_search_results))
        .route("/posts", get(get_posts));
    // .route("/blog-update/:id", put(""));

    let router = axum::Router::new()
        .nest("/api", api_router)
        .with_state(state);

    if let Err(e) = axum::serve(listener, router).await {
        panic!("Server exited with error: {}", e);
    }

    Ok(())
}
