#!/usr/bin/env bash
# STAGE 2: whole-module claim run on the wall witness + exact-label spellings + name refusal.
# Route (head --help, unpiped): gunbc test takes POSITIONAL target patterns (absolute labels
# //pkg:fn, package //pkg); NO --entry/--tests on this route; claim_batch is deleted (rc 2).
# workspace_root rule (cli_run.rs:570 panic at 13:32Z): the test route must run FROM the git
# checkout -- binaries are shared across workspaces, cwd is the runtime workspace fact. Every
# test invocation below runs in a subshell cd'd into the head worktree, tree-relative roots.
. ./stage_common.sh
HEAD="$1"; T="head-$HEAD"; B="$PWD/$T/target/release/gunbc"
test -x "$B" || { echo NO_BINARY; exit 9; }
echo "BINARY_PROVENANCE=head-built manifest-unmatched sha256=$(sha256sum "$B" | awk '{print $1}') built_from=$(git -C "$T" rev-parse HEAD)"
instr_check "$T" || exit 6
echo "TREE_SHA=$(git -C "$T" rev-parse HEAD)"
echo "GATE1MIN=$(awk '{print $1}' /proc/loadavg)"
cd "$T" || exit 9
"$B" claim_batch --help > ../cb_help.log 2>&1; R=$?; echo "PROBE_CLAIMBATCH_HELP_RC=$R (deleted route: nonzero expected)"
"$B" test --help > ../t_help.log 2>&1; TH=$?
echo "PROBE_TEST_HELP_RC=$TH"
[ "$TH" -ne 0 ] && { echo VOID_NO_TEST_ROUTE; exit 6; }
ROOTS=(--source-root dag --source-root src/v1 --source-root src/v2)
echo "INVOCATION-1 (package): (cd $T && $B test //dag/test/claim/filesystem_effect_reach_wall ${ROOTS[*]})"
"$B" test "//dag/test/claim/filesystem_effect_reach_wall" "${ROOTS[@]}" > ../module_run.log 2>&1; RC=$?
echo "MODULE_RC=$RC"
grep -qi panic ../module_run.log && { echo VOID_STOP_PANIC_MODULE; tail -n 20 ../module_run.log; exit 6; }
grep -cE "^ok " ../module_run.log | sed 's/^/OK_COUNT=/'
grep -ciE "FAIL" ../module_run.log | sed 's/^/FAIL_COUNT=/'
grep -iE "selected|declared" ../module_run.log | head -4
grep -iE "census|refus" ../module_run.log | head -14
echo "INVOCATION-2 (exact label, //dag form):"
"$B" test "//dag/test/claim/filesystem_effect_reach_wall:demand_restatement_new_row_refuses_against_the_real_roster" "${ROOTS[@]}" > ../label_a.log 2>&1; LA=$?
echo "LABEL_A_RC=$LA"; grep -cE "^ok " ../label_a.log | sed 's/^/LABEL_A_OK=/'
grep -qi panic ../label_a.log && { echo VOID_STOP_PANIC_LABEL_A; tail -n 20 ../label_a.log; exit 6; }
echo "INVOCATION-3 (exact label, //test form, expected refusal rc 2 NOT a panic):"
"$B" test "//test/claim/filesystem_effect_reach_wall:demand_restatement_new_row_refuses_against_the_real_roster" "${ROOTS[@]}" > ../label_b.log 2>&1; LB=$?
echo "LABEL_B_RC=$LB"; grep -cE "^ok " ../label_b.log | sed 's/^/LABEL_B_OK=/'
if [ "$(grep -cE '^ok ' ../label_a.log || true)" -ge 1 ]; then BINDING="LABEL_A_//dag_form"; elif [ "$(grep -cE '^ok ' ../label_b.log || true)" -ge 1 ]; then BINDING="LABEL_B_//test_form"; else BINDING=NONE_VACUOUS; fi
echo "LABEL_SPELLING_BINDING=$BINDING"
[ "$BINDING" = "NONE_VACUOUS" ] && { echo VOID_STOP_NO_SITE_SELECTED; tail -n 30 ../module_run.log; tail -n 30 ../label_a.log; exit 6; }
echo "INVOCATION-4 (unknown name must refuse with rc 2, not panic):"
"$B" test "//dag/test/claim/filesystem_effect_reach_wall:definitely_not_a_real_fn_zzz" "${ROOTS[@]}" > ../name_refuse.log 2>&1; N1=$?
echo "NAME_RC=$N1 (expect 2: refuses rather than reporting a pass)"
grep -qi panic ../name_refuse.log && { echo VOID_STOP_PANIC_NAME_LEG; tail -n 20 ../name_refuse.log; exit 6; }
[ "$N1" -eq 0 ] && { echo VOID_STOP_UNKNOWN_NAME_ACCEPTED; tail -n 20 ../name_refuse.log; exit 6; }
grep -m2 -iE "refus|unknown|not found|no such|unbound" ../name_refuse.log
[ "$RC" -eq 0 ] || { echo MODULE_FAILED; tail -n 60 ../module_run.log; }
cd .. || exit 9
exit $RC
