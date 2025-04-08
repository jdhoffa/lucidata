import os
import logging
import psycopg2
from typing import Dict, Any, List
from dotenv import load_dotenv

load_dotenv()

logger = logging.getLogger(__name__)

async def get_database_schema() -> Dict[str, Any]:
    """
    Retrieve the database schema from the PostgreSQL database
    Returns a dictionary representation of tables and columns
    """
    db_url = os.getenv("DATABASE_URL")
    if not db_url:
        logger.warning("DATABASE_URL not set, using hardcoded schema")
        return get_hardcoded_schema()
    
    try:
        conn = psycopg2.connect(db_url)
        cursor = conn.cursor()
        
        # Query to get all tables
        cursor.execute("""
            SELECT tablename FROM pg_catalog.pg_tables
            WHERE schemaname = 'public';
        """)
        
        tables = cursor.fetchall()
        schema = {}
        
        for (table_name,) in tables:
            # Query to get columns for this table
            cursor.execute(f"""
                SELECT column_name, data_type, is_nullable
                FROM information_schema.columns
                WHERE table_name = '{table_name}';
            """)
            
            columns = []
            for column_name, data_type, is_nullable in cursor.fetchall():
                columns.append({
                    "name": column_name,
                    "type": data_type,
                    "nullable": is_nullable == "YES"
                })
            
            schema[table_name] = {
                "columns": columns
            }
        
        conn.close()
        return schema
        
    except Exception as e:
        logger.error(f"Error fetching database schema: {str(e)}")
        # Fall back to hardcoded schema
        return get_hardcoded_schema()

def get_hardcoded_schema() -> Dict[str, Any]:
    """
    Return a hardcoded schema for the cars table
    Used as fallback when database connection fails
    """
    return {
        "cars": {
            "columns": [
                {"name": "id", "type": "integer", "nullable": False},
                {"name": "model", "type": "varchar(50)", "nullable": False},
                {"name": "mpg", "type": "numeric(5,1)", "nullable": True},
                {"name": "cyl", "type": "integer", "nullable": True},
                {"name": "disp", "type": "numeric(6,1)", "nullable": True},
                {"name": "hp", "type": "integer", "nullable": True},
                {"name": "drat", "type": "numeric(4,2)", "nullable": True},
                {"name": "wt", "type": "numeric(5,3)", "nullable": True},
                {"name": "qsec", "type": "numeric(5,2)", "nullable": True},
                {"name": "vs", "type": "integer", "nullable": True},
                {"name": "am", "type": "integer", "nullable": True},
                {"name": "gear", "type": "integer", "nullable": True},
                {"name": "carb", "type": "integer", "nullable": True},
            ]
        }
    }
