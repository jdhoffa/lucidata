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
use serde_json::Value;
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

// Request model for visualization generation
#[derive(Deserialize)]
struct VisualizationRequest {
    query: String,
    results: Value,
    model: Option<String>,
}

// Response model
#[derive(Serialize)]
struct QueryResponse {
    sql_query: String,
    explanation: Option<String>,
    confidence: Option<f64>,
}

// Response model for visualization generation
#[derive(Serialize)]
struct VisualizationResponse {
    html_code: String,
    explanation: String,
    confidence: f64,
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

// Process visualization request
async fn generate_visualization(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<VisualizationRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Generating visualization for query: {}", request.query);
    
    // Get the model name from the request or use a default
    let model = request.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string());
    
    // Format the results data for the prompt
    let results_json = serde_json::to_string_pretty(&request.results)
        .unwrap_or_else(|_| format!("{:?}", request.results));
    
    // Create the prompt for visualization generation
    let system_prompt = format!(
        "You are a data visualization expert. Your task is to create a Plotly.js visualization based on the provided data and query. \
        You must return a complete, valid HTML file that uses Plotly.js to visualize the data.\
        \
        STYLING GUIDELINES:\
        - Use Plotly.js to create the chart\
        - Use 14px font for axis labels, 18px for titles\
        - Use consistent margins and padding\
        - Use a neobrutalist style\
        - Include axis titles with units (if known)\
        - Rotate x-axis labels if they are dates or long strings\
        - Use tight layout with `autosize: true` and `responsive: true`\
        - Enable zoom and pan interactivity\
        - Enable tooltips on hover showing exact values and labels\
        - Use hovermode: 'closest'\
        \
        OUTPUT FORMAT:\
        Your response should be a complete HTML file that can be directly viewed in a browser.\
        Return valid HTML that includes the Plotly.js library from a CDN and creates the visualization.\
        Also include a brief explanation of the visualization choices you made."
    );
    
    let user_prompt = format!(
        "Natural Language Query: {}\n\nData Results:\n{}\n\nBased on this query and data, create a complete HTML file with a Plotly.js visualization.",
        request.query, results_json
    );
    
    // Call the LLM with the specialized prompt
    let llm_response = call_llm_api(&system_prompt, &user_prompt, &model).await
        .map_err(|e| {
            let error_msg = format!("Error calling LLM API: {}", e);
            error!("{}", error_msg);
            AppError::InternalError(error_msg)
        })?;
    
    // Extract the HTML code and explanation from the response
    let (html_code, explanation, confidence) = parse_visualization_response(&llm_response);
    
    Ok(Json(VisualizationResponse {
        html_code,
        explanation,
        confidence,
    }))
}

// Call LLM API with system and user prompts
async fn call_llm_api(system_prompt: &str, user_prompt: &str, model_name: &str) -> Result<String, anyhow::Error> {
    let api_key = env::var("LLM_API_KEY").map_err(|_| anyhow::anyhow!("LLM_API_KEY environment variable not set"))?;
    
    let client = reqwest::Client::new();
    
    #[derive(Serialize, Deserialize)]
    struct Message {
        role: String,
        content: String,
    }
    
    #[derive(Serialize)]
    struct OpenAIRequest {
        model: String,
        messages: Vec<Message>,
        temperature: f64,
    }
    
    let request = OpenAIRequest {
        model: model_name.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ],
        temperature: 0.7, // Slightly higher temperature for more creative visualizations
    };
    
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("LLM API returned error: {}", error_text));
    }
    
    #[derive(Deserialize)]
    struct OpenAIChoice {
        message: Message,
    }
    
    #[derive(Deserialize)]
    struct OpenAIResponse {
        choices: Vec<OpenAIChoice>,
    }
    
    let response_json: OpenAIResponse = response.json().await?;
    
    if response_json.choices.is_empty() {
        return Err(anyhow::anyhow!("LLM API returned empty choices"));
    }
    
    Ok(response_json.choices[0].message.content.clone())
}

// Parse LLM response to extract HTML, explanation and confidence
fn parse_visualization_response(response: &str) -> (String, String, f64) {
    // Try to extract HTML content - look for <!DOCTYPE html> or <html>
    let html_start_patterns = ["<!DOCTYPE html>", "<html>"];
    let mut html_code = String::new();
    let mut explanation = String::new();
    let confidence = 0.8; // Default confidence
    
    // First, check if the response contains a code block with HTML
    if let Some(html_block_start) = response.find("```html") {
        // Find the end of the code block (next ```)
        if let Some(html_block_end) = response[html_block_start + 6..].find("```") {
            // Extract HTML content (skip the ```html and end ```)
            let block_start_pos = html_block_start + "```html".len();
            let block_end_pos = html_block_start + 6 + html_block_end;
            html_code = response[block_start_pos..block_end_pos].trim().to_string();
            
            // Look for explanation after the HTML block
            if block_end_pos + 3 < response.len() {
                explanation = response[block_end_pos + 3..].trim().to_string();
            }
        }
    }
    // If no code block, try to find direct HTML
    else {
        for pattern in html_start_patterns.iter() {
            if let Some(start_idx) = response.find(pattern) {
                html_code = response[start_idx..].trim().to_string();
                
                // Everything before HTML is considered explanation
                if start_idx > 0 {
                    explanation = response[0..start_idx].trim().to_string();
                }
                break;
            }
        }
    }
    
    // If still no HTML found, look for any content between <script> tags or <div id="plot">
    if html_code.is_empty() {
        if let Some(script_start) = response.find("<script>") {
            if let Some(script_end) = response[script_start..].find("</script>") {
                // Create a basic HTML wrapper around the script
                let script_content = &response[script_start..script_start + script_end + 9];
                html_code = format!(
                    "<!DOCTYPE html>\n<html>\n<head>\n<title>Visualization</title>\n<script src=\"https://cdn.plot.ly/plotly-latest.min.js\"></script>\n</head>\n<body>\n<div id=\"plot\"></div>\n{}\n</body>\n</html>",
                    script_content
                );
                
                // Everything else is explanation
                explanation = response.replace(script_content, "").trim().to_string();
            }
        }
    }
    
    // If still nothing found, return the whole response as HTML with a warning
    if html_code.is_empty() {
        html_code = format!(
            "<!DOCTYPE html>\n<html>\n<head>\n<title>Visualization Error</title>\n</head>\n<body>\n<h1>Could not generate visualization</h1>\n<pre>{}</pre>\n</body>\n</html>",
            response.replace("<", "&lt;").replace(">", "&gt;")
        );
        explanation = "Could not parse LLM response into valid HTML visualization.".to_string();
    }
    
    // If explanation is empty, provide a default
    if explanation.is_empty() {
        explanation = "Visualization generated from the provided data.".to_string();
    }
    
    (html_code, explanation, confidence)
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
        .route("/generate", post(generate_visualization))
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
