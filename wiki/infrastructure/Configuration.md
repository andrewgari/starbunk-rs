# Configuration

## Environment Variables

| Variable | Purpose |
|---|---|
| `DISCORD_TOKEN` | Token used by `run_bot` at runtime |
| `STARBUNK_TOKEN` | Fallback token used by all bots in Docker Compose |
| `{BOTNAME}_TOKEN` | Per-bot override (e.g. `BUNKBOT_TOKEN`, `COVABOT_TOKEN`) |
| `RUST_LOG` | Log level for `tracing-subscriber` (e.g. `info`, `debug`, `starbunk=debug`) |
| `OPENAI_API_KEY` | API key for OpenAI |
| `OPENAI_BASE_URL` | Custom base URL for OpenAI-compatible endpoints |
| `ANTHROPIC_API_KEY` | API key for Anthropic |
| `ANTHROPIC_BASE_URL` | Custom base URL for Anthropic API |
| `GOOGLE_API_KEY` | API key for Google Gemini |
| `GOOGLE_BASE_URL` | Custom base URL for Google API |
| `OLLAMA_BASE_URL` | Base URL for local Ollama instance (default: `http://localhost:11434`) |
| `LLM_TIER_HIGH_PROVIDER` | Provider for high-capability tier (e.g. `anthropic`) |
| `LLM_TIER_HIGH_MODEL` | Model for high-capability tier (e.g. `claude-3-5-sonnet-latest`) |
| `LLM_TIER_MEDIUM_PROVIDER` | Provider for medium-capability tier (e.g. `google`) |
| `LLM_TIER_MEDIUM_MODEL` | Model for medium tier (e.g. `gemini-1.5-flash`) |
| `LLM_TIER_LOW_PROVIDER` | Provider for low-capability tier (e.g. `openai`) |
| `LLM_TIER_LOW_MODEL` | Model for low tier (e.g. `text-embedding-3-small`) |
| `DATABASE_URL` | PostgreSQL connection string for CovaBot memory |

Each Docker Compose service resolves its token as:
```
${BOTNAME_TOKEN:-${STARBUNK_TOKEN}}
```

## BunkBot Configuration (bots.yml)

BunkBot reads its reply bot routing and triggers configuration from a `bots.yml` file:

- **Local Development**: Looks for `config/bots.yml` relative to the workspace root. This path is gitignored to avoid leaking custom reply personas to GitHub.
- **Production (GKE)**: Projected from the `starbunk-secrets` Kubernetes Secret (under the key `BOTS_CONFIG_YAML`) and mounted into the pod at `/app/config/bots.yml`.
  - To update this configuration in production, edit your local `config/bots.yml`, run `kube_secrets.sh` to update the GKE secret, and trigger a deployment rollout.

## Docker Compose Files

| File | Purpose |
|---|---|
| `docker-compose.yml` | **Production** — pulls pre-built GHCR images. Deployed to Tower by `deploy.yml`. Requires `stack.env` on the server. |
| `docker/docker-compose.yml` | **Local dev** — builds from source using `docker/Dockerfile` with `BOT_NAME` build arg. |

## Local Dev Setup

```bash
cp .env.example .env   # fill in STARBUNK_TOKEN at minimum
docker compose -f docker/docker-compose.yml up -d --build
```

## See Also

- [[../development/Getting-Started|Getting Started]]
