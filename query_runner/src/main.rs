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
use serde_json::{Value, json};
use tower_http::cors::{CorsLayer, Any};
use tracing::{info, error};
use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use tokio_postgres::{NoTls, Row};

// Initialize tracing
fn setup_logging() {
    tracing_subscriber::fmt::init();
}

#[derive(Clone)]
struct AppState {
    db_url: String,
}

// Request model
#[derive(Deserialize)]
struct ExecuteQueryRequest {
    query: String,
    params: Option<HashMap<String, Value>>,
}

// Response model
#[derive(Serialize)]
struct ExecuteQueryResponse {
    results: Vec<HashMap<String, Value>>,
    metadata: HashMap<String, Value>,
}

// Error handling
enum AppError {
    InternalError(String),
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "error": msg
                }))).into_response()
            },
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, Json(json!({
                    "error": msg
                }))).into_response()
            }
        }
    }
}

// Root endpoint
async fn root() -> impl IntoResponse {
    Json(json!({
        "status": "ok", 
        "message": "Query Runner Service is running"
    }))
}

// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok"
    }))
}

// Execute query endpoint
async fn execute_query(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ExecuteQueryRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Executing query: {}", request.query);
    
    // Connect to the database
    let client = match tokio_postgres::connect(&state.db_url, NoTls).await {
        Ok((client, connection)) => {
            // Handle the connection in the background
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    error!("Connection error: {}", e);
                }
            });
            
            client
        },
        Err(e) => {
            let error_msg = format!("Database connection error: {}", e);
            error!("{}", error_msg);
            return Err(AppError::InternalError(error_msg));
        }
    };
    
    // Execute the query
    let start_time = std::time::Instant::now();
    
    let rows = match client.query(&request.query, &[]).await {
        Ok(rows) => rows,
        Err(e) => {
            let error_msg = format!("Query execution error: {}", e);
            error!("{}", error_msg);
            return Err(AppError::BadRequest(error_msg));
        }
    };
    
    let query_time = start_time.elapsed().as_millis();
    
    // Convert rows to the expected output format
    let results = match rows_to_json(rows) {
        Ok(data) => data,
        Err(e) => {
            let error_msg = format!("Error converting query results: {}", e);
            error!("{}", error_msg);
            return Err(AppError::InternalError(error_msg));
        }
    };
    
    // Get column names if available
    let column_names = if !results.is_empty() {
        let first_row = &results[0];
        first_row.keys().map(|k| k.to_string()).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    
    // Create the response
    let metadata = HashMap::from([
        ("row_count".to_string(), json!(results.len())),
        ("query_execution_time_ms".to_string(), json!(query_time)),
        ("column_names".to_string(), json!(column_names)),
    ]);
    
    let response = ExecuteQueryResponse {
        results,
        metadata,
    };
    
    Ok(Json(response))
}

// Helper function to convert Postgres rows to JSON-compatible format
fn rows_to_json(rows: Vec<Row>) -> Result<Vec<HashMap<String, Value>>, anyhow::Error> {
    let mut result = Vec::new();
    
    for row in rows {
        let mut row_data = HashMap::new();
        
        for i in 0..row.len() {
            let column_name = match row.columns()[i].name() {
                name => name.to_string(),
            };
            
            // Handle different data types
            let value = if let Ok(v) = row.try_get::<_, Option<i32>>(i) {
                match v {
                    Some(val) => json!(val),
                    None => Value::Null,
                }
            } else if let Ok(v) = row.try_get::<_, Option<i64>>(i) {
                match v {
                    Some(val) => json!(val),
                    None => Value::Null,
                }
            } else if let Ok(v) = row.try_get::<_, Option<f64>>(i) {
                match v {
                    Some(val) => json!(val),
                    None => Value::Null,
                }
            } else if let Ok(v) = row.try_get::<_, Option<String>>(i) {
                match v {
                    Some(val) => json!(val),
                    None => Value::Null,
                }
            } else if let Ok(v) = row.try_get::<_, Option<bool>>(i) {
                match v {
                    Some(val) => json!(val),
                    None => Value::Null,
                }
            } else {
                // If we can't determine the type, convert to string representation
                json!(format!("{:?}", row.get::<_, Value>(i)))
            };
            
            row_data.insert(column_name, value);
        }
        
        result.push(row_data);
    }
    
    Ok(result)
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Set up logging
    setup_logging();
    
    // Get database URL from environment
    let db_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            error!("DATABASE_URL environment variable not set");
            std::process::exit(1);
        }
    };
    
    // Create app state
    let state = Arc::new(AppState {
        db_url,
    });
    
    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/execute-query", post(execute_query))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any)
        )
        .with_state(state);
        
    // Get the port from environment or use default
    let port = env::var("QUERY_RUNNER_PORT")
        .unwrap_or_else(|_| "8003".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
        
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Query Runner listening on {}", addr);
    
    // Run the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
