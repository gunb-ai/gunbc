set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=$1
BASESHA=$2
LEDGER=dag/test/claim/ledger_row_coherence.dag
SCHED=dag/test/wet_evidence.dag
cd $REPO
git fetch --force origin $HEADSHA $BASESHA 2>&1 | tail -1
git worktree add -f /tmp/wt14671b $HEADSHA 2>&1 | tail -1
cd /tmp/wt14671b
git rev-parse HEAD | grep -q "^$HEADSHA" || { echo "PR_REF_MOVED/GUARD FAIL"; exit 9; }
export CARGO_TARGET_DIR=$REPO/.t14671b
CG=/sys/fs/cgroup/ver14671b; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs 2>/dev/null && echo $BASHPID >&12 2>/dev/null || echo "NOTE: fd join unavailable; runner ExternallyBounded (BUILDBUDDY_RUN_ID=$BUILDBUDDY_RUN_ID)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=$REPO/.t14671b/release/claim_batch
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha256sum $CB|cut -c1-12) $(date -u +%H:%M:%SZ)"
FNS=$(grep -oE '^test fn [A-Za-z_0-9]+' $LEDGER | awk '{print $3}' | paste -sd, -)
echo "CLAIM_COUNT=$(echo $FNS | tr ',' '\n' | wc -l)"
timeout 2400 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $LEDGER --functions "$FNS" > /tmp/c14671b.out 2>&1; echo "[head] rc=$?"
grep -ac '^PASS ' /tmp/c14671b.out | sed 's/^/PASS_COUNT=/'; grep -ac '^FAIL ' /tmp/c14671b.out | sed 's/^/FAIL_COUNT=/'
grep -aE '^FAIL ' /tmp/c14671b.out | head -5 | cut -c1-220 | sed 's/^/  F| /'
echo "=== schedule diff vs base: exactly the PR's own 18 rows ==="
git diff $BASESHA $HEADSHA -- $SCHED > /tmp/sched671.diff; echo "diff_lines=$(wc -l < /tmp/sched671.diff)"
grep -c '^+.*claim_identity' /tmp/sched671.diff | sed 's/^/ADDED_identity_rows=/'
grep -c '^-.*claim_identity' /tmp/sched671.diff | sed 's/^/REMOVED_identity_rows=/'
grep -A2 '^+' /tmp/sched671.diff | grep -c 'WetScheduledClaim {}' | sed 's/^/EMPTY_RECORDS=/' || true
# duplicate field probe inside the added rows
grep '^+' /tmp/sched671.diff | grep -oE 'expectation: [A-Za-z_]+' | sort | uniq -c | awk '$1>1' | head -3 | sed 's/^/DUP_FIELD| /'
python3 - <<'PY'
import re
s=open('/tmp/wt14671b/'+ "dag/test/wet_evidence.dag").read()
ids=re.findall(r'claim_identity: "([^"]+)"', s)
dup=[i for i in set(ids) if ids.count(i)>1]
print("SCHED_TOTAL_ROWS=", len(ids), "DUPES=", dup[:5])
PY
echo DONE_14671B $(date -u +%H:%M:%SZ)
