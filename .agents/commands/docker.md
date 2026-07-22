---
description: Manage local Docker containers for Starbunk-rs development
argument-hint: [up|down|logs|build|ps|restart]
allowed-tools: [Bash]
---

# Docker

Manage Docker containers for Starbunk-rs (local dev builds from source).

## Arguments

The user invoked this with: $ARGUMENTS

- `up` (or no argument): start all containers
- `down`: stop all containers
- `logs`: show recent logs (optionally for a specific service)
- `build`: build Docker images from source
- `ps`: show container status
- `restart`: restart all containers

## Instructions

Execute Docker operations based on the argument. All local dev commands use `docker/docker-compose.yml` (builds from source):

1. **up** (or no argument): Start all containers
   ```bash
   docker compose -f docker/docker-compose.yml up -d
   ```

2. **down**: Stop all containers
   ```bash
   docker compose -f docker/docker-compose.yml down
   ```

3. **logs**: Show recent logs (optionally for a specific service)
   ```bash
   docker compose -f docker/docker-compose.yml logs --tail=50
   ```
   If a service name follows (e.g. `logs bluebot`), append it:
   ```bash
   docker compose -f docker/docker-compose.yml logs --tail=50 starbunk-rs-bluebot
   ```

4. **build**: Build Docker images from source
   ```bash
   docker compose -f docker/docker-compose.yml build
   ```

5. **ps**: Show container status
   ```bash
   docker compose -f docker/docker-compose.yml ps
   ```

6. **restart**: Restart all containers
   ```bash
   docker compose -f docker/docker-compose.yml restart
   ```

Available services: `bluebot`, `bunkbot`, `covabot`, `djcova`, `ratbot`
(Container names are prefixed `starbunk-rs-<service>`)

Report the status of the operation and any errors encountered.
