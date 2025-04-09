# lucidata: Democratized data access
[![CI](https://github.com/jdhoffa/lucidata/actions/workflows/ci.yml/badge.svg)](https://github.com/jdhoffa/lucidata/actions/workflows/ci.yml)
[![Build and Test System](https://github.com/jdhoffa/lucidata/actions/workflows/build.yml/badge.svg)](https://github.com/jdhoffa/lucidata/actions/workflows/build.yml)

Lucidata is an LLM based query tool designed to democratize data access. It translates natural language questions into SQL/API queries over structured datasets, returning clear, traceable answers and exports.

## Features (WIP)

- Natural Language Interface: Ask questions in plain English
- Query Translation: Automatic conversion to SQL/API queries
- Result Visualization: Clear tables and charts
- Export Options: Download results in various formats (CSV, Excel, etc.)
- Query Transparency: Track and export generated queries

## Getting Started

### Prerequisites

- `docker` installed
- OpenAPI `API_KEY`

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

3. Enter your natural language query in the input field and click "Submit"

4. Review the results and use the export options as needed

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
"What is the projected energy mix in 2030 according to IEA's Net Zero scenario?"

"How does natural gas production in the US compare to China over the next decade in WoodMac's base case?"

"Show me the top 5 countries by renewable energy growth in the next 5 years."
```
