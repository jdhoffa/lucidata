# lucidata: Natural Language Data Exploration

Lucidata is a secure, internal AI/LLM based tool designed to democratize data access. It translates natural language questions into SQL/API queries over structured datasets, returning clear, traceable answers and exports.

## Features

- **Natural Language Interface**: Ask questions in plain English
- **Query Translation**: Automatic conversion to SQL/API queries
- **Result Visualization**: Clear charts and tables 
- **Export Options**: Download results in various formats (CSV, Excel, etc.)
- **Query History**: Track and reuse previous queries

## Getting Started

### Prerequisites

- Docker and Docker Compose installed on your machine
- Access credentials for supported datasets

### Usage

1. Clone the repository
   ```bash
   git clone https://github.com/jdhoffa/lucidata.git
   cd lucidata
   ```

2. Start the application with Docker Compose
   ```bash
   docker compose up
   ```

3. Access the web interface at http://localhost:3000

4. Enter your natural language query in the input field and click "Submit"

5. Review the results and use the export options as needed

## Example Queries

```
"What is the projected energy mix in 2030 according to IEA's Net Zero scenario?"

"How does natural gas production in the US compare to China over the next decade in WoodMac's base case?"

"Show me the top 5 countries by renewable energy growth in the next 5 years."
```