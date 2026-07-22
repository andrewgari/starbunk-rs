---
description: Add a new bot crate to the starbunk-rs workspace with full DevOps registration
argument-hint: <bot-name>
allowed-tools: [Bash, Read, Write, Edit, MultiEdit, Glob, Grep]
---

# Add a New Bot

Complete checklist to add a new bot to the workspace, covering all required DevOps registrations and wiki documentation.

## Arguments

The user invoked this with: $ARGUMENTS

Parse the bot name from arguments. If not provided, ask: "What is the new bot's name? (e.g. `mybot`)"

## Step 1 — Create the crate

```bash
mkdir -p crates/<botname>/src
```

Create `crates/<botname>/Cargo.toml` (lib + bin crate, depends on `starbunk-shared`).
Create `crates/<botname>/src/lib.rs` with Handler + EventHandler + `pub async fn run()`.
Create `crates/<botname>/src/main.rs` calling `<botname>::run().await`.

Add `"crates/<botname>"` to the `members` list in the root `Cargo.toml`.

## Step 2 — Register in all 6 DevOps files

**`docker-compose.yml`** — add a service block:
```yaml
<botname>:
  image: ghcr.io/andrewgari/starbunk-rs-<botname>:${IMAGE_TAG:-latest}
  container_name: starbunk-rs-<botname>
  restart: unless-stopped
  environment:
    - DISCORD_TOKEN=${NEWBOT_TOKEN:-${STARBUNK_TOKEN}}
    - RUST_LOG=${RUST_LOG:-info}
  logging:
    driver: "json-file"
    options:
      max-size: "10m"
      max-file: "3"
  labels:
    - "com.centurylinklabs.watchtower.enable=true"
```

**`docker/docker-compose.yml`** — add a service block:
```yaml
<botname>:
  build:
    context: ..
    dockerfile: docker/Dockerfile
    args:
      BOT_NAME: <botname>
  container_name: starbunk-rs-<botname>
  restart: unless-stopped
  environment:
    - DISCORD_TOKEN=${NEWBOT_TOKEN:-${STARBUNK_TOKEN}}
    - RUST_LOG=${RUST_LOG:-info}
  logging:
    driver: "json-file"
    options:
      max-size: "10m"
      max-file: "3"
```

**`.github/workflows/ci.yml`** — add path filter `crates/<botname>/**`.

**`.github/workflows/main.yml`** — add `<botname>` to the docker build matrix.

**`scripts/deployment/health-check.sh`** — add `"<botname>"` to `EXPECTED_SERVICES`.

**`AGENTS.md`** — update the bot list in the Architecture and Bots sections.

## Step 3 — Validate DevOps consistency

```bash
bash scripts/devops-validate.sh
```

Fix every `FAIL` line before continuing.

## Step 4 — Wiki and docs

- Create `wiki/bots/<BotName>.md` documenting the bot's purpose, triggers, and behavior.
- Update `wiki/Home.md` to reference the new bot.
- Add an entry to `wiki/Changelog.md`.

## Rules

- All 6 DevOps files must be updated — CI will fail if any are missing.
- Run `devops-validate.sh` after each change to catch drift early.
- Never skip the wiki docs step.
