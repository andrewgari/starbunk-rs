#!/usr/bin/env bash
# cleanup-worktrees.sh
# Removes git worktrees that have no outstanding changes (clean working tree,
# no staged files, no untracked files). Does NOT require gh or network access.
#
# Usage:
#   bash scripts/cleanup-worktrees.sh          # dry-run (shows what would be removed)
#   bash scripts/cleanup-worktrees.sh --apply  # actually remove

set -euo pipefail

DRY_RUN=true
if [[ "${1:-}" == "--apply" ]]; then
    DRY_RUN=false
fi

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "==> Scanning worktrees for outstanding changes..."
echo

REMOVED=0
KEPT=0

while IFS= read -r line; do
    [[ "$line" =~ ^worktree\ (.+)$ ]] || continue
    wt_path="${BASH_REMATCH[1]}"

    # Never touch the main worktree
    if [[ "$wt_path" == "$ROOT" ]]; then
        continue
    fi

    branch=$(git -C "$wt_path" rev-parse --abbrev-ref HEAD 2>/dev/null || true)
    if [[ -z "$branch" || "$branch" == "HEAD" ]]; then
        echo "  SKIP  $wt_path (detached HEAD)"
        continue
    fi

    # Check for any outstanding changes: staged, unstaged, or untracked
    if git -C "$wt_path" status --porcelain | grep -q .; then
        echo "  KEEP  $wt_path  (branch: $branch, has outstanding changes)"
        (( KEPT++ )) || true
    else
        if $DRY_RUN; then
            echo "  WOULD REMOVE  $wt_path  (branch: $branch, clean)"
        else
            echo "  REMOVING  $wt_path  (branch: $branch, clean)"
            if ! git worktree remove "$wt_path" --force; then
                echo "  ERROR  git worktree remove failed for $wt_path — skipping (investigate manually)"
                (( KEPT++ )) || true
                continue
            fi
        fi
        (( REMOVED++ )) || true
    fi
done < <(git worktree list --porcelain)

echo
echo "==> Done. Kept: $KEPT  |  Removed: $REMOVED"

if $DRY_RUN && (( REMOVED > 0 )); then
    echo
    echo "Run with --apply to actually remove these worktrees."
fi
