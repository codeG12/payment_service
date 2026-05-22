# Build stage
FROM rust:slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Switch to nightly to support 1.86+ dependencies
RUN rustup toolchain install nightly && rustup default nightly

# Copy all files
COPY . .

# Build the application
RUN cargo build --release

# Run stage
FROM debian:trixie-slim

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

