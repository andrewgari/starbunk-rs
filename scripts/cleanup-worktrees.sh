#!/usr/bin/env bash
# cleanup-worktrees.sh
# Removes git worktrees whose branches have been merged or closed on GitHub.
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

echo "==> Checking worktrees against GitHub PR state..."
echo

REMOVED=0
KEPT=0
ERRORS=0

while IFS= read -r line; do
    # Parse "worktree <path>" lines from --porcelain output
    [[ "$line" =~ ^worktree\ (.+)$ ]] || continue
    wt_path="${BASH_REMATCH[1]}"

    # Skip the main worktree
    if [[ "$wt_path" == "$ROOT" ]]; then
        continue
    fi

    # Get the branch for this worktree
    branch=$(git -C "$wt_path" rev-parse --abbrev-ref HEAD 2>/dev/null || true)
    if [[ -z "$branch" || "$branch" == "HEAD" ]]; then
        echo "  SKIP  $wt_path (detached HEAD or unreadable)"
        continue
    fi

    # Ask GitHub about the PR state
    pr_state=$(gh pr view "$branch" --json state --jq '.state' 2>/dev/null || echo "NONE")

    case "$pr_state" in
        MERGED|CLOSED)
            if $DRY_RUN; then
                echo "  WOULD REMOVE  $wt_path  (branch: $branch, PR: $pr_state)"
            else
                echo "  REMOVING  $wt_path  (branch: $branch, PR: $pr_state)"
                git worktree remove "$wt_path" --force 2>/dev/null || {
                    echo "    WARNING: worktree remove failed, trying manual cleanup"
                    rm -rf "$wt_path"
                    git worktree prune
                }
                # Delete the tracking branch if it's fully merged
                git branch -d "$branch" 2>/dev/null || \
                    echo "    NOTE: branch $branch not deleted (unmerged commits or already gone)"
            fi
            (( REMOVED++ )) || true
            ;;
        OPEN)
            echo "  KEEP  $wt_path  (branch: $branch, PR: open)"
            (( KEPT++ )) || true
            ;;
        NONE)
            echo "  KEEP  $wt_path  (branch: $branch, no PR found)"
            (( KEPT++ )) || true
            ;;
        *)
            echo "  ERROR  $wt_path  (branch: $branch, unexpected state: $pr_state)"
            (( ERRORS++ )) || true
            ;;
    esac
done < <(git worktree list --porcelain)

echo
echo "==> Done. Kept: $KEPT  |  Removed: $REMOVED  |  Errors: $ERRORS"

if $DRY_RUN && (( REMOVED > 0 )); then
    echo
    echo "Run with --apply to actually remove these worktrees."
fi
