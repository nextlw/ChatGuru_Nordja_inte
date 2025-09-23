# Build stage
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config ./config

RUN cargo build --release

# Runtime stage
FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/chatguru-clickup-middleware /usr/local/bin/chatguru-clickup-middleware
COPY --from=builder /app/config /config

EXPOSE 8080

CMD ["chatguru-clickup-middleware"]