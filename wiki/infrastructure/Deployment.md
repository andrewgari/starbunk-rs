# Deployment

Production runs on **Tower** (Unraid, `192.168.50.3`) as a **Docker Compose
stack managed by Portainer**, built from source. Docker Compose is the only
deployment path — there are no cluster manifests, registry pulls, or cloud
deploy workflows in this repo.

---

## Compose files

| File | Use |
|---|---|
| `docker/docker-compose.tower.yml` | **Production** on Tower — absolute appdata paths, per-bot tokens, required secrets |
| `docker/docker-compose.yml` | **Local dev** — same services, repo-relative paths |
| `docker/docker-compose.e2e.yml` | E2E test runner |

All three build the bots from `docker/Dockerfile` with a `BOT_NAME` build arg.

## Tower stack

| Piece | Location |
|---|---|
| Stack definition | `docker/docker-compose.tower.yml` |
| Collector config | `observability/otel-collector.tower.yaml` (mounted from `/mnt/config/appdata/starbunk/observability/`) |
| Host appdata root | `/mnt/config/appdata/starbunk/` |
| Bot config (read-only bind mounts) | `/mnt/config/appdata/starbunk/config/{bunkbot,covabot}` |
| Bot tokens | `BLUEBOT_TOKEN`, `BUNKBOT_TOKEN`, `COVABOT_TOKEN`, `DJCOVA_TOKEN`, `RATBOT_TOKEN` |

Services: `otel-collector`, `loki`, `tempo`, `prometheus`, `grafana`,
`postgres` (pgvector), and the five bots (`bluebot`, `bunkbot`, `covabot`,
`djcova`, `ratbot`). Every service uses `restart: unless-stopped`; bots wait on
`otel-collector` (and `postgres` where they need it) being healthy.

### Deploy / update

Portainer → **Stacks → Create from repository** (`git.lan:starbunk-rs`, compose
path `docker/docker-compose.tower.yml`). To roll out new code, use **Pull and
redeploy** on the stack. Equivalent on the host:

```bash
git pull
docker compose -f docker/docker-compose.tower.yml up -d --build
```

Images are built locally by the stack; nothing is pulled from a registry for the
bots.

### Restart / stop / logs

```bash
docker compose -f docker/docker-compose.tower.yml restart bunkbot
docker compose -f docker/docker-compose.tower.yml stop djcova
docker compose -f docker/docker-compose.tower.yml logs -f covabot
```

Or use the stack's container list in Portainer. `starbunk-ui` is read-only for
bot lifecycle: it shows status from each bot's HTTP API and never controls
containers (no Docker socket). See [[UI]].

### Updating bot config

`bots.yml` and CovaBot config live under
`/mnt/config/appdata/starbunk/config/<bot>/` on Tower and are mounted `:ro`.
Edit the file on the host — BunkBot hot-reloads on change. See
[[Configuration]].

### Discord tokens

Discord bot tokens are 1:1 with application identity, so **each bot needs its own
token**. There is no `STARBUNK_TOKEN` fallback in the Tower stack — reusing one
token for several bots causes identify conflicts and random disconnects.

### Required environment

The stack fails fast (`${VAR:?…}`) rather than starting with a known default
password:

| Variable | Notes |
|---|---|
| `POSTGRES_PASSWORD` | required — no default |
| `GRAFANA_ADMIN_PASSWORD` | required — no default |
| `LANGFUSE_AUTH_STRING` | `base64(pk-lf-…:sk-lf-…)`; leave empty to disable export |
| `BLUEBOT_TOKEN` … `RATBOT_TOKEN` | one per bot |
| `OLLAMA_BASE_URL` | defaults to the Framework desktop (`http://192.168.50.2:11434`) |

See `.env.example` for the full list. Secrets are injected by 1Password into a
git-ignored `.env` (or set as stack environment variables in Portainer).

### Grafana

Bound to `127.0.0.1:3000` and reachable over the tailnet. Anonymous access is
**read-only** (`GF_AUTH_ANONYMOUS_ORG_ROLE=Viewer`); publishing dashboards
requires the admin password. Dashboards are provisioned from the bind-mounted
`observability/grafana/dashboards` directory.

### Health

Each bot logs `health: startup ok` once its Discord gateway is ready (see
`crates/starbunk/src/health.rs`). BunkBot additionally serves `GET /live` and
`GET /health` on port `9082`.

---

## CI workflows

`ci.yml` runs on every PR (branch protection requires `Validation Success`);
`e2e-manual.yml` is a manual E2E runner. `main.yml` and `release.yml` are
retired GCP Artifact Registry pipelines — manual-dispatch only and gated on the
unset repository variable `GCP_PIPELINES_ENABLED`. No workflow deploys to Tower;
deploys are pulled by Portainer. See [[../development/CI-CD|CI/CD]].

---

## See Also

- [[../development/CI-CD|CI/CD]]
- [[../development/Observability|Observability]]
- [[Configuration]]
- `observability/`
