#!/usr/bin/env bash
# STAGE 2b: RED CONTROL -- the head witness run under the FLOOR binary on the floor tree.
# Without the PR the witness must REFUSE (no rows field, no census fns). rc==0 means the
# control failed: VOID_STOP. Head witness copied into the floor tree; floor is otherwise main.
. ./stage_common.sh
MB=d196f49052362ff8b3ac32d60928bd99f3f3be4a
HEAD="$1"; T="main-$MB"; B="$PWD/bin/gunbc-main-$MB"
test -x "$B" || { echo NO_FLOOR_BINARY; exit 9; }
test -e "head-$HEAD" || { echo HEAD_TREE_MISSING; exit 9; }
echo "BINARY_PROVENANCE=main-built manifest-unmatched sha256=$(sha256sum "$B" | awk '{print $1}') built_from=$(git -C "$T" rev-parse HEAD)"
cp "head-$HEAD/dag/test/claim/filesystem_effect_reach_wall.dag" "$T/dag/test/claim/filesystem_effect_reach_wall.dag"; echo "COPY_RC=$?"
cd "$T" || exit 9  # workspace_root rule: run from the checkout
echo "ROUTE_PROBE(test --help, unpiped):"
"$B" test --help > ../floor_help.log 2>&1; FTH=$?
echo "FLOOR_TEST_HELP_RC=$FTH"
"$B" test "//dag/test/claim/filesystem_effect_reach_wall" --source-root dag --source-root src/v1 --source-root src/v2 > ../floor_wall.log 2>&1; RC=$?
echo "FLOOR_WALL_RC=$RC"
grep -qi panic ../floor_wall.log && { echo RED_PANIC_NOT_REFUSAL; tail -n 20 ../floor_wall.log; exit 6; }
[ "$RC" -eq 0 ] && { echo VOID_STOP_RED_CONTROL_PASSED; tail -n 20 ../floor_wall.log; exit 6; }
grep -m6 -iE "refus|unknown|not found|unresolved|no such|error" ../floor_wall.log
echo "RED_CONTROL_REFUSED as required"; exit 0
