---
name: health-check
description: Comprehensive health check that runs build, test, lint, builds docker containers, checks container health/status, and reports to the user.
---

# Comprehensive Health Check

Run a full end-to-end local health check on the project, including compiling the code, running tests, linting, building Docker images, and verifying container status and recent logs.

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
   Build the images using the local docker-compose override:
   ```bash
   docker compose -f docker/docker-compose.yml build
   ```

5. **Start the Containers**:
   Start the services in the background:
   ```bash
   docker compose -f docker/docker-compose.yml up -d
   ```

6. **Check Container Health/Status**:
   Verify services are up and inspect recent logs:
   ```bash
   # Wait a few seconds for services to start
   sleep 5

   # Verify service/container status
   docker compose -f docker/docker-compose.yml ps

   # Tail recent logs for quick failure signals
   docker compose -f docker/docker-compose.yml logs --tail=50
   ```

7. **Clean up**:
   Bring the containers back down after the check is complete:
   ```bash
   docker compose -f docker/docker-compose.yml down
   ```

8. **Report to the User**:
   Consolidate the results of all the above steps. Clearly report whether each phase (Build, Test, Lint, Docker Build, Health Check) passed or failed, and provide the relevant error logs if any step failed.
