# Custom AI Agents (Claude & Gemini)

> Last Updated: 2026-06-09

Custom subagents/skills for this repo live in `.claude/agents/` for Claude Code.
They are loaded automatically by Claude and invoked based on their `description` field.

---

## Overview

Four agents cover the main concerns of working in this repo. They are designed to complement each other — `pm` clarifies intent, `architect` plans, `rust-craftsman` implements, and `devops` ships.

| Agent | File | Purpose |
|-------|------|---------|
| `pm` | `.claude/agents/pm.md` | Requirements gathering, clarification, scope alignment |
| `architect` | `.claude/agents/architect.md` | Architecture planning, cross-cutting review, directing other agents |
| `rust-craftsman` | `.claude/agents/rust-craftsman.md` | Rust code writing, idiomatic patterns, naming, aesthetics |
| `devops` | `.claude/agents/devops.md` | GitHub, CI/CD, Docker Compose, deployment |

---

## When each agent is used

### `pm` — Product Manager
Use when a request is ambiguous or when scope needs to be defined before building starts. This agent asks focused questions, points out edge cases and unconsidered impacts, and produces a clear summary of what will and won't be built.

**Triggers:** vague requests, new features, anything where "what exactly do we want?" isn't obvious.

### `architect` — Architecture Lead
Use before significant implementation begins — when a change touches multiple bots, shared `src/shared/` libraries, or cross-cutting infrastructure. The architect maps the impact surface, plans the work, and hands off specific direction to other agents.

**Triggers:** changes to `src/shared/`, new bots, significant refactors, anything touching more than one bot.

### `rust-craftsman` — Rust Craftsman
Use for any actual Rust code writing, editing, or review. This agent prioritizes readability, idiomatic Rust patterns (ownership, `Result`/`Option`, traits, async), and thoughtful naming. It reads before writing, matches existing code style, and runs tests before declaring work done.

**Triggers:** implementing features, fixing bugs, refactoring Rust code, reviewing code quality.

### `devops` — DevOps Engineer
Use for anything touching the pipeline: GitHub workflows, Docker Compose files, health checks, PRs, and Tower deployment. This agent knows the files that must stay in sync and always runs `devops-validate.sh` after relevant changes.

**Triggers:** CI/CD changes, Docker changes, adding/removing bots, opening PRs, deployment.

---

## Typical flow for a new feature

1. **`pm`** — clarify what's being built and why; agree on scope
2. **`architect`** — map the impact, identify risks, produce a plan
3. **`rust-craftsman`** — implement the code
4. **`devops`** — open PR, verify CI passes, deploy

Not every task needs all four. A small bug fix might just need `rust-craftsman`. A vague idea needs `pm` first.
