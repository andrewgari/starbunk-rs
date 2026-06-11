# Deployment

## CI/CD Pipeline

| Workflow | Trigger | What it does |
|---|---|---|
| `ci.yml` | PRs to `main` | Validate DevOps consistency, lint (clippy + fmt), test, build binaries |
| `main.yml` | Merge to `main` | Validate, test, build + push Docker images to GHCR, tag `:latest` |
| `deploy.yml` | After `main.yml` succeeds | Tailscale SSH to Tower, pull images, restart services, health check |

## Docker Images

Images are published to GHCR:

```
ghcr.io/andrewgari/starbunk-<bot>:<tag>
```

Tags: `latest` (always points to last successful main merge), plus a versioned
tag created by `main.yml` (e.g., `v1.2.3`).

## Tower Server

Production runs on a server called Tower. `deploy.yml` connects via Tailscale
SSH and uses `scripts/deployment/deploy.sh` to:
1. Stage the production `docker-compose.yml`.
2. Pull new images (`docker compose pull`).
3. Restart services (`docker compose up -d`).
4. Run `scripts/deployment/health-check.sh` to verify all bots are healthy.

The production compose file uses `stack.env` on the server for bot tokens.

## Health Check

`scripts/deployment/health-check.sh` verifies every bot in `EXPECTED_SERVICES`
is running. This is also called at the end of every deploy.

## See Also

- [[../development/CI-CD|CI/CD]]
- `scripts/deployment/`
