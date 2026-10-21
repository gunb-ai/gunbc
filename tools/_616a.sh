set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14616 @ 8f97fcaf55d -- FalsifierCadenceJob demoted from EXECUTING.
# Arms: (1) 8 gauntlet claims + 2 witness claims by name at head; (2) the Rust differential pin by
# name with its selection line; (3) MUT-A restore FalsifierCadenceJob into enrollment_is_scheduled
# (the base's predicate verbatim) -> the cadence-only RED must FAIL and the PIN must FAIL (the pin is
# what makes the Rust seed stop crediting the dead lane); (4) MUT-B mutate EXECUTING_SURFACES -> pin
# FAILS, .dag arms unaffected; (5) byte-exact restore + final clean.
OID=8f97fcaf55d57430f69ee0599c95bdaf6c79b45e  # gh pr view 14616 --json headRefOid
echo "HEADSHA_from_gh=$OID START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w616
git fetch -q origin +refs/pull/14616/head:refs/verifier/14616 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/14616)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14616) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w616 $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w616
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
CW=dag/gunbc/commit_workflow.dag
CLI=src/v1/stage0/src/cli_run.rs
GAUNT=src/v2/test/claim/long/gauntlet_lane_enrollment_witness.dag
WIT=src/v2/test/claim/falsifier_rehomed_rows_unexecuted_witness.dag
cp $CW /tmp/616_cw.orig; cp $CLI /tmp/616_cli.orig
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14616 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14616/release/claim_batch
[ -x $CB ] || { echo NO_BINARY; exit 9; }
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "CB_SHA=$(sha $CB|cut -c1-12) built $(date -u +%H:%M:%S)Z inside head worktree"
echo "  head blobs: cw=$(sha $CW|cut -c1-12) cli=$(sha $CLI|cut -c1-12)"
G8=witness_rehomed_rows_are_cadence_enrolled,witness_split_entries_keep_cheap_half_per_pr,gauntlet_lane_rehome_is_never_bare_deenrollment_holds,falsifier_cadence_only_row_is_refused_RED_control_holds,gauntlet_lane_rehome_bare_deenrollment_RED_control_holds,gauntlet_lane_gate_rehome_is_never_bare_deenrollment_holds,falsifier_cadence_only_gate_is_refused_RED_control_holds,gauntlet_lane_gate_bare_deenrollment_RED_control_holds
W2=falsifier_rehomed_drop_rows_are_priced_out_and_scheduled,falsifier_rehomed_roster_rows_are_not_executing
runga() { # tag
  local tag=$1
  echo "-------- [$tag] gauntlet 8 by name $(date -u +%H:%M:%S)Z"
  timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $GAUNT --functions "$G8" > /tmp/c616_g_$tag.out 2>&1; local rc=$?
  [ "$rc" = "124" ] && { echo "  rc=124 TIMEOUT EXCLUDED"; return 0; }
  echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c616_g_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/c616_g_$tag.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c616_g_$tag.out)"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c616_g_$tag.out | sed 's/^/    V| /' | cut -c1-160
  [ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c616_g_$tag.out)" = "0" ] && echo "  !! EXIT 0 HAVING OBSERVED NOTHING"
  return 0
}
runw() { # tag
  local tag=$1
  echo "-------- [$tag] witness 2 new claims by name $(date -u +%H:%M:%S)Z"
  timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$W2" > /tmp/c616_w_$tag.out 2>&1; local rc=$?
  [ "$rc" = "124" ] && { echo "  rc=124 TIMEOUT EXCLUDED"; return 0; }
  echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c616_w_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/c616_w_$tag.out)"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c616_w_$tag.out | sed 's/^/    V| /' | cut -c1-160
  return 0
}
rpin() { # tag
  local tag=$1
  echo "-------- [$tag] Rust pin executing_surfaces_mirror_enrollment_is_scheduled $(date -u +%H:%M:%S)Z"
  timeout 1500 cargo test -p v1-compiler --lib executing_surfaces_mirror_enrollment_is_scheduled -- --nocapture > /tmp/c616_p_$tag.log 2>&1; local rc=$?
  echo "  rc=$rc selected=[$(grep -aoE 'running [0-9]+ test' /tmp/c616_p_$tag.log | head -1)] result=[$(grep -a 'test result:' /tmp/c616_p_$tag.log | head -1 | cut -c1-70)]"
  grep -aE '^test [a-z]|panicked|assertion|left|right' /tmp/c616_p_$tag.log | head -8 | sed 's/^/    T| /' | cut -c1-175
  grep -aq 'running 0 test' /tmp/c616_p_$tag.log && echo "  !! SELECTED ZERO TESTS -- this arm proves nothing"
  return 0
}
echo "=== 1. clean at head: 8 gauntlet claims (REDs must FAIL as designed? NO -- they are RED CONTROLS: they assert the refusal HOLDS, so PASS) ==="
runga clean; runw clean
echo "=== 2. the pin, by name, selection line required ==="
rpin clean
echo "=== 3. MUT-A: restore the base predicate (count the anchor first) ==="
python3 - "$CW" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
old="""fn enrollment_is_scheduled(e: CommitCheckEnrollment) -> Bool {
  enrollment_has_surface(e: e, surface: GithubActionsCiJob)
}"""
new="""fn enrollment_is_scheduled(e: CommitCheckEnrollment) -> Bool {
  enrollment_has_surface(e: e, surface: GithubActionsCiJob)
    || enrollment_has_surface(e: e, surface: FalsifierCadenceJob)
}"""
n=s.count(old); print("  anchor count (expect 1):",n)
assert n==1, "anchor not unique"
open(p,"w").write(s.replace(old,new)); print("  MUT-A applied (dead lane credited again)")
PY
[ $? = 0 ] || { echo MUT_A_FAILED; exit 9; }
runga mutA; rpin mutA
git cat-file blob HEAD:$CW > $CW
cmp -s /tmp/616_cw.orig $CW && echo "  RESTORED_BYTE_IDENTICAL commit_workflow" || echo "  !!RESTORE_FAILED cw"
echo "=== 4. MUT-B: mutate the Rust seed (the drift the pin exists to catch) ==="
grep -c 'const EXECUTING_COMMIT_SURFACES: \[&str; 1\] = \["GithubActionsCiJob"\];' $CLI | xargs echo "  anchor (expect 1):"
sed -i 's/const EXECUTING_COMMIT_SURFACES: \[&str; 1\] = \["GithubActionsCiJob"\];/const EXECUTING_COMMIT_SURFACES: [\&str; 2] = ["GithubActionsCiJob", "FalsifierCadenceJob"];/' $CLI
grep -q 'FalsifierCadenceJob"\];' $CLI && echo "  MUT-B applied" || { echo MUT_B_NOT_APPLIED; exit 9; }
rpin mutB
git cat-file blob HEAD:$CLI > $CLI
cmp -s /tmp/616_cli.orig $CLI && echo "  RESTORED_BYTE_IDENTICAL cli_run" || echo "  !!RESTORE_FAILED cli"
echo "=== 5. final clean after restores ==="
runga final; rpin final
cmp -s /tmp/616_cw.orig $CW && echo FINAL_MATCH_cw || echo FINAL_MISMATCH_cw
cmp -s /tmp/616_cli.orig $CLI && echo FINAL_MATCH_cli || echo FINAL_MISMATCH_cli
echo DONE_616_A
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
