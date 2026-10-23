#!/usr/bin/env bash
# #14872 verification: cost-partition rows drawn from per-span rows; out-of-span pool_parse typed.
# The partition is computed IN-PROCESS by the driver (bin/claim_batch.rs:1083 -> cli_run::exclusive_cost_partition();
# cli_run compiles INTO the driver). So unlike #14848, the subject-under-test IS the driver: base behavior
# requires the reverse-applied build. Manager spec honored: base = same tree, cli_run.rs diff reverse-applied
# (git apply -R, guarded). Arms print each binary sha; head-driver-on-base-tree + base-driver-on-head-tree
# complete the 2x2 so neither tree nor binary alone can carry the difference.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=81b185ec82fc2fb41544f47db2fe008eea546e5e
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver872 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver872
echo 51539607552 > /sys/fs/cgroup/ver872/memory.max
RB=$(cat /sys/fs/cgroup/ver872/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver872 memory.max=$RB (read-back, bounded)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14872/head:refs/verifier/f14872 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14872)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED got=$(git rev-parse refs/verifier/f14872) -- STOP"; exit 9; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14872
H=/tmp/w872h; B=/tmp/w872b
git worktree prune >/dev/null 2>&1 || true; rm -rf $H $B
git worktree add -q --force --detach $H $HEAD_SHA || { echo WTH_FAIL; exit 9; }
git worktree add -q --force --detach $B $HEAD_SHA || { echo WTB_FAIL; exit 9; }
echo "=== build head driver: $(date -u +%H:%M:%S)Z"
(cd $H && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1) || { echo BUILD_H_FAIL; exit 9; }
mkdir -p $H/target/release && cp -f $CARGO_TARGET_DIR/release/claim_batch $H/target/release/claim_batch
echo "HEAD_BIN sha=$(sha $H/target/release/claim_batch)"
echo "=== reverse-apply cli_run.rs on base tree: $(date -u +%H:%M:%S)Z"
git -C $REPO show $HEAD_SHA:src/v1/stage0/src/cli_run.rs > /tmp/cb872_head.rs 2>/dev/null || git -C $H show HEAD:src/v1/stage0/src/cli_run.rs > /tmp/cb872_head.rs
(cd $B && git apply -R -p1 $REPO/tools/revert872.patch) || { echo APPLY_R_FAIL -- STOP; exit 9; }
cmp -s $B/src/v1/stage0/src/cli_run.rs /tmp/cb872_head.rs && { echo "REVERSE_APPLY_NOOP -- STOP"; exit 9; }
grep -q 'pub out_of_span' $B/src/v1/stage0/src/cli_run.rs && { echo "REVERSE_APPLY_INCOMPLETE -- STOP"; exit 9; }
echo REVERSE_APPLY_OK
echo "=== build base driver (shared cache): $(date -u +%H:%M:%S)Z"
(cd $B && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1) || { echo BUILD_B_FAIL; exit 9; }
mkdir -p $B/target/release && cp -f $CARGO_TARGET_DIR/release/claim_batch $B/target/release/claim_batch
echo "BASE_BIN sha=$(sha $B/target/release/claim_batch)"
CBH=$H/target/release/claim_batch; CBB=$B/target/release/claim_batch
[ "$(sha $CBH)" != "$(sha $CBB)" ] || { echo "BINS_IDENTICAL -- STOP"; exit 9; }
claim() { # tag driver tree entry fn
  local tag=$1 drv=$2 tree=$3 entry=$4 fn=$5
  cd $tree || exit 9
  timeout 900 $drv --claim-run --source-root dag --source-root src/v2 --entry $entry --functions $fn > /tmp/w872_$tag.out 2>&1
  local rc=$?
  echo "  [$tag] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/w872_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/w872_$tag.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/w872_$tag.out) BYTES=$(wc -c < /tmp/w872_$tag.out) $(date -u +%H:%M:%S)Z"
  grep -aE "^(PASS|FAIL|UNOBSERVED) $fn" /tmp/w872_$tag.out | sed 's/^/    V| /' | cut -c1-160
  grep -aE '\[cost-partition\]' /tmp/w872_$tag.out | tail -1 | sed 's/^/    CP| /' | cut -c1-420
  cd $REPO || exit 9
}
SCF=dag/test/claim/string_family_cast.dag; SCFn=string_family_cast_holds
UID_=dag/test/claim/unicode_ident_xid_grounding.dag; UIDn=unicode_ident_xid_grounding_holds
echo "=== claim arms (2x2): $(date -u +%H:%M:%S)Z"
claim scf_hH $CBH $H $SCF $SCFn
claim scf_bB $CBB $B $SCF $SCFn
claim scf_hB $CBH $B $SCF $SCFn
claim scf_bH $CBB $H $SCF $SCFn
claim uid_hH $CBH $H $UID_ $UIDn
claim uid_bB $CBB $B $UID_ $UIDn
claim uid_hB $CBH $B $UID_ $UIDn
claim uid_bH $CBB $H $UID_ $UIDn
echo "=== unit tests (module exclusive_cost_partition_law): $(date -u +%H:%M:%S)Z"
(cd $H && timeout 2400 cargo test -p v1-compiler --lib exclusive_cost_partition_law 2>&1) | tee /tmp/w872_unit.txt | grep -aE 'running [0-9]+ test|test result|^test |panicked|FAILED|error\[' | sed 's/^/  UT| /' | cut -c1-190
echo "=== mutation: out_of_span_rows clamps instead of refusing (expect exactly an_in_span_total_above... FAILED; controls pass): $(date -u +%H:%M:%S)Z"
python3 - $H/src/v1/stage0/src/cli_run.rs <<'PY' || exit 9
import re
p=__import__('sys').argv[1]; s=open(p).read()
m=re.search(r"None => \{\s*return Err\(CostAccountingRefusal::InSpanExceedsThread \{[^}]*\}\);?\s*\}", s)
assert m, "mut-anchor"
open(p,'w').write(s[:m.start()]+"None => {\n                    rows.push(CostPartitionRow { name: all.name, nanos: 0 });\n                }"+s[m.end():])
print("  MUT1 clamp applied")
PY
(cd $H && timeout 2400 cargo test -p v1-compiler --lib exclusive_cost_partition_law 2>&1) | tee /tmp/w872_mut1.txt | grep -aE 'running [0-9]+ test|test result|^test |panicked at|FAILED|error\[' | sed 's/^/  M1| /' | cut -c1-190
git -C $H checkout -q -- src/v1/stage0/src/cli_run.rs
echo DONE_14872
