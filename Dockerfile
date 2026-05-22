# Build stage
FROM rust:1-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy all files
COPY . .

# Build the application directly
RUN cargo build --release

# Run stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/payment_service /usr/local/bin/payment_service

# Expose the port
EXPOSE 3000

# Set environment variables
ENV PORT=3000

# Start the application
CMD ["payment_service"]

