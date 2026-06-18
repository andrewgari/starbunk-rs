# CovaBot

> AI personality emulator with LLM-driven responses.

## Goals & Purpose

CovaBot impersonates a specific user's tone and personality in Discord, using an
LLM to generate contextually-aware replies. It uses Ollama (primary), Anthropic,
Gemini, and OpenAI as fallbacks.

## Major Features

- Personality-driven LLM response generation.
- Conversational context modelling and active conversation tracking.
- Context-aware tagging capable of combining generic and specific tags and reducing duplication.
- Multi-provider LLM support (Ollama → Anthropic → Gemini → OpenAI fallback chain).

## Dependencies & Architecture

- **Entry point:** `src/bin/covabot.rs` → `src/bots/covabot::run()`
- **Framework:** `starbunk::run_bot` + `src/shared/discord::MessageService`
- **Personality Config:** Loads `config/bots/covabot.yml` which defines behavior (`social_battery_config`, `topic_affinities`, etc.) via `personality::Profile`.
- **Engagement Manager:** Controls whether CovaBot responds. It uses `GateReason` checks (Direct Mention, Direct Reply) and evaluates `topic_affinities` alongside the `social_battery`. If the battery is depleted (<= 20%), ambient engagement is dampened.
- **LLM:** `src/shared/llm::Registry` provides High/Medium/Low tier routing.
- **Memory:** `src/shared/memory::MemoryService` handles async pgvector-based fact extraction (Low tier) and similarity search for context injection.
- All LLM calls must be fully async and timeout-resistant.

## Edge Cases

- All LLM providers failing simultaneously.
- Rate limits and hallucination management.
- Infinite loops when interacting with other bots (must ignore bot authors).
- Parsing extremely long conversation threads efficiently.

## See Also

- [[../infrastructure/Configuration|Configuration]] for LLM env vars
- [[../infrastructure/Architecture|Architecture]]
