import React, { useState } from 'react';
import './App.css';
import QueryInput from './components/QueryInput';
import ResponseDisplay from './components/ResponseDisplay';
import { sendQuery } from './services/api';

function App() {
  const [query, setQuery] = useState('');
  const [response, setResponse] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const handleQuerySubmit = async (queryText) => {
    setQuery(queryText);
    setLoading(true);
    setError(null);
    
    try {
      const result = await sendQuery(queryText);
      setResponse(result);
    } catch (err) {
      setError(err.message || 'An error occurred processing your query');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="App">
      <header className="App-header">
        <h1>Lucidata</h1>
        <p>Query your data using natural language</p>
      </header>
      
      <main className="App-main">
        <QueryInput onSubmit={handleQuerySubmit} />
        
        {loading && <div className="loading">Processing query...</div>}
        {error && <div className="error">{error}</div>}
        {response && <ResponseDisplay data={response} originalQuery={query} />}
      </main>
    </div>
  );
}

export default App;
