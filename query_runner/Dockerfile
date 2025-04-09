FROM rust:1.83 AS builder

WORKDIR /usr/src/query_runner
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
COPY --from=builder /usr/src/query_runner/target/release/query_runner /app/query_runner

# Command to run the application
CMD ["/app/query_runner"]
