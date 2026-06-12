#!/bin/bash
set -euo pipefail

# Starbunk-Rs Health Check Script
# Verifies all bot containers are running after deployment.
# Called by the GitHub Actions deploy workflow via SSH.

COMPOSE_DIR="${1:-/mnt/user/appdata/starbunk-rs}"
RETRY_COUNT=3
RETRY_DELAY=10

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Starbunk-Rs Health Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Compose Directory: ${COMPOSE_DIR}"
echo "Time: $(date)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cd "$COMPOSE_DIR"

if command -v docker-compose &> /dev/null; then
  COMPOSE_CMD="docker-compose --env-file stack.env"
elif docker compose version &> /dev/null; then
  COMPOSE_CMD="docker compose --env-file stack.env"
else
  echo "ERROR: Neither docker-compose nor docker compose is available"
  exit 1
fi

EXPECTED_SERVICES=(bluebot bunkbot covabot djcova ratbot)

check_containers_running() {
  echo ""
  echo "Checking container status..."
  ALL_RUNNING=true

  for service in "${EXPECTED_SERVICES[@]}"; do
    CONTAINER_INFO=$($COMPOSE_CMD ps "$service" --format json 2>/dev/null | jq -r '.Name + " | " + .State' || echo "")

    if [ -z "$CONTAINER_INFO" ]; then
      echo "FAIL  $service: NOT FOUND"
      ALL_RUNNING=false
      continue
    fi

    CONTAINER_STATE=$(echo "$CONTAINER_INFO" | cut -d'|' -f2 | xargs)

    if [ "$CONTAINER_STATE" = "running" ]; then
      echo "OK    $service: running"
    else
      echo "FAIL  $service: ${CONTAINER_STATE}"
      ALL_RUNNING=false
    fi
  done

  [ "$ALL_RUNNING" = true ]
}

check_restart_counts() {
  echo ""
  echo "Checking container restart counts..."
  EXCESSIVE=false

  for service in "${EXPECTED_SERVICES[@]}"; do
    CONTAINER_ID=$(docker ps -q -f "name=starbunk-${service}" 2>/dev/null || echo "")
    [ -z "$CONTAINER_ID" ] && continue

    RESTART_COUNT=$(docker inspect "$CONTAINER_ID" --format='{{.RestartCount}}' 2>/dev/null || echo "0")

    if [ "$RESTART_COUNT" -gt 3 ]; then
      echo "WARN  $service: restarted ${RESTART_COUNT} times"
      EXCESSIVE=true
    else
      echo "OK    $service: restart count ${RESTART_COUNT}"
    fi
  done

  if [ "$EXCESSIVE" = true ]; then
    echo ""
    echo "WARNING: Some containers have high restart counts — may indicate instability"
  fi
}

check_container_logs() {
  echo ""
  echo "Checking recent logs for fatal errors..."

  for service in "${EXPECTED_SERVICES[@]}"; do
    RECENT_LOGS=$($COMPOSE_CMD logs --tail=30 "$service" 2>/dev/null || echo "")
    [ -z "$RECENT_LOGS" ] && continue

    ERROR_COUNT=$(echo "$RECENT_LOGS" | grep -icE "(panicked at|FATAL|thread '.*' panicked)" || echo "0")

    if [ "$ERROR_COUNT" -gt 0 ]; then
      echo "WARN  $service: ${ERROR_COUNT} fatal/panic entries in recent logs"
      echo "$RECENT_LOGS" | grep -iE "(panicked at|FATAL|thread '.*' panicked)" | head -3 | sed 's/^/      /'
    else
      echo "OK    $service: no fatal errors"
    fi
  done
}

check_djcova_logs() {
  echo ""
  echo "Checking djcova voice/audio health..."

  RECENT_LOGS=$($COMPOSE_CMD logs --tail=50 djcova 2>/dev/null || echo "")
  [ -z "$RECENT_LOGS" ] && return

  # Check yt-dlp errors
  YTDLP_ERRORS=$(echo "$RECENT_LOGS" | grep -icE "(yt.dlp.*error|yt-dlp.*failed|yt-dlp exited)" || echo "0")
  if [ "$YTDLP_ERRORS" -gt 0 ]; then
    echo "WARN  djcova: ${YTDLP_ERRORS} yt-dlp error(s) in recent logs"
    echo "$RECENT_LOGS" | grep -iE "(yt.dlp.*error|yt-dlp.*failed|yt-dlp exited)" | head -3 | sed 's/^/      /'
  else
    echo "OK    djcova: no yt-dlp errors"
  fi

  # Check voice connection errors
  VOICE_ERRORS=$(echo "$RECENT_LOGS" | grep -icE "(Failed to join voice|VoiceService.*error|songbird.*error)" || echo "0")
  if [ "$VOICE_ERRORS" -gt 0 ]; then
    echo "WARN  djcova: ${VOICE_ERRORS} voice connection error(s) in recent logs"
    echo "$RECENT_LOGS" | grep -iE "(Failed to join voice|VoiceService.*error|songbird.*error)" | head -3 | sed 's/^/      /'
  else
    echo "OK    djcova: no voice connection errors"
  fi

  # Confirm bot connected successfully
  if echo "$RECENT_LOGS" | grep -qE "bot.*djcova.*connected|connected.*djcova"; then
    echo "OK    djcova: connected log entry found"
  else
    echo "WARN  djcova: no 'connected' log entry found — bot may not have started cleanly"
  fi
}

main() {
  ATTEMPT=1

  while [ $ATTEMPT -le $RETRY_COUNT ]; do
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Health Check Attempt $ATTEMPT of $RETRY_COUNT"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    if check_containers_running; then
      echo ""
      echo "All containers are running!"
      check_restart_counts
      check_container_logs
      check_djcova_logs
      echo ""
      echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
      echo "Health Check PASSED"
      echo "Completed: $(date)"
      echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
      exit 0
    fi

    if [ $ATTEMPT -lt $RETRY_COUNT ]; then
      echo ""
      echo "Retrying in ${RETRY_DELAY} seconds..."
      sleep $RETRY_DELAY
    fi

    ATTEMPT=$((ATTEMPT + 1))
  done

  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "Health Check FAILED"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "Container status:"
  $COMPOSE_CMD ps
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  exit 1
}

main
