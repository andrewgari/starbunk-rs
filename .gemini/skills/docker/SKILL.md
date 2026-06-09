---
name: docker
description: Manage local Docker containers for Starbunk-rs development
---

# Docker

Manage Docker containers for Starbunk-rs (local dev builds from source).

All local dev commands use `docker/docker-compose.yml` (builds from source).

## Operations

- **up** (default): Start all containers
  ```bash
  docker compose -f docker/docker-compose.yml up -d
  ```

- **down**: Stop all containers
  ```bash
  docker compose -f docker/docker-compose.yml down
  ```

- **logs**: Show recent logs (optionally for a specific service)
  ```bash
  docker compose -f docker/docker-compose.yml logs --tail=50
  # For a specific service:
  docker compose -f docker/docker-compose.yml logs --tail=50 starbunk-rs-<bot>
  ```

- **build**: Build Docker images from source
  ```bash
  docker compose -f docker/docker-compose.yml build
  ```

- **ps**: Show container status
  ```bash
  docker compose -f docker/docker-compose.yml ps
  ```

- **restart**: Restart all containers
  ```bash
  docker compose -f docker/docker-compose.yml restart
  ```

Available services: `bluebot`, `bunkbot`, `covabot`, `djcova`, `ratbot`
(Container names are prefixed `starbunk-rs-<service>`)

Report the status of the operation and any errors encountered.
