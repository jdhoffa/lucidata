import os
import logging
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import uvicorn
from typing import Optional, Dict, Any, List
import psycopg2
import psycopg2.extras
import requests
import json
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    handlers=[logging.StreamHandler()]
)

logger = logging.getLogger(__name__)

# Initialize FastAPI app
app = FastAPI(
    title="Lucidata Query Runner",
    description="Execute SQL queries against the database",
    version="0.1.0"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Request models
class ExecuteQueryRequest(BaseModel):
    query: str
    params: Optional[Dict[str, Any]] = None

class ExecuteQueryResponse(BaseModel):
    results: List[Dict[str, Any]]
    metadata: Dict[str, Any]

@app.get("/")
def read_root():
    return {"status": "ok", "message": "Query Runner Service is running"}

@app.get("/health")
def health_check():
    return {"status": "ok"}

@app.post("/execute-query", response_model=ExecuteQueryResponse)
async def execute_query(request: ExecuteQueryRequest):
    """Execute a SQL query against the database"""
    try:
        db_url = os.getenv("DATABASE_URL")
        if not db_url:
            raise HTTPException(status_code=500, detail="Database connection not configured")
        
        # Connect to the database
        conn = psycopg2.connect(db_url)
        cursor = conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor)
        
        # Log the query (in production, you'd want to sanitize this log)
        logger.info(f"Executing query: {request.query}")
        
        # Execute the query
        try:
            cursor.execute(request.query, request.params or {})
            results = cursor.fetchall()
            
            # Convert results to a list of dictionaries
            results_list = [dict(row) for row in results]
            
            # Get query metadata
            metadata = {
                "row_count": len(results_list),
                "query_execution_time_ms": None,  # Would need timing logic to implement
                "column_names": [desc[0] for desc in cursor.description] if cursor.description else []
            }
            
            return ExecuteQueryResponse(results=results_list, metadata=metadata)
        
        except psycopg2.Error as e:
            logger.error(f"Database query error: {e}")
            raise HTTPException(status_code=400, detail=f"Query execution error: {str(e)}")
        
        finally:
            cursor.close()
            conn.close()
    
    except Exception as e:
        logger.error(f"Error executing query: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Error: {str(e)}")

if __name__ == "__main__":
    port = int(os.getenv("QUERY_RUNNER_PORT", "8003"))
    uvicorn.run("src.main:app", host="0.0.0.0", port=port, reload=True)
