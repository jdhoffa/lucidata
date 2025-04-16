# lucidata: Democratized data access
[![CI](https://github.com/jdhoffa/lucidata/actions/workflows/ci.yml/badge.svg)](https://github.com/jdhoffa/lucidata/actions/workflows/ci.yml)
[![Build and Test System](https://github.com/jdhoffa/lucidata/actions/workflows/build.yml/badge.svg)](https://github.com/jdhoffa/lucidata/actions/workflows/build.yml)

Lucidata is an LLM based query tool designed to democratize data access. It translates natural language questions into SQL/API queries over structured datasets, returning clear, traceable answers and exports.

## Features

- Natural Language Interface: Ask questions in plain English
- Query Translation: Automatic conversion to SQL queries
- Query Transparency: Track and export generated queries, explanations, and model confidence

### Road-Map

- Support for Generic WebAPI queries
- Result Visualization

## Getting Started

### Prerequisites

- `docker` installed
- An OpenAPI `API_KEY`

### Usage

1. Clone the repository
   ```bash
   gh repo clone jdhoffa/lucidata
   cd lucidata
   ```

2. Build and start the application with `docker compose`:
   ```bash
   docker compose build # it can take a while to compile, be patient :-)
   docker compose up
   ```

3. Send your query to the query_router endpoint, and check out the results!
``` bash
curl -X POST "http://localhost:8003/translate-and-execute" \
  -H "Content-Type: application/json" \
  -d '{
    "natural_query": "Show me the cars with the best power-to-weight ratio, sorted from highest to lowest"
  }'
```

4. (Optional) Pipe the output to the `jq` CLI:
``` bash
curl -X POST "http://localhost:8003/translate-and-execute" \
  -H "Content-Type: application/json" \
  -d '{
    "natural_query": "Show me the cars with the best power-to-weight ratio, sorted from highest to lowest"
  }' | jq

# you can also select a specific tag
curl -X POST "http://localhost:8003/translate-and-execute" \
  -H "Content-Type: application/json" \
  -d '{
    "natural_query": "Show me the cars with the best power-to-weight ratio, sorted from highest to lowest"
  }' | jq '.results'
```

## System Architecture

Below is a diagram showing the flow of information and expected user journey:

```mermaid
graph TD
    A[User's Natural Language Input] --> B[Frontend Chat UI]
    B --> |Request| C[LLM Query Engine]
    C --> |Structured Query| D[Query Runner Service]
    D --> |Data Request| E[Data Store]
    E --> |Raw Data| D
    D --> |Processed Data| F[Response Formatter]
    F --> |Formatted Results| B
    B --> |Display Results| A

    style A fill:#f9f,stroke:#333,stroke-width:2px,color:#000
    style B fill:#bbf,stroke:#333,stroke-width:2px,color:#000
    style C fill:#bfb,stroke:#333,stroke-width:2px,color:#000
    style D fill:#fbf,stroke:#333,stroke-width:2px,color:#000
    style E fill:#fbb,stroke:#333,stroke-width:2px,color:#000
    style F fill:#bff,stroke:#333,stroke-width:2px,color:#000

    subgraph "Frontend"
        A
        B[Frontend Chat UI<br>React or Teams plugin]
    end

    subgraph "Backend Services"
        C[LLM Query Engine<br>- Prompt templates<br>- Guardrails<br>- Schema-aware]
        D[Query Runner Service<br>- SQL engine <br>- API connector]
        F[Response Formatter<br>- HTML table<br>- CSV export<br>- Original query<br>- JS widgets/plots]
    end

    subgraph "Data Sources"
        E[Data Store<br>- Emissions Data<br>- Production Data<br>- Climate Scenarios <br>- etc.]
    end
```

## Example Queries

```
# Query #1 tests mathematical operations (division of hp/wt)
"Show me the cars with the best power-to-weight ratio, sorted from highest to lowest."

# Query #2 tests sorting and multi-column selection
"Compare fuel efficiency (MPG) and horsepower for all cars, sorted by MPG."

# Query #3 tests aggregation functions with grouping
"What's the average horsepower and MPG for automatic vs manual transmission cars?"

# Query #4 tests more complex aggregation and grouping
"Show me the relationship between number of cylinders and fuel efficiency with average MPG by cylinder count"

# Query #5 tests limiting results and specific column selection
"Find the top 5 cars with the highest horsepower and their quarter-mile time (qsec)"
```
