use std::env;
use std::net::SocketAddr;
use axum::{
    routing::{get, post},
    Router,
    Json,
    http::StatusCode,
    extract::State,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{CorsLayer, Any};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use anyhow::Result;
use std::sync::Arc;

mod llm_processor;
mod db_schema;

use llm_processor::process_natural_language_query;
use db_schema::get_database_schema;

#[derive(Clone)]
struct AppState {
    // Add any shared state here if needed
}

// Request model
#[derive(Deserialize)]
struct QueryRequest {
    query: String,
    model: Option<String>,
}

// Response model
#[derive(Serialize)]
struct QueryResponse {
    sql_query: String,
    explanation: Option<String>,
    confidence: Option<f64>,
}

// Error handling
enum AppError {
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": msg
                }))).into_response()
            }
        }
    }
}

// Root endpoint
async fn root() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok", 
        "message": "LLM Query Engine is running"
    }))
}

// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok"
    }))
}

// Process query endpoint
async fn process_query(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<QueryRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Processing query: {}", request.query);
    
    match get_database_schema().await {
        Ok(db_schema) => {
            match process_natural_language_query(
                request.query,
                request.model,
                Some(db_schema),
            ).await {
                Ok(result) => {
                    let response = QueryResponse {
                        sql_query: result.sql_query,
                        explanation: result.explanation,
                        confidence: result.confidence,
                    };
                    Ok(Json(response))
                },
                Err(e) => {
                    let error_msg = format!("Error processing query: {}", e);
                    error!("{}", error_msg);
                    Err(AppError::InternalError(error_msg))
                }
            }
        },
        Err(e) => {
            let error_msg = format!("Error getting database schema: {}", e);
            error!("{}", error_msg);
            Err(AppError::InternalError(error_msg))
        }
    }
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Create app state
    let state = Arc::new(AppState {});
    
    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/process-query", post(process_query))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any)
        )
        .with_state(state);
        
    // Get the port from environment or use default
    let port = env::var("LLM_ENGINE_PORT")
        .unwrap_or_else(|_| "8001".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
        
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("LLM Engine listening on {}", addr);
    
    // Run the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
