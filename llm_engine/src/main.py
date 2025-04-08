import os
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from dotenv import load_dotenv
import uvicorn
import logging
from typing import Optional, List, Dict, Any
import json

# Import local modules
from src.llm_processor import process_natural_language_query
from src.db_schema import get_database_schema

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
    title="Lucidata LLM Query Engine",
    description="Natural language to SQL query processor",
    version="0.1.0"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, restrict this to actual origins
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Request models
class QueryRequest(BaseModel):
    query: str
    model: Optional[str] = None

class QueryResponse(BaseModel):
    sql_query: str
    explanation: Optional[str] = None
    confidence: Optional[float] = None

@app.get("/")
def read_root():
    return {"status": "ok", "message": "LLM Query Engine is running"}

@app.get("/health")
def health_check():
    return {"status": "ok"}

@app.post("/process-query", response_model=QueryResponse)
async def process_query(request: QueryRequest):
    try:
        # Get database schema for context
        db_schema = await get_database_schema()
        
        # Process the query using the LLM
        result = await process_natural_language_query(
            query=request.query,
            model=request.model,
            db_schema=db_schema
        )
        
        return result
    except Exception as e:
        logger.error(f"Error processing query: {str(e)}")
        raise HTTPException(
            status_code=500,
            detail=f"Error processing query: {str(e)}"
        )

if __name__ == "__main__":
    port = int(os.getenv("LLM_ENGINE_PORT", "8001"))
    uvicorn.run("src.main:app", host="0.0.0.0", port=port, reload=True)
