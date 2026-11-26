#!/usr/bin/env bash
# STAGE PRE-FLIGHT (manager rule 13:16Z): ONE acquisition, cheap non-verdict checks only.
# Verdict legs run only after this prints PRE_FLIGHT_VERDICT GREEN with raw rcs.
# Fixes since first run (VOID rc 6 at 13:32:12Z, both honest): (a) bash -n must name run/ files
# by absolute path -- stage_common cd's out of run/, the loop tested stale top-level copies and
# stage_precheck.sh itself was 127; stale copies now deleted; (b) the test route panics outside
# a git checkout (cli_run.rs:570) -- panic rc 101 is NOT a refusal; unknown-name control now
# requires rc==2 AND no panic string; all test probes run from inside the head worktree.
. ./stage_common.sh
HEAD="$1"; T="head-$HEAD"; B="$PWD/$T/target/release/gunbc"
RUNDIR="$PWD/run"
FAIL=0
test -x "$B" || { echo PRECHECK_NO_BINARY; exit 6; }
"$B" --help > pc_top.log 2>&1; R=$?; echo "PC_TOP_HELP_RC=$R"; [ "$R" -ne 0 ] && FAIL=1
"$B" test --help > pc_test.log 2>&1; R=$?; echo "PC_TEST_HELP_RC=$R"; [ "$R" -ne 0 ] && FAIL=1
"$B" compile --help > pc_compile.log 2>&1; R=$?; echo "PC_COMPILE_HELP_RC=$R"; [ "$R" -ne 0 ] && FAIL=1
"$B" run --help > pc_run.log 2>&1; R=$?; echo "PC_RUN_HELP_RC=$R"; [ "$R" -ne 0 ] && FAIL=1
FE=$(grep -c -- "--function" pc_run.log || true); EE=$(grep -c -- "--entry" pc_run.log || true)
echo "PC_RUN_FLAG_COUNTS --function=$FE --entry=$EE (manager corroboration line: run route carries them; counts printed, not gated)"
"$B" claim_batch --help > pc_cb.log 2>&1; R=$?; echo "PC_CLAIMBATCH_HELP_RC=$R (nonzero expected: deleted route)"; [ "$R" -eq 0 ] && { echo "PC_FAIL deleted route alive"; FAIL=1; }
# Positional binding + fail-closed control, run FROM the head worktree (workspace_root rule).
( cd "$T" && "$B" test "//dag/test/claim/filesystem_effect_reach_wall:definitely_not_a_real_fn_zzz" --source-root dag --source-root src/v1 --source-root src/v2 ) > pc_unknown.log 2>&1; UN=$?
echo "PC_UNKNOWN_NAME_RC=$UN (expect 2: typed refusal, not a 101 panic)"
grep -qi panic pc_unknown.log && { echo "PC_FAIL panic on unknown name"; FAIL=1; }
[ "$UN" -eq 0 ] && { echo "PC_FAIL unknown name accepted"; FAIL=1; }
[ "$UN" -eq 101 ] && { echo "PC_FAIL panic rc on unknown name"; FAIL=1; }
grep -m1 -iE "refus|unknown|not found|no such|unbound" pc_unknown.log
# Freshness vs the VOID ruling (11:22:00Z = 1795692120).
MT=$(stat -c %Y "$B"); echo "PC_BINARY sha256=$(sha256sum "$B" | awk '{print $1}') mtime=$(stat -c %y "$B") mtime_epoch=$MT"
if [ "$MT" -gt 1795692120 ]; then echo "PC_FRESHNESS_RC=0 (> VOID ruling 11:22:00Z)"; else echo "PC_FRESHNESS_RC=1"; FAIL=1; fi
# Tree-side instruments at required non-zero counts.
instr_check "$T" || FAIL=1
# bash -n on every stage script, absolute (the run/ directory is the single authority; stale
# top-level copies removed).
for s in "$RUNDIR"/stage_common.sh "$RUNDIR"/stage1_build.sh "$RUNDIR"/stage2_module.sh "$RUNDIR"/stageG_gate.sh "$RUNDIR"/stageP1_probe.sh "$RUNDIR"/stageP2_probe.sh "$RUNDIR"/stage0b_floor_bin.sh "$RUNDIR"/stage2b_floorsrc.sh "$RUNDIR"/stageF_floor.sh "$RUNDIR"/stage_precheck.sh "$RUNDIR"/run_stage.sh; do
  bash -n "$s"; R=$?; echo "PC_BASHN $(basename "$s") rc=$R"; [ "$R" -ne 0 ] && FAIL=1
done
if [ "$FAIL" -eq 0 ]; then echo "PRE_FLIGHT_VERDICT GREEN all rcs as printed"; exit 0; else echo "PRE_FLIGHT_VERDICT VOID_STOP see PC_FAIL lines above"; exit 6; fi
