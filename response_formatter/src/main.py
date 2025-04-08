import os
import logging
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import uvicorn
from typing import Optional, Dict, Any, List
import pandas as pd
import json
from dotenv import load_dotenv
import io
import base64
import matplotlib.pyplot as plt

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
    title="Lucidata Response Formatter",
    description="Format query results into various outputs",
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

# Request and response models
class FormatterRequest(BaseModel):
    data: List[Dict[str, Any]]
    format: str = "html"  # html, csv, json
    visualization_type: Optional[str] = None  # bar, line, pie, etc.
    title: Optional[str] = None
    description: Optional[str] = None

class FormatterResponse(BaseModel):
    formatted_data: str
    visualization: Optional[str] = None
    content_type: str

@app.get("/")
def read_root():
    return {"status": "ok", "message": "Response Formatter Service is running"}

@app.get("/health")
def health_check():
    return {"status": "ok"}

@app.post("/format", response_model=FormatterResponse)
async def format_data(request: FormatterRequest):
    """Format the data in the specified format"""
    try:
        # Convert data to pandas DataFrame for easier manipulation
        df = pd.DataFrame(request.data)
        
        if df.empty:
            return FormatterResponse(
                formatted_data="No data to format",
                content_type="text/plain"
            )
        
        # Format based on request
        if request.format.lower() == "csv":
            # Generate CSV
            csv_buffer = io.StringIO()
            df.to_csv(csv_buffer, index=False)
            formatted_data = csv_buffer.getvalue()
            content_type = "text/csv"
            
        elif request.format.lower() == "json":
            # Generate JSON
            formatted_data = df.to_json(orient="records")
            content_type = "application/json"
            
        else:  # Default to HTML
            # Generate HTML table
            html_table = df.to_html(
                classes=["table", "table-striped", "table-hover"], 
                index=False,
                border=0
            )
            
            # Add wrapper with title and description if provided
            formatted_data = f"""
            <div class="lucidata-result">
                {f'<h3>{request.title}</h3>' if request.title else ''}
                {f'<p>{request.description}</p>' if request.description else ''}
                {html_table}
            </div>
            """
            content_type = "text/html"
        
        # Generate visualization if requested
        visualization = None
        if request.visualization_type:
            visualization = generate_visualization(df, request.visualization_type, request.title)
        
        return FormatterResponse(
            formatted_data=formatted_data,
            visualization=visualization,
            content_type=content_type
        )
    
    except Exception as e:
        logger.error(f"Error formatting data: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Error formatting data: {str(e)}")

def generate_visualization(df, viz_type, title=None):
    """Generate a visualization based on the data and type"""
    try:
        plt.figure(figsize=(10, 6))
        
        if viz_type == "bar":
            # Use the first column as x and second as y if available
            if len(df.columns) >= 2:
                df.plot(kind='bar', x=df.columns[0], y=df.columns[1])
            else:
                df.plot(kind='bar')
                
        elif viz_type == "line":
            df.plot(kind='line')
            
        elif viz_type == "pie" and len(df.columns) >= 2:
            # Use first column for labels and second for values
            df.plot(kind='pie', y=df.columns[1], labels=df[df.columns[0]])
            
        else:
            # Default to a simple bar chart of the first numeric column
            numeric_cols = df.select_dtypes(include=['number']).columns
            if len(numeric_cols) > 0:
                df.plot(kind='bar', y=numeric_cols[0])
            else:
                return None  # No numeric data to visualize
        
        if title:
            plt.title(title)
            
        plt.tight_layout()
        
        # Save the plot to a bytes buffer
        buffer = io.BytesIO()
        plt.savefig(buffer, format='png')
        buffer.seek(0)
        
        # Convert to base64 for embedding in HTML
        image_base64 = base64.b64encode(buffer.getvalue()).decode('utf-8')
        plt.close()
        
        return f"data:image/png;base64,{image_base64}"
    
    except Exception as e:
        logger.error(f"Error generating visualization: {str(e)}")
        return None

if __name__ == "__main__":
    port = int(os.getenv("FORMATTER_PORT", "8002"))
    uvicorn.run("src.main:app", host="0.0.0.0", port=port, reload=True)
