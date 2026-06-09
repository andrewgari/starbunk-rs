---
name: build
description: Verify that all packages compile successfully
---

# Build

Compile all Rust packages to catch import or syntax errors fast.

## Instructions

1. Run the build command for all binaries:
   ```bash
   cargo build --bins
   ```

2. If you need to build specific bot binaries to verify them individually:
   ```bash
   for bot in bluebot bunkbot covabot djcova ratbot; do
       cargo build --bin $bot || echo "BROKEN: $bot"
   done
   ```

3. Report the outcome.
