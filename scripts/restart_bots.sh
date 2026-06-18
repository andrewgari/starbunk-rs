#!/bin/bash
set -e

# Dynamically locate repository root
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Configuration
CLUSTER_NAME="starbunk-gke-cluster"
REGION="us-central1"
PROJECT_ID="starbunk-bot"
NAMESPACE="starbunk"
ALL_BOTS=("bluebot" "bunkbot" "covabot" "djcova" "ratbot")

# Parse target bot
TARGET="${1:-all}"

# Validate target
if [ "$TARGET" != "all" ]; then
  VALID=false
  for BOT in "${ALL_BOTS[@]}"; do
    if [ "$TARGET" == "$BOT" ]; then
      VALID=true
      break
    fi
  done
  
  # Also allow otel-collector or postgres specifically
  if [ "$TARGET" == "otel-collector" ] || [ "$TARGET" == "postgres" ]; then
    VALID=true
  fi
  
  if [ "$VALID" = false ]; then
    echo "Error: Invalid target '$TARGET'."
    echo "Usage: ./scripts/restart_bots.sh [all|bluebot|bunkbot|covabot|djcova|ratbot|postgres|otel-collector]"
    exit 1
  fi
fi

echo "Triggering rolling restart in GKE..."
echo "Target: $TARGET"
echo "----------------------------------------"

# Run kubectl commands via google/cloud-sdk docker container
docker run --rm \
  -v ~/.config/gcloud:/root/.config/gcloud \
  -v ~/.kube:/root/.kube \
  -v "$REPO_ROOT":/app \
  -w /app \
  google/cloud-sdk:latest bash -c "
set -euo pipefail

echo 'Fetching GKE cluster credentials...'
gcloud container clusters get-credentials $CLUSTER_NAME --region $REGION --project $PROJECT_ID

if [ \"$TARGET\" == \"all\" ]; then
  for BOT in ${ALL_BOTS[@]}; do
    echo \"Restarting deployment/\$BOT...\"
    kubectl rollout restart deployment/\$BOT -n $NAMESPACE
  done
  for BOT in ${ALL_BOTS[@]}; do
    echo \"Waiting for deployment/\$BOT rollout to complete...\"
    kubectl rollout status deployment/\$BOT -n $NAMESPACE --timeout=3m
  done
else
  # Determine resource type (postgres is statefulset, otel-collector and bots are deployments)
  TYPE=\"deployment\"
  if [ \"$TARGET\" == \"postgres\" ]; then
    TYPE=\"statefulset\"
  fi
  echo \"Restarting \$TYPE/$TARGET...\"
  kubectl rollout restart \$TYPE/$TARGET -n $NAMESPACE
  echo \"Waiting for \$TYPE/$TARGET rollout to complete...\"
  kubectl rollout status \$TYPE/$TARGET -n $NAMESPACE --timeout=3m
fi
"

echo "----------------------------------------"
echo "Restart completed successfully!"
