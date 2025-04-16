use std::sync::Arc;
use std::{env, net::SocketAddr, time::Instant};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::error;

// Models for requests and responses
#[derive(Debug, Deserialize)]
struct TranslateAndExecuteRequest {
    natural_query: String,
    #[serde(default = "default_model")]
    model: String,
}

fn default_model() -> String {
    "gpt-3.5-turbo".to_string()
}

#[derive(Debug, Serialize)]
struct TranslateAndExecuteResponse {
    natural_query: String,
    sql_query: String,
    results: Value,
    explanation: String,
    metadata: ResponseMetadata,
}

#[derive(Debug, Serialize)]
struct ResponseMetadata {
    confidence: f64,
    execution_time_ms: u64,
    llm_processing_time_ms: u64,
    total_time_ms: u64,
}

// LLM Engine response structure
#[derive(Debug, Deserialize)]
struct LlmResponse {
    sql_query: String,
    explanation: String,
    confidence: f64,
}

// Application state
struct AppState {
    client: Client,
    llm_engine_url: String,
    api_url: String,
}

// Error types
#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("Failed to call LLM engine: {0}")]
    LlmEngineError(#[from] reqwest::Error),
    
    #[error("Failed to process LLM response: {0}")]
    LlmResponseError(String),
    
    #[error("Failed to execute SQL query: {0}")]
    SqlExecutionError(String),
}

// Convert AppError to Axum Response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::LlmEngineError(e) => (StatusCode::BAD_GATEWAY, format!("LLM engine error: {}", e)),
            AppError::LlmResponseError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::SqlExecutionError(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables, don't require .env file in Docker
    let _ = dotenv::dotenv();
    
    // Initialize logging
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Initialize application state
    let llm_engine_url = env::var("LLM_ENGINE_URL")
        .expect("LLM_ENGINE_URL must be set");
    
    let api_url = env::var("API_URL")
        .expect("API_URL must be set");
    
    let state = Arc::new(AppState {
        client: Client::new(),
        llm_engine_url,
        api_url,
    });

    // Create middleware stack with CORS
    let middleware = ServiceBuilder::new()
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        );

    // Create the router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/translate-and-execute", post(translate_and_execute))
        .with_state(state)
        .layer(middleware);

    // Get port from environment variable or use default
    let port = env::var("QUERY_ROUTER_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8003);
        
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Query Router server starting on {}", addr);
    
    // Start the server with graceful error handling for address-in-use errors
    match axum::Server::try_bind(&addr) {
        Ok(server) => {
            server.serve(app.into_make_service()).await?;
        },
        Err(err) => {
            if err.to_string().contains("Address already in use") {
                tracing::error!("Port {} is already in use. Try setting a different port with the PORT environment variable.", port);
                std::process::exit(1);
            } else {
                return Err(err.into());
            }
        }
    };
    
    Ok(())
}

// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

// Main endpoint for translating natural language to SQL and executing it
async fn translate_and_execute(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TranslateAndExecuteRequest>,
) -> Result<Json<TranslateAndExecuteResponse>, AppError> {
    let start_time = Instant::now();
    
    // Step 1: Forward the natural language query to the LLM engine
    let llm_start_time = Instant::now();
    let llm_response = call_llm_engine(&state, &request).await?;
    let llm_processing_time = llm_start_time.elapsed().as_millis() as u64;
    
    // Step 2: Send the generated SQL to the API execution endpoint
    let execution_start_time = Instant::now();
    let query_result = execute_sql_query(&state, &llm_response.sql_query).await?;
    let execution_time = execution_start_time.elapsed().as_millis() as u64;
    
    // Step 3: Combine results and return to client
    let total_time = start_time.elapsed().as_millis() as u64;
    
    let response = TranslateAndExecuteResponse {
        natural_query: request.natural_query,
        sql_query: llm_response.sql_query,
        results: query_result,
        explanation: llm_response.explanation,
        metadata: ResponseMetadata {
            confidence: llm_response.confidence,
            execution_time_ms: execution_time,
            llm_processing_time_ms: llm_processing_time,
            total_time_ms: total_time,
        },
    };
    
    Ok(Json(response))
}

// Call LLM engine to convert natural language to SQL
async fn call_llm_engine(
    state: &AppState, 
    request: &TranslateAndExecuteRequest
) -> Result<LlmResponse, AppError> {
    let url = format!("{}/process-query", state.llm_engine_url);
    
    let llm_request = json!({
        "query": request.natural_query,
        "model": request.model
    });
    
    let response = state.client
        .post(&url)
        .json(&llm_request)
        .send()
        .await
        .map_err(AppError::LlmEngineError)?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(AppError::LlmResponseError(format!("LLM engine returned error ({}): {}", status, error_text)));
    }
    
    let llm_response = response.json::<LlmResponse>().await
        .map_err(|e| AppError::LlmResponseError(format!("Failed to parse LLM response: {}", e)))?;
    
    Ok(llm_response)
}

// Execute SQL using the API service
async fn execute_sql_query(state: &AppState, sql_query: &str) -> Result<Value, AppError> {
    let url = format!("{}/api/query", state.api_url);
    
    let query_request = json!({
        "query": sql_query
    });
    
    let response = state.client
        .post(&url)
        .json(&query_request)
        .send()
        .await
        .map_err(|e| AppError::SqlExecutionError(e.to_string()))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(AppError::SqlExecutionError(format!("SQL execution failed ({}): {}", status, error_text)));
    }
    
    let json_response: serde_json::Value = response.json().await
        .map_err(|e| AppError::SqlExecutionError(format!("Failed to parse API response: {}", e)))?;
    
    // Extract the results field from the API response
    let result = json_response["result"].clone();
    if result.is_null() {
        return Err(AppError::SqlExecutionError("API returned null result".to_string()));
    }
    
    Ok(result)
}
