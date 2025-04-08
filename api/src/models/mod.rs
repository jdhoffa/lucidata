use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Car {
    pub id: i32,
    pub model: String,
    pub mpg: Option<f64>,
    pub cyl: Option<i32>,
    pub disp: Option<f64>,
    pub hp: Option<i32>,
    pub drat: Option<f64>,
    pub wt: Option<f64>,
    pub qsec: Option<f64>,
    pub vs: Option<i32>,
    pub am: Option<i32>,
    pub gear: Option<i32>,
    pub carb: Option<i32>,
}

#[derive(Deserialize)]
pub struct QueryRequest {
    pub query: String,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub result: serde_json::Value,
    pub executed_query: String,
}
