# Start with image that has the Rust toolchain installed
FROM rust:1.85-alpine AS chef
USER root
# Add cargo-chef to cache dependencies
RUN apk add --no-cache musl-dev build-base && cargo install cargo-chef --locked
WORKDIR /app

# ------------ 1. planner
FROM chef AS planner
COPY . .
# Capture info needed to build dependencies
RUN cargo chef prepare --recipe-path recipe.json

# ------------ 2. builder-prepare
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
COPY db-test-macro db-test-macro
RUN cargo chef cook --release --recipe-path recipe.json

# ------------ 3. builder-build
# Build application
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin auth-service

# We do not need the Rust toolchain to run the binary!
# Start with a minimal image and copy over the binary and assets folder.
FROM debian:buster-slim AS runtime
WORKDIR /app
# Install curl for health checks
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/auth-service /usr/local/bin
COPY --from=builder /app/assets /app/assets
ENV REDIS_HOST_NAME=redis
ENTRYPOINT ["/usr/local/bin/auth-service"]
