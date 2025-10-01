# Use Rust 1.90 which supports edition2024 (required by google-cloud-gax-1.0.0)
FROM rust:1.90-slim as builder

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

# Copy source code and config
COPY src ./src
COPY config ./config

# Build normally (Rust 1.90 debian has compatible GLIBC)
RUN cargo build --release

# Runtime stage - usar mesma base Debian que o builder para compatibilidade GLIBC
FROM rust:1.90-slim

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
# PORT é automaticamente definido pelo Cloud Run (não definir aqui)

# Health check removed for Cloud Run (uses its own health checking)

# Run the application
CMD ["./chatguru-clickup-middleware"]