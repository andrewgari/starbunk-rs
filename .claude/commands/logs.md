---
description: View Docker logs from Starbunk-rs containers on the remote server
argument-hint: [bot-name] [line-count]
allowed-tools: [Bash]
---

# Logs

View Docker logs from Starbunk-rs containers running on the remote Tower server.

## Arguments

The user invoked this with: $ARGUMENTS

Parse the arguments to determine:
- **Container**: `bluebot`, `bunkbot`, `covabot`, `djcova`, `ratbot`, or `all` (default: all)
- **Lines**: number of lines to show (default: 100)

Examples of valid arguments:
- `bunkbot` — last 100 lines from bunkbot
- `djcova 50` — last 50 lines from djcova
- `200` — last 200 lines from all containers
- `covabot -f` — follow logs from covabot (streaming)

## Instructions

SSH into the remote server using `tower` and run docker commands from the starbunk-rs stack directory.

Container names on the server are `starbunk-rs-<bot>`.

1. **Specific container**:
   ```bash
   ssh tower "cd /mnt/user/appdata/portainer/starbunk-rs && docker compose logs --tail=<lines> starbunk-rs-<bot>"
   ```

2. **All containers**:
   ```bash
   ssh tower "cd /mnt/user/appdata/portainer/starbunk-rs && docker compose logs --tail=<lines>"
   ```

3. **Follow mode** (if `-f` or `follow` is in arguments):
   ```bash
   ssh tower "cd /mnt/user/appdata/portainer/starbunk-rs && docker compose logs -f starbunk-rs-<bot>"
   ```

Available containers:
- `bluebot` (`starbunk-rs-bluebot`) — Blue Mage pattern-matching bot
- `bunkbot` (`starbunk-rs-bunkbot`) — Administrative backbone and reply bot
- `covabot` (`starbunk-rs-covabot`) — AI personality emulator
- `djcova` (`starbunk-rs-djcova`) — Voice channel music streaming
- `ratbot` (`starbunk-rs-ratbot`) — Secret Santa / Ratmas organiser

After showing logs, offer to filter or search for specific patterns if the output is large.
