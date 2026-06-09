# Getting Started

## Prerequisites

- Rust (stable) via [rustup](https://rustup.rs/)
- Docker + Docker Compose
- A Discord bot token (set `DISCORD_TOKEN` or `STARBUNK_TOKEN`)

## Run a single bot locally

```bash
DISCORD_TOKEN=<token> cargo run --bin bunkbot
```

## Run all bots via Docker (local dev)

```bash
docker compose -f docker/docker-compose.yml up -d --build
```

## Build all binaries

```bash
cargo build --bins
```

## Run tests

```bash
cargo test
```

## Run clippy (lint)

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## Check formatting

```bash
cargo fmt --all -- --check
```

## Validate DevOps consistency

Run this after any bot or CI/CD change:

```bash
bash scripts/devops-validate.sh
```

## See Also

- [[../infrastructure/Configuration|Configuration]] — environment variables
- [[Testing|Testing]] — test guide
- [[CI-CD|CI/CD]] — pipeline overview
