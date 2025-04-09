FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libpq-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy Cargo.toml first to leverage Docker cache
COPY Cargo.toml ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy the actual source code
COPY src src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libssl-dev libpq-dev ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*


# Copy the binary from the builder stage
COPY --from=builder /app/target/release/llm_engine /app/llm_engine

# Set environment variables
ENV LLM_ENGINE_PORT=8001

# Expose the port
EXPOSE 8001

# Command to run the application
CMD ["/app/llm_engine"]
