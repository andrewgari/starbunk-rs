#!/usr/bin/env bash
# Pre-commit hook: cargo clippy + cargo build
# Matches the CI linter exactly.
#
# Install:
#   bash scripts/install-hooks.sh
#
# Requires cargo (installed via mise: `mise use rust@stable`)

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# --- rustfmt (format check) ---
STAGED_RS=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$' || true)
if [ -n "$STAGED_RS" ]; then
    UNFORMATTED=$(echo "$STAGED_RS" | xargs rustfmt --check 2>&1 | grep "^Diff" | awk '{print $3}' || true)
    if [ -n "$UNFORMATTED" ]; then
        echo "❌ rustfmt: files need formatting:"
        echo "$UNFORMATTED" | sed 's/^/  /'
        echo "   Run: rustfmt <file>"
        exit 1
    fi
fi

# --- cargo clippy ---
if command -v cargo &>/dev/null; then
    echo "→ cargo clippy -- -D warnings"
    cargo clippy -- -D warnings || exit 1
else
    echo "⚠️  cargo not found — run: mise install"
    exit 1
fi

# --- cargo build ---
echo "→ cargo build --bins"
cargo build --bins || exit 1

echo "✅ pre-commit checks passed"
