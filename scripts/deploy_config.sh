#!/bin/bash
set -e

# Dynamically locate repository root
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Ensure config/bots.yml exists
if [ ! -f "config/bots.yml" ]; then
  echo "Error: config/bots.yml not found."
  echo "Please create config/bots.yml with your reply-bots configuration first."
  exit 1
fi

echo "Deploying BunkBot configuration to GKE..."

# Base64 encode the config/bots.yml file in a portable way
B64_CONFIG=$(cat config/bots.yml | base64 | tr -d '\n')

# Run kubectl patch and rollout restart via google/cloud-sdk docker container
docker run --rm \
  -v ~/.config/gcloud:/root/.config/gcloud \
  -v ~/.kube:/root/.kube \
  -v "$REPO_ROOT":/app \
  -w /app \
  google/cloud-sdk:latest bash -c "
gcloud container clusters get-credentials starbunk-gke-cluster --region us-central1 --project starbunk-bot && \
echo 'Patching starbunk-secrets with new BOTS_CONFIG_YAML...' && \
kubectl patch secret starbunk-secrets -n starbunk -p '{\"data\":{\"BOTS_CONFIG_YAML\":\"$B64_CONFIG\"}}' && \
echo 'Triggering rollout restart for BunkBot...' && \
kubectl rollout restart deployment/bunkbot -n starbunk && \
kubectl rollout status deployment/bunkbot -n starbunk --timeout=3m
"

echo "Configuration deployed successfully!"
