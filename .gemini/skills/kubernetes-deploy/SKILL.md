---
name: kubernetes-deploy
description: Workflows for scheduling releases and deploying cartel services/bots to Kubernetes clusters.
---

# Kubernetes Release & Deployment Protocol

This skill governs the process of packaging, validating, and rolling out cartel services to a Kubernetes cluster.

---

## Deployment Workflow

### 1. Tagging & Changelog
*   **Active Agents:** **The Face** & **The Brains**
*   **Action:** 
    *   **The Face** confirms the release version and scope with **The Man**.
    *   **The Brains** compiles the git release tag and prepends raw changelogs to `wiki/Changelog.md`.

### 2. Packaging & Registry Push
*   **Active Agents:** **The Inspector**
*   **Action:** Run build scripts to containerize the targets (using multi-stage builds) and push images to the designated container registry.

### 3. Manifest Validation & Audit
*   **Active Agents:** **The Consultant**
*   **Action:** 
    *   Verify Kubernetes manifests (YAML files, Helm charts, or Kustomize targets) for security contexts, resource limits, and environment variables.
    *   Ensure OTEL metrics/tracing endpoints are mapped correctly in the configs.

### 4. Cluster Rollout
*   **Active Agents:** **The Inspector**
*   **Action:** 
    *   Perform deployment application (e.g. `kubectl apply -f k8s/` or `helm upgrade --install`).
    *   Monitor the rollout progression: `kubectl rollout status deployment/<deployment-name>`.

### 5. Verification & Smoke Testing
*   **Active Agents:** **The Mechanic** & **The Painter**
*   **Action:** 
    *   **The Mechanic** checks pod statuses and logs: `kubectl logs -l app=<app-name> --tail=100`.
    *   **The Painter** checks Grafana/Prometheus endpoints to confirm telemetry metrics are reporting correctly.
