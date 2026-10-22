set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14539 attempt 4 -- the discriminating experiment, INSIDE the repo (attempts 1-3 are void and
# disclosed: brace-module grammar; then paths outside the workspace panic at
# cli_run.rs:2696 repo_relative_path_normalized, and an absolute outside path is refused at parse).
# Open question from attempt 2: the entry-with-a-bad-import shape gave rc=1 with ZERO stdout, i.e. the
# process aborted before the per-group loop, so the PR's claimed 'FAIL ... (closure diagnostics in'
# line was never printed. Two hypotheses:
#   H1 the whole-tree corpus parses EVERY .dag under the source roots -> an unreachable bad island
#      still aborts the run;
#   H2 the corpus is entry-closure-scoped -> an island is not parsed, and pointing --entry AT the
#      island is the shape that reaches the ClosureDiagnostics arm.
# P6 vs P7 discriminate: same roots, same files; only --entry differs.
OID=aef4b0138d3077866fa601001ea9a2bb75cd19e8  # gh pr view 14539 --json headRefOid (embedded; runner has no gh)
echo "HEADSHA_from_gh=$OID"
echo "PWD=$(pwd) RUNNER_HEAD=$(git rev-parse HEAD | cut -c1-12) START $(date -u +%H:%M:%S)Z"
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
HEAD_CB_256=$(sha tools/stage/cb_head.rs); echo "HEAD_CB_256=${HEAD_CB_256:0:16}"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14539 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14539/release/claim_batch
[ -x $CB ] || { echo "NO BINARY - nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha $CB | cut -c1-12) BUILT_AT $(date -u +%H:%M:%S)Z"
cd $WT

V=src/v2/test/claim/verifier539
mkdir -p $V
cat > $V/okmod.dag <<'EOF'
module v2.test.claim.verifier539.okmod

test fn ok_helper() -> Bool {
  true
}
EOF
cat > $V/good_entry.dag <<'EOF'
module v2.test.claim.verifier539.good_entry

import v2.test.claim.verifier539.okmod { ok_helper }

test fn good_entry_holds() -> Bool {
  ok_helper()
}
EOF
cat > $V/island_badname.dag <<'EOF'
module v2.test.claim.verifier539.island_badname

import v2.test.claim.verifier539.okmod { zz_not_a_real_name_539 }

test fn island_claim() -> Bool {
  true
}
EOF
cat > $V/island_badtype.dag <<'EOF'
module v2.test.claim.verifier539.island_badtype

test fn island_type_claim() -> Bool {
  7
}
EOF
git add -f $V >/dev/null
echo "planted: $(ls $V | tr '\n' ' ')  tracked=$(git ls-files $V | wc -l)/4"

arm() { # label roots... -- entry fn [entry fn ...]   (builds --entry/--functions pairs itself)
  local label=$1; shift
  local roots=""
  while [ "${1:-}" != "--" ]; do roots="$roots $1"; shift; done; shift
  local argv=()
  while [ "$#" -ge 2 ]; do argv+=(--entry "$1" --functions "$2"); shift 2; done
  set -- ${argv[@]+"${argv[@]}"}
  echo "-------- [$label] roots:$roots $(date -u +%H:%M:%S)Z"
  echo "  argv: $CB --claim-run --hermetic$roots $*" | cut -c1-230
  timeout 300 "$CB" --claim-run --hermetic $roots "$@" > /tmp/a_$label.out 2> /tmp/a_$label.err
  local rc=$?
  if [ "$rc" = "124" ]; then echo "  rc=124 TIMEOUT -- EXCLUDED, not a result"; return 0; fi
  echo "  rc=$rc stdout_bytes=$(wc -c < /tmp/a_$label.out) stderr_bytes=$(wc -c < /tmp/a_$label.err)"
  echo "  --- stdout:"; sed 's/^/    O| /' /tmp/a_$label.out | head -14 | cut -c1-190
  echo "  --- stderr tail:"; tail -8 /tmp/a_$label.err | sed 's/^/    Et| /' | cut -c1-190
  echo "  counters: PASS=$(grep -ac '^PASS ' /tmp/a_$label.out) FAIL=$(grep -ac '^FAIL ' /tmp/a_$label.out) UNOBSERVED=$(grep -ac '^UNOBSERVED ' /tmp/a_$label.out)"
  local pre=$(grep -ac 'cannot parse --entry' /tmp/a_$label.err) ab=$(grep -ac 'claim_batch: ' /tmp/a_$label.err)
  local em=$(grep -ac '^UNOBSERVED .*entry resolve failed' /tmp/a_$label.out)
  local ri=$(grep -ac '^UNOBSERVED .*resolve did not reach a verdict' /tmp/a_$label.out)
  local cd=$(grep -ac '^FAIL .*closure diagnostics in' /tmp/a_$label.out)
  echo "  exit_producer: preflight=$pre cli-refusal=$ab entryMissing=$em infra=$ri closureDiagnostics-FAIL=$cd"
  [ "$cd" != "0" ] && echo "  >>> CLOSURE-DIAGNOSTICS ARM REACHED (the PR's claimed finding arm)"
  [ "$em" != "0" ] && echo "  >>> ENTRY-MISSING ARM REACHED (the PR's claimed gap arm)"
  if [ "$cd" = "0" ] && [ "$em" = "0" ] && [ "$ri" = "0" ] && [ "$rc" != "0" ]; then echo "  !! NONZERO EXIT, NO TYPED VERDICT LINE: an earlier layer produced the code; read stderr tail."; fi
  return 0
}
W="--source-root dag --source-root src/v2 --source-root src/v1"
echo "=== P1 control: healthy entry, healthy import"
arm P1 $W -- $V/good_entry.dag good_entry_holds
echo "=== P6 DISCRIMINATOR: healthy entry, with an UNREACHABLE bad island in the same roots"
arm P6 $W -- $V/good_entry.dag good_entry_holds
echo "=== P7 DISCRIMINATOR: --entry AT the island (same roots, same files)"
arm P7 $W -- $V/island_badname.dag island_claim
echo "=== P7b island with a TYPE error instead of a name error"
arm P7b $W -- $V/island_badtype.dag island_type_claim
echo "=== P8 mixed: healthy group + island group (the PR's mixed-composition claim, exit 1 beats 2)"
arm P8 $W -- $V/good_entry.dag good_entry_holds $V/island_badname.dag island_claim
echo "=== P8b same, order swapped"
arm P8b $W -- $V/island_badname.dag island_claim $V/good_entry.dag good_entry_holds
echo "=== P9 the claimed gap arm: --entry that does not exist, INSIDE the repo"
arm P9 $W -- $V/no_such_file_539.dag whatever_claim
echo "=== P10 the claimed gap arm: --entry is a directory inside the repo"
mkdir -p $V/adir.dag
arm P10 $W -- $V/adir.dag whatever_claim
echo "=== P11 gap + finding together (claimed rc=1)"
arm P11 $W -- $V/island_badname.dag island_claim $V/no_such_file_539.dag whatever_claim

echo "=== U1 unit tests by name (selection line printed; 'running 0 tests' is not a pass)"
runtest() {
  local label=$1
  local pat=$2
  local log=/tmp/t_$label.log
  timeout 900 cargo test -p v1-compiler --bin claim_batch "$pat" -- --nocapture > $log 2>&1
  local rc=$?
  echo "  [$label] pattern='$pat' rc=$rc selected=[$(grep -aoE 'running [0-9]+ test' $log | head -1)] result=[$(grep -a 'test result:' $log | head -1 | cut -c1-80)]"
  grep -aE '^test [a-z]' $log | head -4 | sed 's/^/    T| /' | cut -c1-160
  [ "$rc" != "0" ] && tail -16 $log | sed 's/^/    Tt| /' | cut -c1-170
  return 0
}
runtest U1_mixed exit_composition_is_failed_over_unobserved_over_clean

echo "=== M1 MUTATION (only meaningful if P7 reached the arm): ClosureDiagnostics counts as UNOBSERVED"
python3 - <<'PYC'
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
    print("M1_ANCHOR_NOT_UNIQUE count=%d" % b.count(old)); sys.exit(3)
open(p,'wb').write(b.replace(old, old.replace(b"timings.any_failed = true;", b"timings.any_unobserved = true;")))
print("M1_APPLIED: whole-balanced-match-arm, one flag assignment only")
PYC
if [ $? -ne 0 ]; then echo "M1_NOT_APPLIED"; exit 9; fi
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
arm M1a $W -- $V/island_badname.dag island_claim
arm M1b $W -- $V/island_badname.dag island_claim $V/no_such_file_539.dag whatever_claim
runtest M1_test exit_composition_is_failed_over_unobserved_over_clean

echo "=== RESTORE + post-restore"
cp -f /root/workspace/repo-root/tools/stage/cb_head.rs $WT/$CBRS
if [ "$(sha $WT/$CBRS)" = "$HEAD_CB_256" ]; then echo "  RESTORED_BYTE_IDENTICAL ${HEAD_CB_256:0:16}"; else echo "  RESTORE_FAILED"; exit 9; fi
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
arm R1 $W -- $V/island_badname.dag island_claim
echo DONE_14539_E4
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
