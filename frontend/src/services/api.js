import axios from 'axios';

// Get API URL from environment variables or fallback to localhost
const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8000';
const LLM_ENGINE_URL = process.env.REACT_APP_LLM_ENGINE_URL || 'http://localhost:8001';

/**
 * Send a natural language query to the LLM engine
 * @param {string} queryText - The natural language query
 * @returns {Promise<Object>} - The query result
 */
export const sendQuery = async (queryText) => {
  try {
    // First, send the natural language query to the LLM engine to translate it
    const llmResponse = await axios.post(`${LLM_ENGINE_URL}/process-query`, {
      query: queryText
    });
    
    // Check if we received a valid SQL query from the LLM
    if (!llmResponse.data || !llmResponse.data.sql_query) {
      throw new Error('Failed to translate natural language to SQL');
    }
    
    // Send the SQL query to the API
    const apiResponse = await axios.post(`${API_URL}/api/query`, {
      query: llmResponse.data.sql_query
    });
    
    return {
      result: apiResponse.data.result,
      executed_query: llmResponse.data.sql_query,
    };
  } catch (error) {
    console.error('Error processing query:', error);
    throw error;
  }
};

/**
 * Fetch all cars directly from the API
 * @returns {Promise<Array>} - Array of car objects
 */
export const fetchAllCars = async () => {
  try {
    const response = await axios.get(`${API_URL}/api/cars`);
    return response.data;
  } catch (error) {
    console.error('Error fetching cars:', error);
    throw error;
  }
};

/**
 * Fetch a specific car by ID
 * @param {number} id - The car ID
 * @returns {Promise<Object>} - Car object
 */
export const fetchCarById = async (id) => {
  try {
    const response = await axios.get(`${API_URL}/api/cars/${id}`);
    return response.data;
  } catch (error) {
    console.error(`Error fetching car with ID ${id}:`, error);
    throw error;
  }
};
