# Use Rust slim image for building
# Updated to 1.82 to support newer dependencies (icu_collections, idna_adapter, etc.)
FROM rust:1.82-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code and config
COPY src ./src
COPY config ./config

# Build the actual application - force rebuild
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -u 1000 app

# Create app directory and set ownership
WORKDIR /app
RUN chown app:app /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/chatguru-clickup-middleware .
COPY --from=builder /app/config ./config

# Change to app user
USER app

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV PORT=8080

# Health check removed for Cloud Run (uses its own health checking)

# Run the application
CMD ["./chatguru-clickup-middleware"]