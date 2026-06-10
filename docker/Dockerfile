# syntax=docker/dockerfile:1
# Multi-stage Rust workspace build — BOT_NAME selects which binary to build and run.
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

# Copy workspace manifest and lock file first so Cargo can resolve the full dep graph.
COPY Cargo.toml Cargo.lock ./

# Copy all per-crate manifests for dependency-layer caching.
COPY crates/starbunk-shared/Cargo.toml crates/starbunk-shared/
COPY crates/bluebot/Cargo.toml         crates/bluebot/
COPY crates/bunkbot/Cargo.toml         crates/bunkbot/
COPY crates/covabot/Cargo.toml         crates/covabot/
COPY crates/djcova/Cargo.toml          crates/djcova/
COPY crates/ratbot/Cargo.toml          crates/ratbot/

# Create minimal stub sources so `cargo build` can compile all external deps
# without the real implementation.  Stubs for every workspace member are
# required because Cargo resolves the full dependency graph up-front.
RUN mkdir -p crates/starbunk-shared/src \
    && echo "pub fn _stub() {}" > crates/starbunk-shared/src/lib.rs \
    && for bot in bluebot bunkbot covabot djcova ratbot; do \
         mkdir -p crates/$bot/src; \
         echo "pub fn _stub() {}" > crates/$bot/src/lib.rs; \
         echo "fn main() {}" > crates/$bot/src/main.rs; \
       done \
    && cargo build --release -p "${BOT_NAME}" 2>/dev/null || true \
    && rm -rf crates/*/src

# Copy real sources and rebuild (Cargo detects content changes and recompiles
# only the workspace crates; external dep artifacts are reused from the cache).
COPY crates ./crates
RUN cargo build --release -p "${BOT_NAME}"

# ── Stage 2: runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim

ARG BOT_NAME
ENV BOT_NAME=${BOT_NAME}

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && if [ "${BOT_NAME}" = "djcova" ]; then \
         apt-get install -y --no-install-recommends ffmpeg python3 curl \
         && curl -L https://github.com/yt-dlp/yt-dlp/releases/download/2026.06.09/yt-dlp -o /usr/local/bin/yt-dlp \
         && echo "e5d57466682cfa9d61e9cf7c8a4f09b00f4a62af37d3bbdc4bcffdf63615feac  /usr/local/bin/yt-dlp" | sha256sum -c - \
         && chmod a+rx /usr/local/bin/yt-dlp \
         && apt-get purge -y --auto-remove curl; \
       fi \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/${BOT_NAME} /app/bot

RUN useradd -r -s /bin/false botuser && chown botuser:botuser /app/bot
USER botuser

CMD ["/app/bot"]
