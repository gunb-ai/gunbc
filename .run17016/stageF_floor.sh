#!/usr/bin/env bash
# STAGE F: floor route-legitimacy control -- floor binary + floor tree CONTROL module, gunbc test
# route (claim_batch is DELETED on main, gunbc#16872; not used anywhere). Proves the route exists
# and selects sites at the floor, so a 2b refusal is the PR's absence, not a broken route.
. ./stage_common.sh
MB=d196f49052362ff8b3ac32d60928bd99f3f3be4a
T="main-$MB"; B="$PWD/bin/gunbc-main-$MB"
test -x "$B" || { echo NO_FLOOR_BINARY; exit 9; }
echo "BINARY_PROVENANCE=main-built manifest-unmatched sha256=$(sha256sum "$B" | awk '{print $1}') built_from=$(git -C "$T" rev-parse HEAD)"
cd "$T" || exit 9  # workspace_root rule
"$B" claim_batch --help > ../f_cb_help.log 2>&1; FCB=$?
echo "FLOOR_CLAIMBATCH_HELP_RC=$FCB (expect nonzero: route deleted)"
"$B" test --help > ../f_test_help.log 2>&1; FH=$?
echo "FLOOR_TEST_HELP_RC=$FH"
[ "$FH" -ne 0 ] && { echo VOID_NO_TEST_ROUTE; exit 6; }
"$B" test "//dag/test/claim/occurrence_binding_resolve_witness" --source-root dag --source-root src/v1 --source-root src/v2 > ../floor_ctrl.log 2>&1; RC=$?
echo "FLOOR_CONTROL_RC=$RC"
grep -qi panic ../floor_ctrl.log && { echo VOID_STOP_PANIC_FLOOR; tail -n 20 ../floor_ctrl.log; exit 6; }
OK=$(grep -cE "^ok " ../floor_ctrl.log || true); DECL=$(grep -ciE "declared|selected" ../floor_ctrl.log || true)
echo "FLOOR_OK_COUNT=$OK DECLARED_SELECTED_LINES=$DECL"
head -4 ../floor_ctrl.log
[ "$RC" -eq 0 ] && [ "$OK" -gt 0 ] || { echo VOID_STOP_NO_SITES_SELECTED; tail -n 20 ../floor_ctrl.log; exit 6; }
echo "FLOOR_ROUTE_OK"
