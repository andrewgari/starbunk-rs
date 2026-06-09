#!/bin/bash
set -euo pipefail

# Starbunk-Rs Production Deployment Script
# Runs on the Tower server to pull latest images and restart services.
# Called by the GitHub Actions deploy workflow via SSH.

COMPOSE_DIR="${1}"
DEPLOY_TAG="${2:-main}"
VERSION="${3:-unknown}"
INCOMING_COMPOSE="${4:-}"  # optional path to a new docker-compose.yml to install after backup

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Starbunk-Rs Production Deployment"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Compose Directory: ${COMPOSE_DIR}"
echo "Deploy Tag:        ${DEPLOY_TAG}"
echo "Version:           ${VERSION}"
echo "Time:              $(date)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ ! -d "$COMPOSE_DIR" ]; then
  echo "ERROR: Compose directory not found: $COMPOSE_DIR"
  exit 1
fi

cd "$COMPOSE_DIR"

if [ ! -f "docker-compose.yml" ]; then
  echo "ERROR: docker-compose.yml not found in $COMPOSE_DIR"
  exit 1
fi

if command -v docker-compose &> /dev/null; then
  COMPOSE_CMD="docker-compose --env-file stack.env"
elif docker compose version &> /dev/null; then
  COMPOSE_CMD="docker compose --env-file stack.env"
else
  echo "ERROR: Neither docker-compose nor docker compose is available"
  exit 1
fi

echo "Using: $COMPOSE_CMD"
echo ""

backup_current_state() {
  echo "Backing up current deployment state..."
  BACKUP_DIR="${COMPOSE_DIR}/backups/deployment-$(date +%Y%m%d-%H%M%S)"
  mkdir -p "$BACKUP_DIR"
  cp docker-compose.yml "$BACKUP_DIR/" 2>/dev/null || true
  cp stack.env "$BACKUP_DIR/" 2>/dev/null || true
  $COMPOSE_CMD ps --format json > "$BACKUP_DIR/containers.json" 2>/dev/null || true
  echo "Backup saved to: $BACKUP_DIR"
  echo ""
}

install_incoming_compose() {
  if [ -n "$INCOMING_COMPOSE" ] && [ -f "$INCOMING_COMPOSE" ]; then
    echo "Installing new docker-compose.yml from: $INCOMING_COMPOSE"
    mv "$INCOMING_COMPOSE" docker-compose.yml
    echo "docker-compose.yml updated"
    echo ""
  fi
}

pull_images() {
  echo "Pulling latest container images (tag: ${DEPLOY_TAG})..."
  export IMAGE_TAG="${DEPLOY_TAG}"
  timeout 600 $COMPOSE_CMD pull --quiet || {
    echo "ERROR: Image pull timed out or failed"
    exit 1
  }
  echo "Images pulled successfully"
  echo ""
}

restart_services() {
  echo "Restarting services with new images..."
  $COMPOSE_CMD down 2>/dev/null || true
  # Remove any starbunk-rs containers not owned by this compose project.
  docker ps -aq --filter "name=starbunk-rs-" | xargs -r docker rm -f 2>/dev/null || true
  $COMPOSE_CMD up -d --no-build
  EXIT_CODE=$?
  if [ $EXIT_CODE -ne 0 ]; then
    echo "ERROR: Service restart failed with exit code $EXIT_CODE"
    exit $EXIT_CODE
  fi
  echo "Services restarted"
  echo ""
}

wait_for_stability() {
  echo "Waiting for containers to stabilize..."
  sleep 15
  echo "Stability wait complete"
  echo ""
}

show_container_status() {
  echo "Current Container Status:"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  $COMPOSE_CMD ps
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
}

check_restart_loops() {
  echo "Checking for restart loops..."
  RESTARTING=$($COMPOSE_CMD ps --filter "status=restarting" --format json 2>/dev/null | jq -r '.Name' || echo "")
  if [ -n "$RESTARTING" ]; then
    echo "WARNING: Containers in restarting state:"
    echo "$RESTARTING"
    return 1
  else
    echo "No restart loops detected"
  fi
  echo ""
}

main() {
  backup_current_state
  install_incoming_compose
  pull_images
  restart_services
  wait_for_stability
  show_container_status
  check_restart_loops

  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "Deployment completed successfully!"
  echo "Version: ${VERSION}"
  echo "Tag:     ${DEPLOY_TAG}"
  echo "Time:    $(date)"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

main
