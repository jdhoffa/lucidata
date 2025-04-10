mod models;
mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load environment variables, don't require .env file in Docker
    let _ = dotenv();
    
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Database connection
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    tracing::info!("Connecting to database at: {}", database_url.replace(|c| c != '@' && c != ':', "*"));
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");
        
    tracing::info!("Successfully connected to database");
    
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    // Router setup
    let app = Router::new()
        .route("/", get(routes::health_check))
        .route("/api/health", get(routes::health_check))
        .route("/api/cars", get(routes::get_cars))
        .route("/api/cars/:id", get(routes::get_car_by_id))
        .route("/api/query", post(routes::query))
        .layer(cors)
        .with_state(pool);
    
    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
