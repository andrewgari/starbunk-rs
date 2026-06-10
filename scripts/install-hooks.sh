#!/usr/bin/env bash
# Install git hooks from scripts/ into .git/hooks/
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
HOOKS_DIR="$(git rev-parse --git-path hooks)"
SRC="$REPO_ROOT/scripts/git/hooks"

install_hook() {
    local name="$1"
    if [[ -f "$SRC/$name" ]]; then
        cp "$SRC/$name" "$HOOKS_DIR/$name"
        chmod +x "$HOOKS_DIR/$name"
        echo "✅ Installed $name hook"
    fi
}

# Legacy location
cp "$REPO_ROOT/scripts/pre-commit.sh" "$HOOKS_DIR/pre-commit"
chmod +x "$HOOKS_DIR/pre-commit"
echo "✅ Installed pre-commit hook"

install_hook "commit-msg"
