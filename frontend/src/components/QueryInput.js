import React, { useState } from 'react';

function QueryInput({ onSubmit }) {
  const [query, setQuery] = useState('');

  const handleSubmit = (e) => {
    e.preventDefault();
    if (query.trim()) {
      onSubmit(query);
    }
  };

  return (
    <div className="query-input">
      <form onSubmit={handleSubmit}>
        <textarea
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Enter your query in natural language (e.g., 'Show me cars with more than 6 cylinders')"
          rows={4}
          className="query-textarea"
        />
        <button type="submit" disabled={!query.trim()} className="submit-button">
          Ask
        </button>
      </form>
    </div>
  );
}

export default QueryInput;
