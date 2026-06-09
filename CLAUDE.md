# CLAUDE.md

Agent guide for Claude Code. All rules, architecture notes, and the DevOps
maintenance checklist live in [AGENTS.md](AGENTS.md) and are imported below.

## Claude Code: Proactive Skill Use

Use available skills **without being told**. When the situation matches, invoke
the skill immediately — don't describe what you'd do, just do it.

| Situation | Skill / Rule |
|---|---|
| Any coding, fixing, or refactoring task | `/task` |
| Developing features, bugfixes, or ports | **Mandatory TDD**: Write Rust tests first (Test-Only PR 1), then implement (PR 2) |
| Code has been written or changed | `/simplify` — review for quality and reuse |
| File exceeds ~150 lines | Split by responsibility — one concept per file |
| New bot or shared module added | Verify isolation: `bots/` never imports from another bot; `shared/` never imports from `bots/` |
| Deploying or updating containers on Tower | `/deploy` |
| PR is open — review comments to address | run `cargo test` locally, then address each comment |
| User asks about Claude Code / Anthropic API | `claude-code-guide` agent |
| Setting up hooks or automated behaviors | `/update-config` |

Before declaring any task done, follow the TDD SDLC workflow and run `cargo test` locally. If tests fail,
fixing them is part of the task.

Rust quality checklist — apply to every file touched:
- Dependencies injected as `Arc<dyn Trait>`, not concrete types
- `#[derive(Debug)]` on every public struct and enum
- Regexes compiled once in `LazyLock<Regex>` statics
- No `.unwrap()` in production code (`.expect("reason")` on programmer-error panics only)
- Slow async work spawned with `tokio::spawn`
- See **Rust Code Standards** in `AGENTS.md` for the full ruleset
- See **Git Commit Standards** in `AGENTS.md` for conventional commit enforcement

@AGENTS.md
