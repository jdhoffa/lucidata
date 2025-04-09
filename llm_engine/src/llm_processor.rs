use std::env;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::{Result, anyhow};
use tracing::{info, error};
use reqwest::Client;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub sql_query: String,
    pub explanation: Option<String>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f64,
    max_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

/// Process a natural language query and convert it to SQL using LLM
pub async fn process_natural_language_query(
    query: String,
    model: Option<String>,
    db_schema: Option<HashMap<String, Value>>,
) -> Result<QueryResult> {
    // Get model name from parameters or environment
    let model_name = model
        .or_else(|| env::var("LLM_MODEL").ok())
        .unwrap_or_else(|| "gpt-4".to_string());
    
    // Generate prompt with query and schema context
    let prompt = generate_prompt(&query, db_schema.as_ref());
    
    // Call the LLM API
    match call_llm_api(&prompt, &model_name).await {
        Ok(response) => {
            // Parse the LLM response to extract SQL
            let (sql_query, explanation, confidence) = parse_llm_response(&response);
            
            info!("Processed query: '{}' -> SQL: '{}'", query, sql_query);
            
            Ok(QueryResult {
                sql_query,
                explanation,
                confidence,
            })
        },
        Err(e) => {
            error!("Error calling LLM API: {}", e);
            Err(anyhow!("Error processing query with LLM: {}", e))
        }
    }
}

/// Call the OpenAI API to get response for the prompt
async fn call_llm_api(prompt: &str, model_name: &str) -> Result<String> {
    let api_key = env::var("LLM_API_KEY").map_err(|_| anyhow!("LLM_API_KEY environment variable not set"))?;
    
    let client = Client::new();
    
    let request = OpenAIRequest {
        model: model_name.to_string(),
        messages: vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant that translates natural language questions into SQL queries for a PostgreSQL database.".to_string(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
        temperature: 0.1,
        max_tokens: 500,
    };
    
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await?;
        return Err(anyhow!("OpenAI API error: {} - {}", status, text));
    }
    
    let response_data: OpenAIResponse = response.json().await?;
    
    if let Some(choice) = response_data.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err(anyhow!("Empty response from OpenAI API"))
    }
}

/// Generate a prompt for the LLM with the query and schema context
fn generate_prompt(query: &str, db_schema: Option<&HashMap<String, Value>>) -> String {
    // Convert schema to a readable format for the prompt
    let schema_str = match db_schema {
        Some(schema) => format_schema_for_prompt(schema),
        None => "No schema available.".to_string(),
    };
    
    // Create the prompt with the specific task
    format!(
        r#"
Given the following PostgreSQL database schema:

{}

Translate this natural language question into a valid SQL query:
"{}"

Return the answer in the following format:
SQL: <the SQL query>
EXPLANATION: <brief explanation of how the query works>
CONFIDENCE: <a number from 0 to 1 indicating confidence>

Make sure the SQL is valid PostgreSQL syntax, contains no syntax errors, and would run correctly against the described database.
"#,
        schema_str,
        query
    )
}

/// Format database schema into a string for the prompt
fn format_schema_for_prompt(db_schema: &HashMap<String, Value>) -> String {
    if db_schema.is_empty() {
        return "No schema available.".to_string();
    }
    
    let mut schema_parts = Vec::new();
    
    for (table_name, table_info) in db_schema {
        if let Some(columns) = table_info.get("columns").and_then(Value::as_array) {
            let columns_str: Vec<String> = columns
                .iter()
                .filter_map(|col| {
                    let name = col.get("name")?.as_str()?;
                    let col_type = col.get("type")?.as_str()?;
                    Some(format!("{} {}", name, col_type))
                })
                .collect();
                
            schema_parts.push(format!(
                "Table: {} ({})",
                table_name,
                columns_str.join(", ")
            ));
        }
    }
    
    schema_parts.join("\n")
}

/// Extract SQL, explanation, and confidence from LLM response
fn parse_llm_response(response: &str) -> (String, Option<String>, Option<f64>) {
    let mut sql = None;
    let mut explanation = None;
    let mut confidence = None;
    
    // Extract SQL query
    if response.contains("SQL:") {
        let parts: Vec<&str> = response.split("SQL:").collect();
        if parts.len() >= 2 {
            let sql_parts: Vec<&str> = parts[1].split("EXPLANATION:").collect();
            sql = Some(sql_parts[0].trim().to_string());
        }
    }
    
    // Extract explanation if available
    if response.contains("EXPLANATION:") {
        let parts: Vec<&str> = response.split("EXPLANATION:").collect();
        if parts.len() >= 2 {
            let explanation_parts: Vec<&str> = parts[1].split("CONFIDENCE:").collect();
            explanation = Some(explanation_parts[0].trim().to_string());
        }
    }
    
    // Extract confidence if available
    if response.contains("CONFIDENCE:") {
        let parts: Vec<&str> = response.split("CONFIDENCE:").collect();
        if parts.len() >= 2 {
            let confidence_str = parts[1].trim();
            if let Ok(conf_value) = confidence_str.parse::<f64>() {
                confidence = Some(conf_value);
            }
        }
    }
    
    // Default to a fallback query if no SQL extracted
    let final_sql = sql.unwrap_or_else(|| {
        error!("Could not extract SQL from LLM response");
        "SELECT * FROM cars LIMIT 10;".to_string()
    });
    
    (final_sql, explanation, confidence)
}
