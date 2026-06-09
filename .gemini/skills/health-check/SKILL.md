---
name: health-check
description: Comprehensive health check that runs build, test, lint, builds docker containers, checks container health/status, and reports to the user.
---

# Comprehensive Health Check

Run a full end-to-end local health check on the project.

## Instructions

Run the following checks sequentially:

1. **Build the Code**:
   ```bash
   cargo build --bins
   ```

2. **Test the Code**:
   ```bash
   cargo test --all
   ```

3. **Lint**:
   ```bash
   cargo clippy -- -D warnings
   ```

4. **Build Docker Containers**:
   ```bash
   docker compose -f docker/docker-compose.yml build
   ```

5. **Start the Containers**:
   ```bash
   docker compose -f docker/docker-compose.yml up -d
   ```

6. **Check Container Health/Status**:
   ```bash
   sleep 5
   docker compose -f docker/docker-compose.yml ps
   docker compose -f docker/docker-compose.yml logs --tail=50
   ```

7. **Clean up**:
   ```bash
   docker compose -f docker/docker-compose.yml down
   ```

8. **Report to the User**:
   Clearly report whether each phase (Build, Test, Lint, Docker Build, Health Check) passed or failed, and provide relevant error logs if any step failed.
