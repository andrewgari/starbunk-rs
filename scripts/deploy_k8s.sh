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
REGISTRY="us-central1-docker.pkg.dev/starbunk-bot/starbunk-repo"
BOTS=("bluebot" "bunkbot" "covabot" "djcova" "ratbot")

# Optional deployment tag (defaults to latest if not provided)
DEPLOY_TAG="${1:-latest}"

echo "Deploying Kubernetes manifests to GKE..."
echo "Cluster: $CLUSTER_NAME ($REGION)"
echo "Project: $PROJECT_ID"
echo "Image Tag: $DEPLOY_TAG"
echo "----------------------------------------"

# Run deployment steps via the Google Cloud SDK docker container to avoid local dependency issues
docker run --rm \
  -v ~/.config/gcloud:/root/.config/gcloud \
  -v ~/.kube:/root/.kube \
  -v "$REPO_ROOT":/app \
  -w /app \
  google/cloud-sdk:latest bash -c "
set -euo pipefail

echo 'Fetching GKE cluster credentials...'
gcloud container clusters get-credentials $CLUSTER_NAME --region $REGION --project $PROJECT_ID

echo 'Applying namespace.yaml...'
kubectl apply -f kubernetes/namespace.yaml

echo 'Applying all Kubernetes manifests...'
kubectl apply -f kubernetes/

# If a specific image tag (other than latest or if explicitly set) needs to be set, pin it now
if [ \"$DEPLOY_TAG\" != \"latest\" ]; then
  echo 'Pinning deployments to image tag: $DEPLOY_TAG...'
  for BOT in ${BOTS[@]}; do
    kubectl set image \"deployment/\${BOT}\" \"\${BOT}=$REGISTRY/starbunk-\${BOT}:$DEPLOY_TAG\" -n $NAMESPACE
  done
fi

echo 'Verifying rollout status...'
kubectl rollout status statefulset/postgres -n $NAMESPACE --timeout=3m
kubectl rollout status deployment/otel-collector -n $NAMESPACE --timeout=3m
for BOT in ${BOTS[@]}; do
  kubectl rollout status \"deployment/\${BOT}\" -n $NAMESPACE --timeout=3m
done
"

echo "----------------------------------------"
echo "Deployment completed successfully!"
