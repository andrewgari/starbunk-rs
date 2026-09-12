#!/usr/bin/env bash
# smoke.sh — Build, launch, and probe the starbunk-rs bot system.
# Usage: bash .claude/skills/run-starbunk-rs/smoke.sh [bot]
#   bot: bluebot | bunkbot | covabot | djcova | ratbot | all (default: all)
#
# Required env vars:
#   DISCORD_TOKEN=<real or "fake" for unit-test-only mode>
#   DATABASE_URL=postgres://starbunk:starbunk@localhost/starbunk_memory  (bunkbot only)
#   BUNKBOT_ADMIN_TOKEN=<any string>                                      (bunkbot only)
#
# Outputs:
#   - PASS/FAIL per check, final exit code 0 = all passed

set -euo pipefail

BOT="${1:-all}"
PASS=0
FAIL=0

ok()   { echo "PASS: $*"; ((PASS++)) || true; }
fail() { echo "FAIL: $*"; ((FAIL++)) || true; }

cleanup_pids=()
cleanup() {
  for pid in "${cleanup_pids[@]}"; do
    kill "$pid" 2>/dev/null || true
  done
  wait 2>/dev/null || true
}
trap cleanup EXIT

# ── 1. Build ──────────────────────────────────────────────────────────────────
echo "--- Build ---"
if ! cargo build --workspace; then
  fail "cargo build failed"
  exit 1
fi
ok "cargo build --workspace"

# ── 2. Unit tests ─────────────────────────────────────────────────────────────
echo "--- Unit tests ---"
if cargo test --workspace; then
  ok "cargo test --workspace (all suites)"
else
  fail "cargo test --workspace"
fi

# ── 3. Per-bot health endpoints ───────────────────────────────────────────────
echo "--- Health endpoints ---"

launch_and_probe() {
  local bin="$1"
  local port="$2"
  local need_db="${3:-false}"

  local env_args=()
  env_args+=("DISCORD_TOKEN=${DISCORD_TOKEN:-fake}")
  if [[ "$need_db" == "true" ]]; then
    env_args+=("DATABASE_URL=${DATABASE_URL:-postgres://starbunk:starbunk@localhost/starbunk_memory}")
    env_args+=("BUNKBOT_ADMIN_TOKEN=${BUNKBOT_ADMIN_TOKEN:-testtoken}")
  fi

  env "${env_args[@]}" cargo run --bin "$bin" 2>/dev/null &
  local pid=$!
  cleanup_pids+=("$pid")

  # Poll health endpoint — it comes up fast but crashes on bad Discord auth.
  # With a real token it stays up; with a fake token we grab it before it exits.
  local resp=""
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    sleep 0.2
    resp=$(curl -sf "http://127.0.0.1:${port}/health" 2>/dev/null || echo "")
    [[ -n "$resp" ]] && break
  done
  if [[ "$resp" == '{"status":"ok"}' ]]; then
    ok "$bin health endpoint (port $port)"
  else
    fail "$bin health endpoint (port $port) — got: '$resp'"
  fi

  kill "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
  cleanup_pids=("${cleanup_pids[@]/$pid}")
  sleep 0.3
}

case "$BOT" in
  bluebot) launch_and_probe bluebot 8081 ;;
  bunkbot) launch_and_probe bunkbot 8082 true ;;
  covabot) launch_and_probe covabot 8083 ;;
  djcova)  launch_and_probe djcova  8084 ;;
  ratbot)  launch_and_probe ratbot  8085 true ;;
  all)
    launch_and_probe bluebot 8081
    launch_and_probe bunkbot 8082 true
    launch_and_probe covabot 8083
    launch_and_probe djcova  8084
    launch_and_probe ratbot  8085 true
    ;;
  *) echo "Unknown bot: $BOT"; exit 1 ;;
esac

# ── 4. BunkBot API (only when bunkbot is tested) ──────────────────────────────
if [[ "$BOT" == "bunkbot" || "$BOT" == "all" ]]; then
  echo "--- BunkBot API (port 9082) ---"
  DISCORD_TOKEN="${DISCORD_TOKEN:-fake}" \
  DATABASE_URL="${DATABASE_URL:-postgres://starbunk:starbunk@localhost/starbunk_memory}" \
  BUNKBOT_ADMIN_TOKEN="${BUNKBOT_ADMIN_TOKEN:-testtoken}" \
  cargo run --bin bunkbot 2>/dev/null &
  API_PID=$!
  cleanup_pids+=("$API_PID")
  sleep 1.5

  resp=$(curl -sf http://127.0.0.1:9082/config 2>/dev/null || echo "")
  if echo "$resp" | grep -q "reply-bots\|name:"; then
    ok "GET /config returns yaml"
  else
    fail "GET /config — unexpected: '$resp'"
  fi

  resp=$(curl -sf http://127.0.0.1:9082/api/bots 2>/dev/null || echo "")
  if echo "$resp" | grep -q '"name"'; then
    ok "GET /api/bots returns bot list"
  else
    fail "GET /api/bots — unexpected: '$resp'"
  fi

  resp=$(curl -sf http://127.0.0.1:9082/api/bots/status 2>/dev/null || echo "")
  if echo "$resp" | grep -q '"enabled"'; then
    ok "GET /api/bots/status returns status list"
  else
    fail "GET /api/bots/status — unexpected: '$resp'"
  fi

  kill "$API_PID" 2>/dev/null || true
  wait "$API_PID" 2>/dev/null || true
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo "==============================="
echo "PASS: $PASS  FAIL: $FAIL"
echo "==============================="
[[ "$FAIL" -eq 0 ]]
