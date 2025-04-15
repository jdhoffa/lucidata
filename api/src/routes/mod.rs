use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Json as RequestJson,
};
use serde_json::{json, Value};
use sqlx::{PgPool, Row, Column, TypeInfo};

use crate::models::{Car, QueryRequest, QueryResponse};

// Health check endpoint
pub async fn health_check() -> &'static str {
    "OK"
}

// Get all cars
pub async fn get_cars(State(pool): State<PgPool>) -> Result<Json<Vec<Car>>, (StatusCode, String)> {
    let cars = sqlx::query_as::<_, Car>("SELECT * FROM cars")
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(Json(cars))
}

// Get a single car by ID
pub async fn get_car_by_id(
    Path(id): Path<i32>,
    State(pool): State<PgPool>,
) -> Result<Json<Car>, (StatusCode, String)> {
    let car = sqlx::query_as::<_, Car>("SELECT * FROM cars WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, format!("Car with id {} not found", id)),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ),
        })?;

    Ok(Json(car))
}

// Execute a raw SQL query
pub async fn query(
    State(pool): State<PgPool>,
    RequestJson(payload): RequestJson<QueryRequest>,
) -> Result<Json<QueryResponse>, (StatusCode, String)> {
    // Note: In a production environment, you'd want to validate and sanitize this query
    // or use a query builder to prevent SQL injection
    let query = payload.query;
    
    let rows = sqlx::query(&query)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Query error: {}", e),
            )
        })?;

    // Convert the rows to a JSON array
    let result = rows
        .iter()
        .map(|row| {
            let mut map = serde_json::Map::new();
            for i in 0..row.len() {
                let column_name = row.column(i).name();
                // Type info is used only in the fallback case
                
                // Handle different data types explicitly
                let value = if let Ok(v) = row.try_get::<Option<i32>, _>(i) {
                    match v {
                        Some(val) => json!(val),
                        None => Value::Null,
                    }
                } else if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
                    match v {
                        Some(val) => json!(val),
                        None => Value::Null,
                    }
                } else if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
                    match v {
                        Some(val) => json!(val),
                        None => Value::Null,
                    }
                } else if let Ok(v) = row.try_get::<Option<String>, _>(i) {
                    match v {
                        Some(val) => json!(val),
                        None => Value::Null,
                    }
                } else if let Ok(v) = row.try_get::<Option<bool>, _>(i) {
                    match v {
                        Some(val) => json!(val),
                        None => Value::Null,
                    }
                } else {
                    // For any other types, try simpler approaches
                    let type_info = row.column(i).type_info();
                    let type_name = type_info.name();
                    
                    // Try to decode as JSON value first (works for many types)
                    if let Ok(v) = row.try_get::<Option<serde_json::Value>, _>(i) {
                        match v {
                            Some(val) => val,
                            None => Value::Null,
                        }
                    } else {
                        // Try to get as a string (most types can be represented as strings)
                        if let Ok(v) = row.try_get::<Option<String>, _>(i) {
                            match v {
                                Some(s) => json!(s),
                                None => Value::Null,
                            }
                        } else {
                            // If all else fails, return the type name as a fallback
                            json!(format!("Value of type: {}", type_name))
                        }
                    }
                };
                
                map.insert(column_name.to_string(), value);
            }
            Value::Object(map)
        })
        .collect::<Vec<Value>>();

    Ok(Json(QueryResponse {
        result: json!(result),
        executed_query: query,
    }))
}
