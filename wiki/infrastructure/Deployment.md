# Deployment

Production runs on **Tower** (Unraid, `192.168.50.3`) as a **Portainer stack**
built from source. GCP/GKE is retired.

---

## Current: Tower / Portainer

| Piece | Location |
|---|---|
| Stack definition | `docker/docker-compose.tower.yml` |
| Collector config (Tower) | `observability/otel-collector.tower.yaml` |
| Bot tokens | per-bot env vars — `BLUEBOT_TOKEN`, `BUNKBOT_TOKEN`, `COVABOT_TOKEN`, `DJCOVA_TOKEN`, `RATBOT_TOKEN` |
| Host appdata root | `/mnt/config/appdata/starbunk/` |

Deploy: **Portainer → Stacks → Create from repository** (`git.lan:starbunk-rs`),
or on the host:

```bash
docker compose -f docker/docker-compose.tower.yml up -d --build
```

Images are built locally by the stack; nothing is pulled from a registry.

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

See `.env.example` for the full list. Secrets are injected by 1Password into a
git-ignored `.env`.

### Grafana

Bound to `127.0.0.1:3000` and reachable over the tailnet. Anonymous access is
**read-only** (`GF_AUTH_ANONYMOUS_ORG_ROLE=Viewer`); publishing dashboards
requires the admin password. Dashboards are provisioned from the bind-mounted
`observability/grafana/dashboards` directory.

---

## Retired: GCP pipelines

All GCP workflows now carry a `# DISABLED: GCP pipelines are retired` header and
are **manual-dispatch only with every job gated on the repository variable
`GCP_PIPELINES_ENABLED`**. That variable is unset, so a dispatch is a no-op.

| Workflow | Old trigger (removed) |
|---|---|
| `release.yml` | weekly cron — tagged + released every Sunday |
| `health-check.yml` | daily cron — GKE rollout checks |
| `main.yml` | push to `main` — Artifact Registry publishes |
| `deploy.yml` | `release: published` — GKE deploy |
| `deploy-pr.yml` | `v*.*.*` tag push — GKE test deploy |

`ci.yml` and `e2e-manual.yml` are **not** disabled — `ci.yml` still runs on PRs
(branch protection requires `Validation Success`) and `e2e-manual.yml` is a
manual E2E runner that does not touch GCP.

To re-enable the GCP pipeline, set the `GCP_PIPELINES_ENABLED` repository
variable to `true`.

### One-time Kubernetes cleanup

The `starbunk-ui-tailscale` Deployment was folded into the `starbunk-ui` pod as a
sidecar. `kubectl apply` does not prune removed objects, and the old pod would
keep the `tailscale-proxy-pvc` (ReadWriteOnce) volume mounted, conflicting with
the new sidecar and/or leaving a duplicate Tailscale node. Delete it once:

```bash
kubectl -n starbunk delete deployment starbunk-ui-tailscale --ignore-not-found
```

The `langfuse-secrets` Secret now ships as a manifest with an empty placeholder
so the collector does not land in `CreateContainerConfigError`. Populate the real
value out-of-band (see the comment in `kubernetes/otel-collector.yaml`).

---

## See Also

- [[../development/CI-CD|CI/CD]]
- [[../development/Observability|Observability]]
- `observability/`
