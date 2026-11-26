#!/usr/bin/env bash
# STAGE 0: floor-tree worktree at the manager-named base (main, d196f490523...), plain git, NO SLOT.
. ./stage_common.sh
MB=d196f49052362ff8b3ac32d60928bd99f3f3be4a
REPO="$HOME/scratch/bold-crane-719/gunbc"
test -d "$REPO/.git" || { echo REPO_MISSING; exit 9; }
git -C "$REPO" fetch -q origin main; echo "FETCH_RC=$?"
echo "ORIGIN_MAIN=$(git -C "$REPO" rev-parse origin/main)"
BASE=$(git -C "$REPO" rev-parse --verify "${MB}^{commit}" 2>&1); RC=$?
echo "BASE_REVPARSE_RC=$RC BASE_REVPARSE_OUT=$BASE"
[ $RC -ne 0 ] && { echo NO_FLOOR_BASE; exit 7; }
git -C "$REPO" merge-base --is-ancestor "$MB" origin/main; ANC=$?
echo "MERGE_BASE_IS_ANCESTOR_RC=$ANC"
[ $ANC -ne 0 ] && { echo NOT_ANCESTOR_OF_MAIN; exit 7; }
test -e "main-$MB/.git" && { echo MAIN_TREE_PRESENT; git -C "main-$MB" rev-parse HEAD; exit 0; }
git -C "$REPO" worktree add -f --detach "$HOME/scratch/bold-crane-719/main-$MB" "$MB" >/dev/null 2>&1; WT=$?
echo "WORKTREE_ADD_RC=$WT"; [ $WT -ne 0 ] && exit 8
git -C "main-$MB" rev-parse HEAD; echo "MAIN_TREE_HEAD_CHECK_RC=$?"
test -f "main-$MB/dag/test/claim/occurrence_binding_resolve_witness.dag" && echo CONTROL_MODULE_PRESENT || { echo CONTROL_MODULE_MISSING; exit 8; }
test -f "main-$MB/dag/test/claim/filesystem_effect_reach_wall.dag" && echo FLOOR_HAS_WALL_UNEXPECTED || echo FLOOR_LACKS_WALL_AS_EXPECTED
