set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14539 DELTA RE-VERIFY at aef4b0138d3 -- manager scope: the EXIT SPLIT.
# Attempt 1 is VOID and disclosed: my fixtures used `module X { }` (brace form), which is not .dag
# grammar, so every arm died in the parse-phase preflight (`cannot parse --entry ... while validating
# functions`, rc=2) and no arm ever reached the split. Attempt 2 uses real grammar copied from
# src/v2/test/claim/*, and every arm prints WHICH line produced its exit code, because an exit code
# never establishes a cause: 'cannot parse --entry' (preflight, NOT the split) vs 'UNOBSERVED ... (entry
# resolve failed:' (EntryMissing arm) vs 'FAIL ... (closure diagnostics in' (ClosureDiagnostics arm).
OID=aef4b0138d3077866fa601001ea9a2bb75cd19e8  # printed by `gh pr view 14539 --json headRefOid` on my side and embedded here; my first attempt read it from a local file, which does not exist on the runner (its /tmp is fresh per dispatch) and the
# script correctly refused to run rather than guess. The FETCH_HEAD assertion below is the real guard.
[ -n "${OID:-}" ] || { echo "NO_HEAD_OID - refusing to guess a SHA"; exit 9; }
echo "PWD=$(pwd) RUNNER_HEAD=$(git rev-parse HEAD | cut -c1-12) (runner git is main; head gating ONLY from the FETCH_HEAD assertion) START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt539
git fetch -q origin refs/pull/14539/head 2>&1 | tail -1
[ "$(git rev-parse FETCH_HEAD)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse FETCH_HEAD) want=$OID -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt539 $OID || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/wt539
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo "WORKTREE_NOT_AT_HEAD"; exit 9; }
echo "WORKTREE_HEAD=$(git -C $WT rev-parse HEAD) HEADSHA_from_gh=$OID"
sha() { sha256sum "$1" | cut -d' ' -f1; }
CBRS=src/v1/stage0/src/bin/claim_batch.rs
HEAD_CB_SHA=$(git -C $WT rev-parse HEAD:$CBRS)   # resolved from the worktree itself, never hand-written
echo "head claim_batch.rs blob=$HEAD_CB_SHA"
mkdir -p tools/stage && cp -f $WT/$CBRS tools/stage/cb_head.rs
[ "$(sha tools/stage/cb_head.rs)" = "$(git -C $WT hash-object $CBRS | tr -d '[:space:]' | sed 's/^/x/;s/x//')" ] || echo "note: blob is sha1; byte-integrity guard below is sha256 of the restored copy"
HEAD_CB_256=$(sha tools/stage/cb_head.rs); echo "HEAD_CB_256=$HEAD_CB_256"

