---
description: Build Starbunk bot binaries. Specify a bot name or build all.
argument-hint: [bot-name|all|release]
allowed-tools: [Bash]
---

# Build

Build Starbunk bot binaries.

## Arguments

The user invoked this with: $ARGUMENTS

- No argument or `all`: build all bots in debug mode
- A specific bot name (`bluebot`, `bunkbot`, `covabot`, `djcova`, `ratbot`): build that bot only
- `release`: build all bots in release mode
- `check`: run `cargo check` without producing binaries (fast)

## Instructions

Build based on the argument provided:

1. If `check` is provided, fast-check all packages compile:
   ```bash
   cargo check --all
   ```

2. If `release` is provided, build all bots in release mode:
   ```bash
   cargo build --bins --release
   ```

3. If a specific bot is provided (bluebot, bunkbot, covabot, djcova, ratbot), build that bot:
   ```bash
   cargo build --bin <bot>
   ```

4. If no argument or `all` is provided, build everything in debug mode:
   ```bash
   cargo build --bins
   ```

Report any compilation errors and suggest fixes.
