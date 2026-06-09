# syntax=docker/dockerfile:1
# Multi-stage Rust build — BOT_NAME selects which binary to build and run.
# Usage: docker build --build-arg BOT_NAME=bluebot -t starbunk-rs-bluebot .

ARG BOT_NAME=bluebot

# ── Stage 1: build ──────────────────────────────────────────────────────────
FROM rust:slim AS builder

ARG BOT_NAME

# System deps for sqlx / openssl / native-tls
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependency layer — copy manifests first, then src
COPY Cargo.toml Cargo.lock ./
# Create dummy source so `cargo build` can resolve deps without the real source
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release --bin "${BOT_NAME}" 2>/dev/null || true \
    && rm -rf src

# Now copy the real source and do the real build
COPY src ./src
RUN cargo build --release --bin "${BOT_NAME}"

# ── Stage 2: runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim

ARG BOT_NAME
ENV BOT_NAME=${BOT_NAME}

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/${BOT_NAME} /app/bot

RUN useradd -r -s /bin/false botuser && chown botuser:botuser /app/bot
USER botuser

CMD ["/app/bot"]
