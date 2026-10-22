#!/usr/bin/env bash
# #14783 E (trigger hunt): my unknown-import plant was INERT (census stayed Observed -> claims PASSed
# both head and base, void runs). Read Rust: unknown imports are silently dropped by
# fixture_imported_corpus_sources; NotRunnable comes from (a) closure fixpoint Err or (b) compile panic.
# Test 4 payloads in the RO canonical probe source at HEAD: if any flips
# the_canonical_row_taking_apply_compiles_without_admission_refusal to FAIL (exit 2), it is the trigger;
# also one payload in the GU green source. All restores byte-exact.
set -u
cd /root/workspace/repo-root 2>/dev/null || true
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
HEAD=5a92a871b872124273dd0e168628d8b76df4bc1c
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver783e 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver783e
echo 51539607552 > /sys/fs/cgroup/ver783e/memory.max
RB=$(cat /sys/fs/cgroup/ver783e/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver783e memory.max=$RB (read-back, bounded)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t783b
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w783e
git fetch -q origin +refs/pull/14783/head:refs/verifier/f783e || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f783e)" = "$HEAD" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w783e $HEAD || { echo WT_FAIL; exit 9; }
cd /tmp/w783e || exit 9
[ "$(git rev-parse HEAD)" = "$HEAD" ] || { echo WT_NOT_AT_HEAD; exit 9; }
RO=dag/test/claim/role_singleton_convergence_witness.dag
GU=dag/test/claim/guarantee_probe_corpus_witness.dag
[ "$(git cat-file blob $HEAD:$RO | sha256sum | cut -d' ' -f1)" = "$(sha $RO)" ] || { echo "NOT_HEAD_BLOB RO -- STOP"; exit 9; }
[ "$(git cat-file blob $HEAD:$GU | sha256sum | cut -d' ' -f1)" = "$(sha $GU)" ] || { echo "NOT_HEAD_BLOB GU -- STOP"; exit 9; }
cp -f $RO /tmp/ro783e_orig; cp -f $GU /tmp/gu783e_orig
CB=/root/workspace/repo-root/.t783b/release/claim_batch
[ -x $CB ] || { echo "NO BINARY (built earlier this target dir at $HEAD) -- try build"; cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1; }
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha $CB) (expect 6ebab24c8db8...)"
run1() { # $1 tag $2 entry $3 claim
  timeout 800 $CB --claim-run --source-root dag --source-root src/v2 --entry $2 --functions "$3" > /tmp/e783_${1}.out 2>&1
  local rc=$?
  echo "  [$1] rc=$rc PASS=$(grep -ac "^PASS $3\$" /tmp/e783_${1}.out) FAIL=$(grep -ac "^FAIL $3\$" /tmp/e783_${1}.out) UNOBS=$(grep -ac "^UNOBSERVED $3\$" /tmp/e783_${1}.out)"
  grep -aE "^(PASS|FAIL|UNOBSERVED) $3\$" /tmp/e783_${1}.out | sed 's/^/    V| /' | cut -c1-240
}
try() { # $1 file $2 anchor $3 payload-insert $4 tag $5 entry $6 claim
  python3 - "$1" "$2" "$3" <<'PY'
import sys
p,anchor,ins=sys.argv[1],sys.argv[2],sys.argv[3]
s=open(p).read()
assert s.count(anchor)==1, f"anchor {s.count(anchor)}"
open(p,'w').write(s.replace(anchor, anchor+ins)); print(f"  plant[{sys.argv[4] if len(sys.argv)>4 else ''}] applied")
PY
  run1 "$4" "$5" "$6"
}
echo "=== E1 unterminated string in RO canonical source (panic candidate): $(date -u +%H:%M:%S)Z"
try $RO 'data canonical_apply_source: String = "module probe_canonical_apply\n' 'let broken = "abc
' e1_ro_unterm $RO the_canonical_row_taking_apply_compiles_without_admission_refusal
cp -f /tmp/ro783e_orig $RO; [ "$(sha $RO)" = "$(sha /tmp/ro783e_orig)" ] && echo "  RESTORE=YES" || echo "  RESTORE=NO"
echo "=== E2 unbalanced brace storm:"
try $RO 'data canonical_apply_source: String = "module probe_canonical_apply\n' 'fn storm() -> Int { {{{{{{{{{{{{{1
' e2_ro_brace $RO the_canonical_row_taking_apply_compiles_without_admission_refusal
cp -f /tmp/ro783e_orig $RO; [ "$(sha $RO)" = "$(sha /tmp/ro783e_orig)" ] && echo "  RESTORE=YES" || echo "  RESTORE=NO"
echo "=== E3 module keyword omitted (entry is not a module):"
python3 - <<'PY'
p='dag/test/claim/role_singleton_convergence_witness.dag'
s=open(p).read()
o='data canonical_apply_source: String = "module probe_canonical_apply\\n'
n='data canonical_apply_source: String = "notamodule probe_canonical_apply\\n'
assert s.count(o)==1
open(p,'w').write(s.replace(o,n)); print("  plant e3 applied")
PY
run1 e3_ro_nomod $RO the_canonical_row_taking_apply_compiles_without_admission_refusal
cp -f /tmp/ro783e_orig $RO; [ "$(sha $RO)" = "$(sha /tmp/ro783e_orig)" ] && echo "  RESTORE=YES" || echo "  RESTORE=NO"
echo "=== E4 control-char payload:"
try $RO 'data canonical_apply_source: String = "module probe_canonical_apply\n' 'fn weird() -> Int { \x01\x02 1
' e4_ro_ctl $RO the_canonical_row_taking_apply_compiles_without_admission_refusal
cp -f /tmp/ro783e_orig $RO; [ "$(sha $RO)" = "$(sha /tmp/ro783e_orig)" ] && echo "  RESTORE=YES" || echo "  RESTORE=NO"
echo "=== E5 GU green source with panic candidate (claim1 =>true site):"
try $GU 'data call_duplicate_green_source: String = "module probe_call_duplicate_green\n' 'let broken = "abc
' e5_gu_unterm $GU call_duplicate_two_anonymous_positional_params_never_collide
cp -f /tmp/gu783e_orig $GU; [ "$(sha $GU)" = "$(sha /tmp/gu783e_orig)" ] && echo "  RESTORE=YES" || echo "  RESTORE=NO"
git status --porcelain | head -3
echo DONE_783E $(date -u +%H:%M:%S)Z
