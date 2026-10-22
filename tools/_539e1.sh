set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14539 DELTA RE-VERIFY at aef4b0138d3 (manager scope: the exit split).
# Arms, each run BY NAME at the head with the CLI's own lines printed (exit code never establishes
# cause -- so every arm prints stdout+stderr, and I read WHICH line produced the code):
#   A1 closure/name error inside the entry   -> claimed rc=1 with a FAIL line naming 'closure diagnostics'
#   A2 nonexistent --entry                   -> claimed rc=2 ... but WHICH line? the new EntryMissing
#        arm prints 'UNOBSERVED <fn> (entry resolve failed: ...)', while the parse-phase preflight
#        (validate_explicit_functions) prints 'cannot parse --entry ... while validating functions'.
#        Both exit 2. Only the first is the split; the second means the arm is unreachable from the CLI.
#   A3 entry that parses but is outside every --source-root (route never reached the subject)
#   A4 healthy control                       -> rc=0
#   A5 mixed: a closure-error group beside an unobserved group -> claimed rc=1 (Kleene: false AND unknown)
#   U1 the unit test for the mixed composition (cargo test --bin claim_batch, by name)
#   M1 MUTATION: collapse the split (ClosureDiagnostics sets any_unobserved instead of any_failed).
#      If A1 then exits 2 and U1 still PASSES, the split's exit-1 class has no mechanical guard.
HEADSHA=aef4b0138d3077866fa601001ea9a2bb75cd19e8  # from gh pr view 14539 --json headRefOid, read programmatically (my two prior hand-written constants were both wrong-suffix fabrications from the 11-char prefix; the
# FETCH_HEAD assertion below is what caught them -- that guard is load-bearing, keep it
CBRS=src/v1/stage0/src/bin/claim_batch.rs
HEAD_CB_SHA=b74e7b5e4dee235fd37b0eb900e57bdb318f489c23930ab8b563b6f6acd32980  # recomputed locally at the
git show <head>:src/v1/stage0/src/bin/claim_batch.rs | sha256sum -- the runner printed this same value, so the
guard now agrees with the head blob instead of a stale constant from the commit I had first mis-resolved.
echo "PWD=$(pwd) RUNNER_HEAD=$(git rev-parse HEAD | cut -c1-12) (runner git is main; head gating ONLY from the FETCH_HEAD assertion) START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt539
git fetch -q origin refs/pull/14539/head 2>&1 | tail -1
[ "$(git rev-parse FETCH_HEAD)" = "$HEADSHA" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse FETCH_HEAD) want=$HEADSHA -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt539 $HEADSHA || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/wt539
[ "$(git -C $WT rev-parse HEAD)" = "$HEADSHA" ] || { echo "WORKTREE_NOT_AT_HEAD"; exit 9; }
echo "WORKTREE_HEAD=$(git -C $WT rev-parse HEAD)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
[ "$(sha $WT/$CBRS)" = "$HEAD_CB_SHA" ] && echo "SUBJECT_OK claim_batch.rs ${HEAD_CB_SHA:0:16}" || { echo "SUBJECT_DRIFT claim_batch.rs got=$(sha $WT/$CBRS)"; exit 9; }
cp -f $WT/$CBRS tools/stage/cb_head.rs 2>/dev/null || mkdir -p tools/stage && cp -f $WT/$CBRS tools/stage/cb_head.rs
[ "$(sha tools/stage/cb_head.rs)" = "$HEAD_CB_SHA" ] || { echo "PLANT_SOURCE_NOT_HEAD_BYTES"; exit 9; }