export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14539 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
CB=/root/workspace/repo-root/.t14539/release/claim_batch
[ -x $CB ] || { echo "NO BINARY - nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha $CB | cut -c1-12) BUILT_AT $(date -u +%H:%M:%S)Z"
cd $WT

# ---- fixtures: real .dag grammar, inside the tree, index-tracked where the arm needs a subject ----
D=src/v2/test/claim
cat > $D/verifier_539_ok.dag <<'EOF'
module v2.test.claim.verifier_539_ok

import std.types { Bool }

test fn verifier_plant_ok_holds() -> Bool {
  true
}
EOF
# tracked so the resolve index carries it (otherwise every healthy arm is EntryMissing)
git add -f $D/verifier_539_ok.dag >/dev/null
cat > $D/verifier_539_badname.dag <<'EOF'
module v2.test.claim.verifier_539_badname

import std.types { Bool }
import v2.test.claim.verifier_539_ok { zz_no_such_name_anywhere_539 }

test fn verifier_plant_badname() -> Bool {
  true
}
EOF
cat > $D/verifier_539_badtype.dag <<'EOF'
module v2.test.claim.verifier_539_badtype

import std.types { Bool }

test fn verifier_plant_badtype() -> Bool {
  7
}
EOF
cat > $D/verifier_539_true.dag <<'EOF'
module v2.test.claim.verifier_539_true

import std.types { Bool }

test fn verifier_plant_true_holds() -> Bool {
  true
}
EOF
git add -f $D/verifier_539_badname.dag $D/verifier_539_badtype.dag $D/verifier_539_true.dag >/dev/null
# entry that exists on disk but is NOT in the index -> resolve can never reach a subject
cat > $D/verifier_539_untracked.dag <<'EOF'
module v2.test.claim.verifier_539_untracked

import std.types { Bool }

test fn verifier_plant_untracked() -> Bool {
  true
}
EOF
mkdir -p $D/verifier_539_adir.dag
for f in $D/verifier_539_ok.dag $D/verifier_539_badname.dag $D/verifier_539_badtype.dag $D/verifier_539_true.dag $D/verifier_539_untracked.dag; do
  echo "fixture $(basename $f) bytes=$(sha $f | cut -c1-12) tracked=$(git ls-files --error-unmatch $f >/dev/null 2>&1 && echo YES || echo NO)"
done

arm() { # label -- argv...
  local label=$1; shift
  echo "-------- [$label] $(date -u +%H:%M:%S)Z"
  echo "  argv: $*"
  timeout 900 "$@" > /tmp/a_$label.out 2> /tmp/a_$label.err
  local rc=$?
  echo "  rc=$rc stdout_bytes=$(wc -c < /tmp/a_$label.out) stderr_bytes=$(wc -c < /tmp/a_$label.err)"
  echo "  --- stdout verbatim:"; head -20 /tmp/a_$label.out | sed 's/^/    O| /' | cut -c1-200
  echo "  --- stderr head:"; head -12 /tmp/a_$label.err | sed 's/^/    E| /' | cut -c1-200
  echo "  counters: PASS=$(grep -ac '^PASS ' /tmp/a_$label.out) FAIL=$(grep -ac '^FAIL ' /tmp/a_$label.out) UNOBSERVED=$(grep -ac '^UNOBSERVED ' /tmp/a_$label.out) SKIP=$(grep -ac '^SKIP ' /tmp/a_$label.out)"
  local pre=$(grep -ac 'cannot parse --entry' /tmp/a_$label.err)
  local em=$(grep -ac '^UNOBSERVED .*entry resolve failed' /tmp/a_$label.out)
  local ri=$(grep -ac '^UNOBSERVED .*resolve did not reach a verdict' /tmp/a_$label.out)
  local cd=$(grep -ac '^FAIL .*closure diagnostics in' /tmp/a_$label.out)
  echo "  exit_producer: preflight-cannot-parse=$pre entryMissing-UNOBSERVED=$em infra-UNOBSERVED=$ri closureDiagnostics-FAIL=$cd"
  if [ "$pre" != "0" ]; then echo "  !! PREFLIGHT REFUSAL: this arm never reached the split -- it is NOT evidence about the split."; fi
}
CBR="--claim-run --hermetic"
echo "=== A4 healthy control (pure true claim, tracked) -- expect rc=0, PASS=1"
arm A4 "$CB" $CBR --source-root dag --source-root src/v2 --entry $D/verifier_539_true.dag --functions verifier_plant_true_holds
echo "=== A1 a NAME error in the closure (import of a nonexistent name) -- claimed rc=1 via the ClosureDiagnostics FAIL arm"
arm A1 "$CB" $CBR --source-root dag --source-root src/v2 --entry $D/verifier_539_badname.dag --functions verifier_plant_badname
echo "=== A1b a TYPE error in the closure (Bool body returning 7) -- claimed rc=1 via the same arm"
arm A1b "$CB" $CBR --source-root dag --source-root src/v2 --entry $D/verifier_539_badtype.dag --functions verifier_plant_badtype
echo "=== A2 an entry on disk but absent from the index -- claimed rc=2 via the EntryMissing UNOBSERVED arm"
arm A2 "$CB" $CBR --source-root dag --source-root src/v2 --entry $D/verifier_539_untracked.dag --functions verifier_plant_untracked
echo "=== A2b an entry that is a DIRECTORY (the parser-first shape) -- which line produces the 2?"
arm A2b "$CB" $CBR --source-root dag --source-root src/v2 --entry $D/verifier_539_adir.dag --functions whatever_claim
echo "=== A5 MIXED: a closure-error group beside an unobserved group -- claimed rc=1 (1 beats 2, Kleene)"
arm A5 "$CB" $CBR --source-root dag --source-root src/v2 --entry $D/verifier_539_badname.dag --functions verifier_plant_badname --entry $D/verifier_539_untracked.dag --functions verifier_plant_untracked
echo "=== A5b MIXED with the groups SWAPPED -- order-independence"
arm A5b "$CB" $CBR --source-root dag --source-root src/v2 --entry $D/verifier_539_untracked.dag --functions verifier_plant_untracked --entry $D/verifier_539_badname.dag --functions verifier_plant_badname
echo "=== A5c MIXED control: a PASSING group beside the unobserved group -- expect rc=2 (nothing failed)"
arm A5c "$CB" $CBR --source-root dag --source-root src/v2 --entry $D/verifier_539_true.dag --functions verifier_plant_true_holds --entry $D/verifier_539_untracked.dag --functions verifier_plant_untracked
echo "=== A5d MIXED control: a PASSING group beside a closure-error group -- expect rc=1"
arm A5d "$CB" $CBR --source-root dag --source-root src/v2 --entry $D/verifier_539_true.dag --functions verifier_plant_true_holds --entry $D/verifier_539_badname.dag --functions verifier_plant_badname

echo "=== U1 the mixed-composition UNIT TEST, by name. 'running 0 tests' is NOT a pass, so the selection"
echo "    line is printed for every pattern."
runtest() { # label pattern
  local label=$1 pat=$2 log=/tmp/t_$label.log
  timeout 1500 cargo test -p v1-compiler --bin claim_batch "$pat" -- --nocapture > $log 2>&1
  local rc=$?
  echo "  [$label] pattern='$pat' rc=$rc selected=[$(grep -aoE 'running [0-9]+ test' $log | head -1)] result=[$(grep -a 'test result:' $log | head -1 | cut -c1-80)]"
  grep -aE '^test [a-z]' $log | head -6 | sed 's/^/    T| /' | cut -c1-160
  [ "$rc" != "0" ] && { tail -25 $log | sed 's/^/    Tt| /' | cut -c1-160; return 1; }
  return 0
}
runtest U1_mixed exit_composition_is_failed_over_unobserved_over_clean
runtest U1_absargs absolute_existing_args_parse_and_survive_resolution
runtest U1_missing nonexistent_explicit_function_refuses_before_resolution

echo "=== M1 MUTATION: collapse the split -- ClosureDiagnostics sets any_unobserved instead of any_failed"
python3 - <<'PY'
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
new=old.replace(b"timings.any_failed = true;", b"timings.any_unobserved = true;")
open(p,'wb').write(b.replace(old,new))
print("MUTATION_APPLIED: whole-balanced-match-arm replacement, one flag only")
PY
[ $? -eq 0 ] || { echo "MUTATION_NOT_APPLIED -- arms below would be a duplicate baseline, not a red"; exit 9; }
echo "  mutated sha=$(sha $CBRS | cut -c1-16) head_sha=$HEAD_CB_256" | sed 's/head_sha=\(.\{16\}\).*/head_sha=\1/'
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
arm M1a "$CB" $CBR --source-root dag --source-root src/v2 --entry $D/verifier_539_badname.dag --functions verifier_plant_badname
arm M1b "$CB" $CBR --source-root dag --source-root src/v2 --entry $D/verifier_539_badname.dag --functions verifier_plant_badname --entry $D/verifier_539_untracked.dag --functions verifier_plant_untracked
echo "  does the unit test still pass on the COLLAPSED split? (it is pure over two booleans: expect YES,"
echo "  which means it cannot guard the CLI class -- the question is whether any CLAIM catches it)"
runtest M1_test exit_composition_is_failed_over_unobserved_over_clean
echo "=== RESTORE"
cp -f /root/workspace/repo-root/tools/stage/cb_head.rs $CBRS
[ "$(sha $CBRS)" = "$HEAD_CB_256" ] && echo "  RESTORED_BYTE_IDENTICAL ${HEAD_CB_256:0:16}" || { echo "  RESTORE_FAILED got=$(sha $CBRS) want=$HEAD_CB_256"; exit 9; }
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
arm R1 "$CB" $CBR --source-root dag --source-root src/v2 --entry $D/verifier_539_badname.dag --functions verifier_plant_badname
arm R2 "$CB" $CBR --source-root dag --source-root src/v2 --entry $D/verifier_539_badname.dag --functions verifier_plant_badname --entry $D/verifier_539_untracked.dag --functions verifier_plant_untracked
echo "CB_SHA_FINAL=$(sha $CB | cut -c1-12) (compare to the pre-mutation CB_SHA above -- a rebuild of the same bytes)"
echo DONE_14539_E2
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
rm -rf $WT
