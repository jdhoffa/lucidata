use std::env;
use anyhow::Result;
use tracing::{info, error};
use serde_json::{Value, json};
use std::collections::HashMap;
use tokio_postgres::NoTls;

/// Retrieve the database schema from the PostgreSQL database
/// Returns a dictionary representation of tables and columns
pub async fn get_database_schema() -> Result<HashMap<String, Value>> {
    let db_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            info!("DATABASE_URL not set, using hardcoded schema");
            return Ok(get_hardcoded_schema());
        }
    };
    
    match get_schema_from_db(&db_url).await {
        Ok(schema) => Ok(schema),
        Err(e) => {
            error!("Error fetching database schema: {}", e);
            info!("Falling back to hardcoded schema");
            Ok(get_hardcoded_schema())
        }
    }
}

/// Query the database to get its schema
async fn get_schema_from_db(db_url: &str) -> Result<HashMap<String, Value>> {
    // Connect to the database
    let (client, connection) = tokio_postgres::connect(db_url, NoTls).await?;
    
    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });
    
    // Query to get all tables
    let tables_query = "
        SELECT tablename FROM pg_catalog.pg_tables
        WHERE schemaname = 'public';
    ";
    
    let mut schema = HashMap::new();
    
    for row in client.query(tables_query, &[]).await? {
        let table_name: String = row.get(0);
        
        // Query to get columns for this table
        let columns_query = format!(
            "
                SELECT column_name, data_type, is_nullable
                FROM information_schema.columns
                WHERE table_name = '{}';
            ",
            table_name
        );
        
        let mut columns = Vec::new();
        
        for col_row in client.query(&columns_query, &[]).await? {
            let column_name: String = col_row.get(0);
            let data_type: String = col_row.get(1);
            let is_nullable: String = col_row.get(2);
            
            columns.push(json!({
                "name": column_name,
                "type": data_type,
                "nullable": is_nullable == "YES"
            }));
        }
        
        schema.insert(table_name, json!({
            "columns": columns
        }));
    }
    
    Ok(schema)
}

/// Return a hardcoded schema for the cars table
/// Used as fallback when database connection fails
fn get_hardcoded_schema() -> HashMap<String, Value> {
    let mut schema = HashMap::new();
    
    let columns = json!([
        {"name": "id", "type": "integer", "nullable": false},
        {"name": "model", "type": "varchar(50)", "nullable": false},
        {"name": "mpg", "type": "numeric(5,1)", "nullable": true},
        {"name": "cyl", "type": "integer", "nullable": true},
        {"name": "disp", "type": "numeric(6,1)", "nullable": true},
        {"name": "hp", "type": "integer", "nullable": true},
        {"name": "drat", "type": "numeric(4,2)", "nullable": true},
        {"name": "wt", "type": "numeric(5,3)", "nullable": true},
        {"name": "qsec", "type": "numeric(5,2)", "nullable": true},
        {"name": "vs", "type": "integer", "nullable": true},
        {"name": "am", "type": "integer", "nullable": true},
        {"name": "gear", "type": "integer", "nullable": true},
        {"name": "carb", "type": "integer", "nullable": true}
    ]);
    
    schema.insert("cars".to_string(), json!({
        "columns": columns
    }));
    
    schema
}
