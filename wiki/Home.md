# Starbunk-Rs Wiki

> **Second Brain for the Starbunk-Rs Discord Bot System**
> Last Updated: 2026-06-09

---

## What Is Starbunk-Rs?

Starbunk-Rs is a Rust monorepo containing 5 independent Discord bots, each with
its own binary and Docker container. It is a port of
[starbunk-go](https://github.com/andrewgari/starbunk-go) to Rust, sharing the
same bot personalities but using async/await with Tokio and Serenity for a
memory-safe, high-performance implementation.

---

## Navigation

### Bots
- [[bots/BlueBot|BlueBot]] — Pattern-matching bot for "blue" / Blue Mage references
- [[bots/BunkBot|BunkBot]] — Administrative backbone and general reply bot
- [[bots/CovaBot|CovaBot]] — AI personality emulator (LLM-driven)
- [[bots/DJCova|DJCova]] — Voice channel music streaming (YouTube)
- [[bots/RatBot|RatBot]] — Rat-themed SecretRat bot

### Infrastructure & Deployment
- [[infrastructure/Architecture|Architecture]] — Monorepo layout, bot pattern, shared libraries
- [[infrastructure/Deployment|Deployment]] — CI/CD pipeline, Docker images, Tower server
- [[infrastructure/Configuration|Configuration]] — Environment variables and Docker Compose files
- [[infrastructure/UI|Bot Management UI]] — Next.js control plane for bot lifecycle and settings

### Development
- [[development/Getting-Started|Getting Started]] — Local dev setup
- [[development/Testing|Testing]] — Rust test guide (`#[test]`, `tokio::test`)
- [[development/TDD|TDD SDLC Workflow]] — Mandatory test-first development process
- [[development/CI-CD|CI/CD]] — GitHub Actions workflows
- [[development/MessageFiltering|Message Filtering]] — Composable message filter abstraction

### AI Agents
- [[agents/Agents|Custom Agents]] — Claude Code subagents: rust-craftsman, architect, pm, devops

### History
- [[Changelog|Changelog]] — Running log of all work done on this project

---

## Quick Reference

| Bot | Image | Binary |
|-----|-------|--------|
| BlueBot | `ghcr.io/andrewgari/starbunk-bluebot` | `crates/bluebot/src/main.rs` |
| BunkBot | `ghcr.io/andrewgari/starbunk-bunkbot` | `crates/bunkbot/src/main.rs` |
| CovaBot | `ghcr.io/andrewgari/starbunk-covabot` | `crates/covabot/src/main.rs` |
| DJCova  | `ghcr.io/andrewgari/starbunk-djcova`  | `crates/djcova/src/main.rs`  |
| RatBot  | `ghcr.io/andrewgari/starbunk-ratbot`  | `crates/ratbot/src/main.rs`  |

### Key Commands

```bash
cargo test                                                  # run all tests
bash scripts/devops-validate.sh                             # validate DevOps file consistency
docker compose -f docker/docker-compose.yml up -d --build  # local dev
```

---

## Agent Instructions

> These instructions apply to **all AI agents** working in this repository.
> They are also codified in `AGENTS.md` at the repo root.

1. **Before starting any task** — read the relevant wiki page(s) for the area you will touch.
2. **After completing any task** — update the relevant wiki page(s) with any changes to architecture, behavior, config, or patterns.
3. **For every significant change or PR** — add an entry to [[Changelog]] under today's date.
4. If a wiki page does not exist for the area you are working in, create it.
5. Use Obsidian-style `[[Page]]` or `[[folder/Page|Display Name]]` links between pages.
