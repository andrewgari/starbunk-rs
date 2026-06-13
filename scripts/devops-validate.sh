#!/bin/bash
# Validates that all DevOps files are consistent with the bots defined in crates/.
#
# Run this script any time you add, remove, or rename a bot, or after editing
# any CI/CD or Docker file. It is also executed as a CI check on every PR.
#
# Usage: bash scripts/devops-validate.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

ERRORS=0

fail() { echo "  FAIL  $1"; ERRORS=$((ERRORS + 1)); }
ok()   { echo "  ok    $1"; }

# ── Discover bots from crates/ ────────────────────────────────────────────────
# A bot crate is any directory under crates/ that has a src/main.rs and is
# not the shared library.
BOTS=()
for dir in crates/*/; do
  bot=$(basename "$dir")
  if [ "$bot" == "starbunk" ] || [ "$bot" == "e2e" ]; then
    continue
  fi
  if [ -f "crates/${bot}/src/main.rs" ]; then
    BOTS+=("$bot")
  fi
done

if [ ${#BOTS[@]} -eq 0 ]; then
  echo "ERROR: No bots found under crates/. Is this the repo root?"
  exit 1
fi

echo "Bots discovered in crates/: ${BOTS[*]}"
echo ""

# ── Check each file for every bot ────────────────────────────────────────────
for bot in "${BOTS[@]}"; do
  echo "[$bot]"

  # 1. docker/docker-compose.yml (local dev — build from source)
  if grep -q "BOT_NAME: ${bot}" docker/docker-compose.yml 2>/dev/null; then
    ok "docker/docker-compose.yml: BOT_NAME=${bot}"
  else
    fail "docker/docker-compose.yml: missing BOT_NAME: ${bot}"
  fi

  # 2. .github/workflows/ci.yml — path filter for crate directory
  if grep -q "crates/${bot}/" .github/workflows/ci.yml 2>/dev/null; then
    ok ".github/workflows/ci.yml: filter includes crates/${bot}/"
  else
    fail ".github/workflows/ci.yml: missing 'crates/${bot}/' in path filter"
  fi

  # 3. .github/workflows/main.yml — docker build matrix
  if grep -q "${bot}" .github/workflows/main.yml 2>/dev/null; then
    ok ".github/workflows/main.yml: docker matrix includes ${bot}"
  else
    fail ".github/workflows/main.yml: missing '${bot}' in docker build matrix"
  fi

  echo ""
done

echo ""

# ── Result ────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $ERRORS -gt 0 ]; then
  echo "FAILED — $ERRORS consistency error(s). Fix the files listed above before"
  echo "         committing. See AGENTS.md § 'DevOps File Maintenance' for the"
  echo "         full list of files that must stay in sync."
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  exit 1
else
  echo "PASSED — all DevOps files are consistent."
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
fi
