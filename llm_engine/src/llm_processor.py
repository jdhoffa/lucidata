import os
import logging
from typing import Dict, Any, Optional
import openai
from dotenv import load_dotenv

load_dotenv()

logger = logging.getLogger(__name__)

# Load OpenAI API key from environment
openai.api_key = os.getenv("LLM_API_KEY")

async def process_natural_language_query(
    query: str, 
    model: Optional[str] = None, 
    db_schema: Dict[str, Any] = None
) -> Dict[str, Any]:
    """
    Process a natural language query and convert it to SQL using LLM
    
    Args:
        query: The natural language query from the user
        model: Optional model name to use (defaults to environment variable)
        db_schema: Database schema information for context
        
    Returns:
        Dictionary with SQL query and explanation
    """
    model_name = model or os.getenv("LLM_MODEL", "gpt-4")
    
    # Get prompt template
    prompt = generate_prompt(query, db_schema)
    
    try:
        # Call OpenAI API
        response = await call_llm_api(prompt, model_name)
        
        # Parse the response to extract SQL
        sql_query, explanation, confidence = parse_llm_response(response)
        
        logger.info(f"Processed query: '{query}' -> SQL: '{sql_query}'")
        
        return {
            "sql_query": sql_query,
            "explanation": explanation,
            "confidence": confidence
        }
    except Exception as e:
        logger.error(f"Error processing query with LLM: {str(e)}")
        raise

async def call_llm_api(prompt: str, model_name: str) -> str:
    """Call the LLM API and get a response"""
    try:
        response = openai.ChatCompletion.create(
            model=model_name,
            messages=[
                {"role": "system", "content": "You are a helpful assistant that translates natural language questions into SQL queries for a PostgreSQL database."},
                {"role": "user", "content": prompt}
            ],
            temperature=0.1,  # Lower temperature for more deterministic outputs
            max_tokens=500
        )
        
        return response.choices[0].message.content
    except Exception as e:
        logger.error(f"Error calling LLM API: {str(e)}")
        raise

def generate_prompt(query: str, db_schema: Dict[str, Any]) -> str:
    """Generate a prompt for the LLM with the query and schema context"""
    # Convert schema to a readable format
    schema_str = format_schema_for_prompt(db_schema)
    
    # Create the prompt with the specific task
    prompt = f"""
Given the following PostgreSQL database schema:

{schema_str}

Translate this natural language question into a valid SQL query:
"{query}"

Return the answer in the following format:
SQL: <the SQL query>
EXPLANATION: <brief explanation of how the query works>
CONFIDENCE: <a number from 0 to 1 indicating confidence>

Make sure the SQL is valid PostgreSQL syntax, contains no syntax errors, and would run correctly against the described database.
"""
    return prompt

def format_schema_for_prompt(db_schema: Dict[str, Any]) -> str:
    """Format database schema into a string for the prompt"""
    if not db_schema:
        return "No schema available."
    
    schema_parts = []
    
    for table_name, table_info in db_schema.items():
        columns = table_info.get("columns", [])
        columns_str = ", ".join([f"{col['name']} {col['type']}" for col in columns])
        schema_parts.append(f"Table: {table_name} ({columns_str})")
    
    return "\n".join(schema_parts)

def parse_llm_response(response: str) -> tuple:
    """Extract SQL, explanation, and confidence from LLM response"""
    sql = None
    explanation = None
    confidence = 0.0
    
    # Extract SQL query
    if "SQL:" in response:
        sql_parts = response.split("SQL:", 1)[1].split("EXPLANATION:", 1)
        sql = sql_parts[0].strip()
    
    # Extract explanation if available
    if "EXPLANATION:" in response:
        explanation_parts = response.split("EXPLANATION:", 1)[1].split("CONFIDENCE:", 1)
        explanation = explanation_parts[0].strip()
    
    # Extract confidence if available
    if "CONFIDENCE:" in response:
        confidence_str = response.split("CONFIDENCE:", 1)[1].strip()
        try:
            confidence = float(confidence_str)
        except ValueError:
            confidence = 0.5  # Default if parsing fails
    
    if not sql:
        logger.warning("Could not extract SQL from LLM response")
        sql = "SELECT * FROM cars LIMIT 10;"  # Fallback query
    
    return sql, explanation, confidence
