#!/usr/bin/env bash
# #14956 dispatch A: ONE head-built claim_batch (PR delta origin/main...head = 11 .dag, 0 .rs => the
# subject is DATA; one driver serves every tree), static retirement scan, tree prep:
#   H = head tree; P = base tree @115143a4d8a (pre-deletion, for the importer sweep + probe RED);
#   M = head + Blocking contract (emitted_binary) repointed at a deleted file (RED);
#   V = head + AuditOnly contract (vacuity) repointed at the same deleted file (other-shape control).
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=1a2cfbedf763df63e5713d0f84d1a7b0e14dfd82
BASE_SHA=115143a4d8a1e870124e7848fa5084c3fcfe1c38
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver956 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver956
echo 51539607552 > /sys/fs/cgroup/ver956/memory.max
RB=$(cat /sys/fs/cgroup/ver956/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver956 memory.max=$RB (read-back, bounded)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14956/head:refs/verifier/f14956b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14956b)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED got=$(git rev-parse refs/verifier/f14956b) -- STOP"; exit 9; }
git cat-file -e $BASE_SHA^{commit} 2>/dev/null || git fetch -q origin $BASE_SHA || true
git cat-file -e $BASE_SHA^{commit} >/dev/null 2>&1 || { echo "BASE_SHA_UNAVAILABLE -- STOP"; exit 9; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t956
H=/tmp/w956h; P=/tmp/w956p; M=/tmp/w956m; V=/tmp/w956v
git worktree prune >/dev/null 2>&1 || true; rm -rf $H $P $M $V
git worktree add -q --force --detach $H $HEAD_SHA || { echo WTH_FAIL; exit 9; }
git worktree add -q --force --detach $P $BASE_SHA || { echo WTP_FAIL; exit 9; }
git worktree add -q --force --detach $M $HEAD_SHA || { echo WTM_FAIL; exit 9; }
git worktree add -q --force --detach $V $HEAD_SHA || { echo WTV_FAIL; exit 9; }
echo "=== build head tree: $(date -u +%H:%M:%S)Z"
(cd $H && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1) || { echo BUILD_FAIL; exit 9; }
[ -x $CARGO_TARGET_DIR/release/claim_batch ] || { echo BUILD_MISSING; exit 9; }
echo "BUILD_COMMIT=$HEAD_SHA"
echo "HEAD_BIN sha256=$(sha $CARGO_TARGET_DIR/release/claim_batch)"
# ---------- static retirement scan on the head tree (author: "grep finds no references except the annotated RFM receipt")
echo "=== SCHEDULE_LENS_SCAN head tree: $(date -u +%H:%M:%S)Z"
(cd $H && git grep -n -i "schedule_lens" -- . | sed "s|^$H/||") | tee /tmp/956scan.txt
echo "SCAN_TOTAL_LINES=$(wc -l < /tmp/956scan.txt)"
echo "=== deleted module + its witness absent at head?"
for f in src/v2/lens/schedule_lens.dag src/v2/test/claim/walk_plan_schedule_lens_test.dag; do
  [ -e $H/$f ] && echo "PRESENT_ANOMALY $f" || echo "ABSENT_OK $f"
done
echo "=== base tree HAS them?"
for f in src/v2/lens/schedule_lens.dag src/v2/test/claim/walk_plan_schedule_lens_test.dag; do
  [ -e $P/$f ] && echo "PRESENT_OK(base) $f" || echo "ABSENT_ANOMALY(base) $f"
done
# ---------- plant the mutations (M: Blocking emitted_binary; V: AuditOnly vacuity), verified by grep after patch
python3 - "$M" "$V" <<'PY'
import sys
gone_entry = 'entry: "src/v2/test/claim/gone_14956_probe.dag",'
gone_fn    = 'function: "gone_14956_probe_holds",'
def repoint(path, block, old_pair):
    src = open(path).read()
    lines = src.split("\n")
    i = next(k for k,l in enumerate(lines) if l.startswith(block))
    j = next(k for k in range(i,len(lines)) if lines[k]=="}")
    blk = "\n".join(lines[i:j+1])
    assert blk.count(old_pair)==1, "anchor count %d in %s -- STOP" % (blk.count(old_pair), path)
    blk2 = blk.replace(old_pair, gone_entry+"\n      "+gone_fn)
    lines[i:j+1] = blk2.split("\n")
    open(path,"w").write("\n".join(lines))
    return blk2
EB = "data lens_contract_emitted_binary_effect_confinement: LensContract"
EBP = 'entry: "src/v2/test/claim/emitted_binary_effect_confinement_red_control.dag",\n      function: "emitted_binary_effect_confinement_authority_points_at_host_transport",'
VC = "data lens_contract_vacuity: LensContract"
VCP = 'entry: "src/v2/test/claim/long/vacuity_consumer_witness_long.dag",\n      function: "vacuity_consumer_witness_real_test_fn_gap_stays_unknown_by_execution",'
mblk = repoint(sys.argv[1]+"/src/v2/lens/enforcement/contract.dag", EB, EBP)
vblk = repoint(sys.argv[2]+"/src/v2/lens/enforcement/contract.dag", VC, VCP)
print("=== M mutated Blocking block ==="); print(mblk)
print("=== V mutated AuditOnly block ==="); print(vblk)
PY
[ $? -eq 0 ] || { echo "MUTATION_FAIL -- STOP"; exit 9; }
for T in $M $V; do
  ls $T/src/v2/test/claim/gone_14956_probe.dag >/dev/null 2>&1 && echo "PLANT_FILE_PRESENT_ANOMALY $T" || echo "PLANT_TARGET_TRULY_ABSENT $T"
done
# head tree: neither gone file exists (same repo state as M/V minus patch)
ls $H/src/v2/test/claim/gone_14956_probe.dag >/dev/null 2>&1 && echo "GONE_PRESENT_ANOMALY" || echo "PLANT_TARGET_TRULY_ABSENT head"
echo "DISPATCH_A_DONE $(date -u +%H:%M:%S)Z"
