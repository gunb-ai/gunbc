set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14539 DELTA RE-VERIFY at the gh-printed head -- manager scope: the EXIT SPLIT (attempt 3).
# Attempt 1 VOID: 'module X { }' is not .dag grammar -> every arm died in the parse-phase preflight.
# Attempt 2 VOID for the decisive arms, and it told me why:
#   - name-error entry INSIDE the tree -> rc=1 with stdout_bytes=0: the process aborts BEFORE the
#     per-group loop (the cause is in stderr), so the claimed 'FAIL ... (closure diagnostics in'
#     line was never printed. A nonzero exit with no verdict line is not the claimed arm.
#   - entry as a DIRECTORY -> rc=2 from 'cannot parse --entry ... while validating functions' (the
#     parse-phase PREFLIGHT), not from the typed 'UNOBSERVED ... (entry resolve failed:' arm.
# Attempt 3 runs every decisive arm in TWO root shapes:
#   n = only a private temp root (whole-tree published-mock corpus empty -> documented entry-closure
#       fallback; this is the shape where the per-group loop is reached)
#   w = temp root + dag + src/v2 (the shape a real label run uses)
# and prints the stderr TAIL as well as stdout, because the aborting cause lives in stderr.
OID=aef4b0138d3077866fa601001ea9a2bb75cd19e8  # printed by gh pr view 14539 --json headRefOid on MY side and embedded (the runner has no gh;
# my previous attempt called gh on the runner and the script correctly refused to run rather than guess one.
# The FETCH_HEAD equality assertion below is the guard that actually binds the run to this commit.
if [ -z "${OID:-}" ] || [ "${#OID}" != "40" ]; then echo "NO_HEAD_OID from gh - refusing to guess"; exit 9; fi
echo "HEADSHA_from_gh=$OID"
echo "PWD=$(pwd) RUNNER_HEAD=$(git rev-parse HEAD | cut -c1-12) (runner git is main; head gating ONLY from FETCH_HEAD) START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt539
git fetch -q origin refs/pull/14539/head 2>&1 | tail -1
[ "$(git rev-parse FETCH_HEAD)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse FETCH_HEAD) want=$OID -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt539 $OID || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/wt539
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo "WORKTREE_NOT_AT_HEAD"; exit 9; }
echo "WORKTREE_HEAD=$(git -C $WT rev-parse HEAD)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
CBRS=src/v1/stage0/src/bin/claim_batch.rs
mkdir -p tools/stage && cp -f $WT/$CBRS tools/stage/cb_head.rs
HEAD_CB_256=$(sha tools/stage/cb_head.rs); echo "HEAD_CB_256=$HEAD_CB_256"