export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14539 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
CB=/root/workspace/repo-root/.t14539/release/claim_batch
[ -x $CB ] || { echo "NO BINARY - nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha $CB | cut -c1-12) BUILT_AT $(date -u +%H:%M:%S)Z"
cd $WT
mkdir -p /tmp/s539
# fixtures: healthy module, an entry importing a name that does not exist, an entry with a type error
printf 'module v.verif.split_ok { }\ndata ok_value: Int = 42\nfn ok_claim() -> Bool { true }\n' > /tmp/s539/split_ok.dag
printf 'module v.verif.split_bad_name { }\nimport v.verif.split_ok { zz_no_such_name_at_all }\nfn bad_name_claim() -> Bool { true }\n' > /tmp/s539/split_bad_name.dag
printf 'module v.verif.split_bad_type { }\nfn bad_type_claim() -> Bool { 7 }\n' > /tmp/s539/split_bad_type.dag
printf 'module v.verif.split_outside { }\nfn outside_claim() -> Bool { true }\n' > /tmp/s539/split_outside.dag
for f in split_ok split_bad_name split_bad_type split_outside; do echo "fixture $f $(sha /tmp/s539/$f.dag | cut -c1-12)"; done

arm() { # label expect_rc  ... args after --
  local label=$1; shift
  echo "-------- [$label] $(date -u +%H:%M:%S)Z argv: $*"
  timeout 900 "$@" > /tmp/a_$label.out 2> /tmp/a_$label.err
  local rc=$?
  echo "  rc=$rc stdout_bytes=$(wc -c < /tmp/a_$label.out) stderr_bytes=$(wc -c < /tmp/a_$label.err)"
  echo "  --- stdout (verdict lines verbatim, first 25):"; head -25 /tmp/a_$label.out | sed 's/^/    O| /' | cut -c1-200
  echo "  --- stderr (first 18):"; head -18 /tmp/a_$label.err | sed 's/^/    E| /' | cut -c1-200
  echo "  --- classifier: FAIL_lines=$(grep -ac '^FAIL ' /tmp/a_$label.out) PASS_lines=$(grep -ac '^PASS ' /tmp/a_$label.out) UNOBSERVED_lines=$(grep -ac '^UNOBSERVED ' /tmp/a_$label.out)"
  echo "      closure-diagnostics-line=$(grep -ac 'closure diagnostics in' /tmp/a_$label.out) entry-resolve-failed-line=$(grep -ac 'entry resolve failed' /tmp/a_$label.out) preflight-cannot-parse-line=$(grep -ac 'cannot parse --entry' /tmp/a_$label.err)"
  echo "  exit_producer: $(if grep -q 'closure diagnostics in' /tmp/a_$label.out; then echo 'SPLIT: FAIL via ClosureDiagnostics arm'; elif grep -q 'entry resolve failed' /tmp/a_$label.out; then echo 'SPLIT: UNOBSERVED via EntryMissing arm'; elif grep -q 'cannot parse --entry' /tmp/a_$label.err; then echo 'PREFLIGHT: parse-phase refusal (NOT the split)'; else echo 'other -- read the dump above'; fi)"
}
echo "=== A1 closure error (bad name in the entry's closure) -- claimed rc=1 + FAIL line"
arm A1 "$CB" --claim-run --source-root /tmp/s539 --entry /tmp/s539/split_bad_name.dag --functions bad_name_claim
echo "=== A1b closure error via a TYPE error in the entry -- claimed rc=1 + FAIL line"
arm A1b "$CB" --claim-run --source-root /tmp/s539 --entry /tmp/s539/split_bad_type.dag --functions bad_type_claim
echo "=== A2 nonexistent --entry -- claimed rc=2; which line produced it?"
arm A2 "$CB" --claim-run --source-root /tmp/s539 --entry /tmp/s539/no_such_entry_at_all.dag --functions whatever_claim
echo "=== A2b --entry is a DIRECTORY (the unit test's own EntryMissing shape) -- claimed rc=2, EntryMissing arm"
mkdir -p /tmp/s539/adir.dag
arm A2b "$CB" --claim-run --source-root /tmp/s539 --entry /tmp/s539/adir.dag --functions whatever_claim
echo "=== A3 entry parses but lives outside every --source-root (route never reached the subject)"
arm A3 "$CB" --claim-run --source-root dag --entry /tmp/s539/split_outside.dag --functions outside_claim
echo "=== A4 healthy control -- claimed rc=0"
arm A4 "$CB" --claim-run --source-root /tmp/s539 --entry /tmp/s539/split_ok.dag --functions ok_claim
echo "=== A5 MIXED: closure-error group beside the outside-root (unobserved) group -- claimed rc=1 (1 beats 2)"
arm A5 "$CB" --claim-run --source-root /tmp/s539 --source-root dag --entry /tmp/s539/split_bad_name.dag --functions bad_name_claim --entry /tmp/s539/split_outside.dag --functions outside_claim
echo "=== A5b MIXED control ordering: unobserved group FIRST, closure error second -- same rc=1 expected (order-independence)"
arm A5b "$CB" --claim-run --source-root dag --source-root /tmp/s539 --entry /tmp/s539/split_outside.dag --functions outside_claim --entry /tmp/s539/split_bad_name.dag --functions bad_name_claim

echo "=== U1 the mixed-composition UNIT TEST, by name. A filter that selects NOTHING is not a pass,"
echo "    so the 'running N test' line is printed and asserted."
runtest() { # label pattern
  local label=$1 pat=$2 log=/tmp/t_$1.log
  timeout 1500 cargo test -p v1-compiler --bin claim_batch "$pat" -- --nocapture > $log 2>&1
  local rc=$?
  echo "  [$label] pattern='$pat' rc=$rc selected=$(grep -aoE 'running [0-9]+ test' $log | head -1) result=[$(grep -a 'test result:' $log | head -1 | cut -c1-90)]"
  grep -aE '^(test |running|error|warning: unused)' $log | head -8 | sed 's/^/    T| /' | cut -c1-170
  if [ "$rc" != "0" ]; then tail -20 $log | sed 's/^/    Tt| /' | cut -c1-170; fi
}
runtest U1_mixed exit_composition_is_failed_over_unobserved_over_clean
runtest U1b_all_unit_tests ""     # whole bin suite: proves every unit test in the file is green at head

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
print("MUTATION_APPLIED whole-expression replacement of the flag assignment only")
PY
[ $? -eq 0 ] || { echo "MUTATION_NOT_APPLIED -- the arms below would be a duplicate baseline, not a red"; exit 9; }
echo "  mutated file sha=$(sha $WT/$CBRS | cut -c1-16) (head=$(echo $HEAD_CB_SHA | cut -c1-16))"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
arm M1a "$CB" --claim-run --source-root /tmp/s539 --entry /tmp/s539/split_bad_name.dag --functions bad_name_claim
arm M1b "$CB" --claim-run --source-root /tmp/s539 --source-root dag --entry /tmp/s539/split_bad_name.dag --functions bad_name_claim --entry /tmp/s539/split_outside.dag --functions outside_claim
echo "  does the unit test still pass on the collapsed split? (it is pure over two booleans, so expect YES -> it cannot guard the CLI class)"
cargo test -p v1-compiler --bin claim_batch exit_composition_is_failed_over_unobserved_over_clean 2>&1 | tail -6 | sed 's/^/    T| /'
echo "=== RESTORE"
cp -f /root/workspace/repo-root/tools/stage/cb_head.rs $WT/$CBRS
[ "$(sha $WT/$CBRS)" = "$HEAD_CB_SHA" ] && echo "  RESTORED_BYTE_IDENTICAL ${HEAD_CB_SHA:0:16}" || { echo "  RESTORE_FAILED got=$(sha $WT/$CBRS)"; exit 9; }
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
arm R1 "$CB" --claim-run --source-root /tmp/s539 --entry /tmp/s539/split_bad_name.dag --functions bad_name_claim
echo "CB_SHA_FINAL=$(sha $CB | cut -c1-12)"
echo DONE_14539_E1
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
rm -rf $WT
