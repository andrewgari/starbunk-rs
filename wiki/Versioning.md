# Versioning

Versions in starbunk-rs are determined **automatically** on every merge to `main`.
The pipeline reads the conventional commit title, bumps the appropriate part of
the last semver tag, builds all bot images, and deploys — no manual tagging needed.

---

## How it works

Merge a PR to `main` with a conventional commit title:

```
feat(ratbot): add opt-out command
```

The pipeline:
1. Reads the last `v*` tag from git (e.g., `v1.2.3`).
2. Determines the bump type from the commit title.
3. Builds all five bot images → pushes `:v1.3.0`, `:latest`, `:sha-<sha>`.
4. Creates git tag `v1.3.0` and a GitHub Release.
5. `deploy.yml` fires → Tower deploys `:v1.3.0`.

That's it. No `git tag`, no `git push --tags`.

---

## Bump type

| Commit title pattern | Bump | Example |
|---|---|---|
| `feat!:` / `BREAKING CHANGE` in body | major | `v1.3.0` → `v2.0.0` |
| `feat:` | minor | `v1.3.0` → `v1.4.0` |
| anything else (`fix:`, `chore:`, `refactor:`, etc.) | patch | `v1.3.0` → `v1.3.1` |

---

## Tag format

```
v<MAJOR>.<MINOR>.<PATCH>
```

Examples: `v1.2.3`, `v0.3.0`, `v1.0.0`

---

## Docker image tags

| Tag | Meaning |
|---|---|
| `:latest` | Most recent release (what Tower runs by default) |
| `:v<MAJOR>.<MINOR>.<PATCH>` | Specific versioned release — Tower pins to this on deploy |
| `:sha-<short-sha>` | Commit SHA — useful for debugging |

`docker-compose.yml` uses `${IMAGE_TAG:-latest}`. On each deploy, Tower sets
`IMAGE_TAG` to the specific version tag (e.g., `v1.3.0`), so deploys are always
pinned to the exact image that was built and tested.

---

## Finding the current version

```bash
# Latest release tag
git tag -l 'v[0-9]*.[0-9]*.[0-9]*' | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -1

# All release tags
git tag -l 'v[0-9]*.[0-9]*.[0-9]*' | sort -V
```

---

## See Also

- [[development/CI-CD|CI/CD]] — workflow details
- [[infrastructure/Deployment|Deployment]] — how Tower pulls the right image tag