export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14539 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
CB=/root/workspace/repo-root/.t14539/release/claim_batch
[ -x $CB ] || { echo "NO BINARY - nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha $CB | cut -c1-12) BUILT_AT $(date -u +%H:%M:%S)Z"

R=/tmp/r539; rm -rf $R; mkdir -p $R
printf 'module v.verif539.ok\n\ntest fn ok_holds() -> Bool {\n  true\n}\n' > $R/ok.dag
printf 'module v.verif539.badname\n\nimport v.verif539.ok { zz_no_such_name_539 }\n\ntest fn badname_claim() -> Bool {\n  true\n}\n' > $R/badname.dag
printf 'module v.verif539.badtype\n\ntest fn badtype_claim() -> Bool {\n  7\n}\n' > $R/badtype.dag
mkdir -p $R/adir.dag
for f in ok badname badtype; do echo "fixture $f.dag $(sha $R/$f.dag | cut -c1-12)"; done
SRN="--source-root $R"
SRW="--source-root $R --source-root dag --source-root src/v2"

arm() { # label roots -- rest of argv
  local label=$1; shift; local roots=$1; shift
  echo "-------- [$label] roots=$(echo $roots | cut -c1-60) $(date -u +%H:%M:%S)Z"
  timeout 300 "$CB" --claim-run --hermetic $roots "$@" > /tmp/a_$label.out 2> /tmp/a_$label.err
  local rc=$?
  if [ "$rc" = "124" ]; then echo "  rc=124 TIMEOUT -- not a result, EXCLUDED"; return 0; fi
  echo "  argv: $CB --claim-run --hermetic $roots $*" | cut -c1-230
  echo "  rc=$rc stdout_bytes=$(wc -c < /tmp/a_$label.out) stderr_bytes=$(wc -c < /tmp/a_$label.err)"
  echo "  --- stdout verbatim:"; sed 's/^/    O| /' /tmp/a_$label.out | head -12 | cut -c1-190
  echo "  --- stderr tail:"; tail -10 /tmp/a_$label.err | sed 's/^/    Et| /' | cut -c1-190
  echo "  counters: PASS=$(grep -ac '^PASS ' /tmp/a_$label.out) FAIL=$(grep -ac '^FAIL ' /tmp/a_$label.out) UNOBSERVED=$(grep -ac '^UNOBSERVED ' /tmp/a_$label.out) SKIP=$(grep -ac '^SKIP ' /tmp/a_$label.out)"
  local pre=$(grep -ac 'cannot parse --entry' /tmp/a_$label.err)
  local em=$(grep -ac '^UNOBSERVED .*entry resolve failed' /tmp/a_$label.out)
  local ri=$(grep -ac '^UNOBSERVED .*resolve did not reach a verdict' /tmp/a_$label.out)
  local cd=$(grep -ac '^FAIL .*closure diagnostics in' /tmp/a_$label.out)
  echo "  exit_producer: preflight-cannot-parse=$pre entryMissing-UNOBSERVED=$em infra-UNOBSERVED=$ri closureDiagnostics-FAIL=$cd"
  if [ "$pre" != "0" ]; then echo "  !! PREFLIGHT REFUSAL: never reached the split; says nothing about the split."; fi
  if [ "$cd" = "0" ] && [ "$em" = "0" ] && [ "$ri" = "0" ] && [ "$rc" != "0" ]; then echo "  !! NONZERO EXIT WITH NO TYPED VERDICT LINE: not the claimed arm; read the stderr tail."; fi
  return 0
}
echo "=== A4 healthy control -- expect rc=0 PASS=1"
arm A4n "$SRN" --entry $R/ok.dag --functions ok_holds
arm A4w "$SRW" --entry $R/ok.dag --functions ok_holds
echo "=== A1 NAME error in the entry closure -- claimed rc=1 WITH a 'FAIL ... (closure diagnostics in' line"
arm A1n "$SRN" --entry $R/badname.dag --functions badname_claim
arm A1w "$SRW" --entry $R/badname.dag --functions badname_claim
echo "=== A1b TYPE error in the entry closure -- same claimed arm"
arm A1bn "$SRN" --entry $R/badtype.dag --functions badtype_claim
arm A1bw "$SRW" --entry $R/badtype.dag --functions badtype_claim
echo "=== A2 nonexistent entry path -- claimed rc=2 via the EntryMissing UNOBSERVED arm"
arm A2n "$SRN" --entry $R/does_not_exist_539.dag --functions whatever_claim
arm A2w "$SRW" --entry $R/does_not_exist_539.dag --functions whatever_claim
echo "=== A2b entry is a DIRECTORY -- which line produces the 2?"
arm A2bn "$SRN" --entry $R/adir.dag --functions whatever_claim
echo "=== A6 both claimed classes together (closure error + missing entry) -- claimed rc=1 (1 beats 2)"
arm A6n "$SRN" --entry $R/badname.dag --functions badname_claim --entry $R/does_not_exist_539.dag --functions whatever_claim
arm A6w "$SRW" --entry $R/badname.dag --functions badname_claim --entry $R/does_not_exist_539.dag --functions whatever_claim

echo "=== U1 unit tests by name; 'running 0 tests' is not a pass, so the selection line is printed."
runtest() {
  local label=$1 pat=$2 log=/tmp/t_$label.log
  timeout 900 cargo test -p v1-compiler --bin claim_batch "$pat" -- --nocapture > $log 2>&1
  local rc=$?
  echo "  [$label] pattern='$pat' rc=$rc selected=[$(grep -aoE 'running [0-9]+ test' $log | head -1)] result=[$(grep -a 'test result:' $log | head -1 | cut -c1-80)]"
  grep -aE '^test [a-z]' $log | head -5 | sed 's/^/    T| /' | cut -c1-160
  if [ "$rc" != "0" ]; then tail -18 $log | sed 's/^/    Tt| /' | cut -c1-170; fi
  return 0
}
runtest U1_mixed exit_composition_is_failed_over_unobserved_over_clean
runtest U1_missing nonexistent_explicit_function_refuses_before_resolution

echo "=== M1 MUTATION: collapse the split -- ClosureDiagnostics counts as UNOBSERVED (a finding becomes a gap: 1 -> 2)"
python3 - <<'PYA'
import sys
p="/tmp/wt539/src/v1/stage0/src/bin/claim_batch.rs"
b=open(p,'rb').read()
old=b'''            EntryResolveRefusal::ClosureDiagnostics { .. } => {
                println!(
                    "FAIL {} (closure diagnostics in {}: {})",
                    function,
                    group.entry,
                    refusal.message().lines().next().unwrap_or("")
                );
                timings.any_failed = true;
            }'''
if b.count(old)!=1:
    print("MUTATION_ANCHOR_NOT_UNIQUE count=%d" % b.count(old)); sys.exit(3)
open(p,'wb').write(b.replace(old, old.replace(b"timings.any_failed = true;", b"timings.any_unobserved = true;")))
print("M1_APPLIED: whole-balanced-match-arm, one flag assignment only")
PYA
if [ $? -ne 0 ]; then echo "M1_NOT_APPLIED -- arms below would be a duplicate baseline, not a red"; exit 9; fi
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
arm M1n "$SRN" --entry $R/badname.dag --functions badname_claim
arm M1w "$SRW" --entry $R/badname.dag --functions badname_claim
runtest M1_test exit_composition_is_failed_over_unobserved_over_clean

echo "=== M2 MUTATION: collapse the other way -- EntryMissing counts as a FINDING (gap becomes a finding: 2 -> 1)"
cp -f /root/workspace/repo-root/tools/stage/cb_head.rs $WT/$CBRS
python3 - <<'PYB'
import sys
p="/tmp/wt539/src/v1/stage0/src/bin/claim_batch.rs"
b=open(p,'rb').read()
old=b'''            EntryResolveRefusal::EntryMissing { .. } => {
                println!(
                    "UNOBSERVED {} (entry resolve failed: {})",
                    function, group.entry
                );
                timings.any_unobserved = true;
            }'''
if b.count(old)!=1:
    print("MUTATION_ANCHOR_NOT_UNIQUE count=%d" % b.count(old)); sys.exit(3)
open(p,'wb').write(b.replace(old, old.replace(b"timings.any_unobserved = true;", b"timings.any_failed = true;")))
print("M2_APPLIED: EntryMissing arm now counts as a finding")
PYB
if [ $? -ne 0 ]; then echo "M2_NOT_APPLIED"; exit 9; fi
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
arm M2n "$SRN" --entry $R/does_not_exist_539.dag --functions whatever_claim
arm M2w "$SRW" --entry $R/does_not_exist_539.dag --functions whatever_claim
runtest M2_test exit_composition_is_failed_over_unobserved_over_clean

echo "=== RESTORE + post-restore controls"
cp -f /root/workspace/repo-root/tools/stage/cb_head.rs $WT/$CBRS
if [ "$(sha $WT/$CBRS)" = "$HEAD_CB_256" ]; then echo "  RESTORED_BYTE_IDENTICAL ${HEAD_CB_256:0:16}"; else echo "  RESTORE_FAILED got=$(sha $WT/$CBRS) want=$HEAD_CB_256"; exit 9; fi
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
arm R1n "$SRN" --entry $R/badname.dag --functions badname_claim
arm R2n "$SRN" --entry $R/does_not_exist_539.dag --functions whatever_claim
arm R3n "$SRN" --entry $R/ok.dag --functions ok_holds
echo DONE_14539_E3
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
rm -rf $WT $R
