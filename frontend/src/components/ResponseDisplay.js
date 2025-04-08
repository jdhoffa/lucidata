import React from 'react';

function ResponseDisplay({ data, originalQuery }) {
  if (!data || !data.result) {
    return null;
  }

  // Helper function to determine if the result is tabular
  const isTabularData = () => {
    return Array.isArray(data.result) && data.result.length > 0;
  };

  // Extract table headers if it's tabular data
  const getTableHeaders = () => {
    if (!isTabularData() || !data.result[0]) {
      return [];
    }
    return Object.keys(data.result[0]);
  };

  // Generate CSV content from the result
  const generateCsvContent = () => {
    if (!isTabularData()) {
      return '';
    }

    const headers = getTableHeaders();
    const headerRow = headers.join(',');
    const rows = data.result.map(item => 
      headers.map(header => {
        const value = item[header];
        // Handle values that might need escaping in CSV
        if (value === null || value === undefined) return '';
        if (typeof value === 'string' && (value.includes(',') || value.includes('"') || value.includes('\n')))
          return `"${value.replace(/"/g, '""')}"`;
        return String(value);
      }).join(',')
    );
    
    return [headerRow, ...rows].join('\n');
  };

  // Handle CSV download
  const handleDownloadCsv = () => {
    const csvContent = generateCsvContent();
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.setAttribute('href', url);
    link.setAttribute('download', 'query-result.csv');
    link.style.display = 'none';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  return (
    <div className="response-display">
      <div className="response-header">
        <h3>Results for: "{originalQuery}"</h3>
        {isTabularData() && (
          <button onClick={handleDownloadCsv} className="download-csv">
            Download CSV
          </button>
        )}
      </div>

      <div className="query-transparency">
        <h4>Executed Query:</h4>
        <pre>{data.executed_query}</pre>
      </div>

      <div className="result-content">
        {isTabularData() ? (
          <div className="table-container">
            <table className="result-table">
              <thead>
                <tr>
                  {getTableHeaders().map((header, index) => (
                    <th key={index}>{header}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {data.result.map((row, rowIndex) => (
                  <tr key={rowIndex}>
                    {getTableHeaders().map((header, colIndex) => (
                      <td key={colIndex}>{row[header] !== null ? String(row[header]) : ''}</td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <pre>{JSON.stringify(data.result, null, 2)}</pre>
        )}
      </div>
    </div>
  );
}

export default ResponseDisplay;
