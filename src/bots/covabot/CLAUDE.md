# CovaBot — Development Instructions

> See also: `wiki/bots/CovaBot.md`

## Goals & Purpose

AI personality emulator. Generates LLM-driven replies that mimic a specific
user's tone and conversational style. Ported from starbunk-js CovaBot.

## Major Features

- **4-stage decision pipeline:** hard filters → mention check → social battery
  → LLM generation. A message must pass every stage before a reply is sent.
- Personality-driven LLM response generation.
- Conversational context modelling (conversation history as prompt context).
- **Social battery** — CovaBot tracks an engagement score; it becomes less
  responsive as the score drains and more responsive as it recharges.
- Multi-provider LLM fallback chain: Ollama → Gemini → OpenAI.

## Dependencies & Architecture

- `src/bin/covabot.rs` — entry point with serenity client.
- `src/shared/discord/` — `MessageService` sends replies.
- **LLM registry:** configured via `.env` (High/Medium/Low tiers via `src/shared/llm/service.rs`).
- **Memory Service:** `src/shared/memory/` uses pgvector to asynchronously extract facts via Low tier LLM, store embeddings, and perform similarity search for context injection.
- All LLM and Memory DB calls must be fully async (`tokio::spawn`) and timeout-resistant.

## Edge Cases

- Handle all LLM providers failing simultaneously — log with `tracing::error!` and stay silent rather than crash. Priority order: Ollama → Anthropic → Gemini → OpenAI.
- Ignore messages from other bots (`msg.author.bot`) to prevent infinite reply loops.
- Manage LLM rate limits with exponential backoff; do not block the tokio event loop.
- Avoid hallucination bleed-through — truncate or summarise long context threads before sending them as prompt context.
- The social battery state must persist across restarts (store in DB or file); do not reset it every time the bot comes up.
