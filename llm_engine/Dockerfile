FROM rust:1.83 AS builder

WORKDIR /usr/src/llm_engine
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl-dev libpq-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/llm_engine/target/release/llm_engine /app/llm_engine

# Set environment variables
ENV LLM_ENGINE_PORT=8001

# Expose the port
EXPOSE 8001

# Command to run the application
CMD ["/app/llm_engine"]
