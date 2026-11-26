#!/usr/bin/env bash
# One-off (NO SLOT): create a real git worktree at the PR head commit from the LOCAL clone.
. ./stage_common.sh
HEAD="$1"
REPO="$HOME/scratch/bold-crane-719/gunbc"
test -d "$REPO/.git" || { echo REPO_MISSING; exit 9; }
git -C "$REPO" fetch -q origin refs/heads/session/bold-crane-719; echo "FETCH_RC=$?"
git -C "$REPO" rev-parse --verify "$HEAD^{commit}" > /dev/null 2>&1; echo "HEAD_RESOLVE_RC=$?"
git -C "$REPO" rev-parse --verify "$HEAD^{commit}" > /dev/null 2>&1 || { echo HEAD_UNRESOLVABLE; exit 8; }
test -e "head-$HEAD/.git" && { git -C "head-$HEAD" rev-parse HEAD; echo "TREE_PRESENT head-$HEAD"; exit 0; }
git -C "$REPO" worktree add -f --detach "$HOME/scratch/bold-crane-719/head-$HEAD" "$HEAD" >/dev/null 2>&1; WT=$?
echo "WORKTREE_ADD_RC=$WT"; [ $WT -ne 0 ] && exit 8
GOT=$(git -C "head-$HEAD" rev-parse HEAD); echo "TREE_HEAD=$GOT"
test "$GOT" = "$HEAD" || { echo TREE_HEAD_MISMATCH; exit 8; }
test -f "head-$HEAD/dag/test/claim/filesystem_effect_reach_wall.dag" && echo WITNESS_PRESENT || { echo WITNESS_MISSING; exit 8; }
