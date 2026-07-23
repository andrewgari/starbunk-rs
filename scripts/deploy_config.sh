#!/bin/bash
set -e

# Dynamically locate repository root
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Ensure config/bunkbot/bots.yml exists
if [ ! -f "config/bunkbot/bots.yml" ]; then
  echo "Error: config/bunkbot/bots.yml not found."
  echo "Please create config/bunkbot/bots.yml with your reply-bots configuration first."
  exit 1
fi

echo "Deploying BunkBot configuration to GKE..."

# Run kubectl create configmap to update the config
docker run --rm \
  -v ~/.config/gcloud:/root/.config/gcloud \
  -v ~/.kube:/root/.kube \
  -v "$REPO_ROOT":/app \
  -w /app \
  google/cloud-sdk:latest bash -c "
set -euo pipefail

gcloud container clusters get-credentials starbunk-gke-cluster --region us-central1 --project starbunk-bot

echo 'Updating bunkbot-configs ConfigMap...'
kubectl create configmap bunkbot-configs -n starbunk --from-file=config/bunkbot/ --dry-run=client -o yaml | kubectl apply -f -

echo 'Triggering rollout restart for BunkBot...'
kubectl rollout restart deployment/bunkbot -n starbunk

kubectl rollout status deployment/bunkbot -n starbunk --timeout=3m
"

echo "Configuration deployed successfully!"
