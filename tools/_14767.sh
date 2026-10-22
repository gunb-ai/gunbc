set -u
cd /root/workspace/repo-root 2>/dev/null || true
HEADSHA=30dc5060bea3cf15d1545f27f39bfb1f564f0cff
BASESHA=08437488ec1c8e954e14876c810241c42c3b0ad8
TST=dag/test/claim/spark/pair_serving_authority_log_real_execution.dag
HC=dag/gunbc/spark/host_commitment.dag
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt67h /tmp/wt67b
git fetch -q origin refs/pull/14767/head 2>&1 | tail -1
[ "$(git rev-parse FETCH_HEAD)" = "$HEADSHA" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse FETCH_HEAD) want=$HEADSHA -- STOP"; exit 9; }
git fetch -q origin $BASESHA 2>&1 | tail -1
[ "$(git cat-file -t $BASESHA)" = commit ] || { echo "BASE_MISSING"; exit 9; }
git worktree add -q --detach /tmp/wt67h $HEADSHA || { echo "WT_HEAD_FAIL"; exit 9; }
git worktree add -q --detach /tmp/wt67b $BASESHA || { echo "WT_BASE_FAIL"; exit 9; }
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver67 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver67
echo 51539607552 > /sys/fs/cgroup/ver67/memory.max
RB=$(cat /sys/fs/cgroup/ver67/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver67 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver67/cgroup.procs; echo $BASHPID >&11
cd /tmp/wt67h
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14767 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14767/release/claim_batch
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha256sum $CB|cut -c1-12) $(date -u +%H:%M:%SZ)"
NAMES=$(grep -oE '^test fn [A-Za-z_0-9]+' $TST | sed 's/^test fn //' | paste -sd, -)
echo "HEAD_CLAIM_COUNT=$(echo $NAMES | tr ',' '\n' | wc -l)"
echo "=== A. head: all claims by name (expect PASS=10) ==="
timeout 2400 $CB --claim-run --source-root dag --source-root src/v2 --entry $TST --functions "$NAMES" > /tmp/o67h.out 2>&1; echo "[head] rc=$?"; grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/o67h.out | sed 's/^/  | /'
echo "=== B. base 08437488ec1 same head-built binary (expect claims 1,2 FAIL; PASS=7) ==="
cd /tmp/wt67b
NB=$(grep -oE '^test fn [A-Za-z_0-9]+' $TST | sed 's/^test fn //' | paste -sd, -)
echo "BASE_CLAIM_COUNT=$(echo $NB | tr ',' '\n' | wc -l)"
timeout 2400 $CB --claim-run --source-root dag --source-root src/v2 --entry $TST --functions "$NB" > /tmp/o67b.out 2>&1; echo "[base] rc=$?"; grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/o67b.out | sed 's/^/  | /'
echo "=== C. seam additive static checks ==="
cd /tmp/wt67h
python3 - <<'PY'
hc=open("/tmp/wt67h/dag/gunbc/spark/host_commitment.dag").read().splitlines()
base=open("/tmp/wt67b/dag/gunbc/spark/host_commitment.dag").read()
import re
def body(src, fn):
    m=re.search(r'fn %s\([^)]*\)[^{]*\{' % fn, src)
    if not m: return None
    i=src.index('{', m.start()); d=0
    for j in range(i,len(src)):
        if src[j]=='{': d+=1
        elif src[j]=='}':
            d-=1
            if d==0: return src[i+1:j]
live_pairs=[("spark_host_standings_current","spark_host_standings_current_roster"),("spark_host_standings_over","spark_host_standings_over_roster")]
for live,seam in live_pairs:
    lb=body(base,live); hb=body(base,live)  # base live body
    sb=body(hc,seam)                        # head seam body (the moved implementation)
    one=body(open('/tmp/wt67h/dag/gunbc/spark/host_commitment.dag').read(),live)
    oneline=[l.strip() for l in one.splitlines() if l.strip() and not l.strip().startswith('//')]
    print(f"LIVEFN {live}: delegation_lines={len(oneline)} -> {'ONE_LINE_DELEGATION' if len(oneline)==1 else oneline}")
    # seam body == base live body modulo the two live-roster tokens
    norm=lambda s: ' '.join(s.split())
    nb=norm(hb).replace('units: units, arm_claims: arm_claims','ROSTER').replace('spark_host_standings_under_roster(units: units','spark_host_standings_under_roster(units: units')
    ab=norm(hb)
    # substitution check: seam body with live roster tokens restored equals base live body
    t=sb
    t=t.replace('units: units','units: dgx_spark_reserved_identities()').replace('arm_claims: arm_claims','arm_claims: spark_serving_arm_host_claims')
    same = norm(t)==norm(hb)
    print(f"SEAM_EQUIV {seam}: base_live_body_modulo_roster == seam_body -> {'IDENTICAL' if same else 'DIFFERS'}")
    if not same:
        import difflib
        a=norm(hb); b2=norm(t)
        for tag,i1,i2,j1,j2 in list(difflib.SequenceMatcher(None,a,b2).get_opcodes()):
            if tag!='equal': print('  OP',tag,repr(a[i1:i2])[:120],'->',repr(b2[j1:j2])[:120])
    print(f"BASE_HAS_SEAM_FNS: {('under_roster' in base)} (expect False)")
PY
echo "=== D. MUTATION: delete the delegation (live fns call seam with FIXTURE rosters); live control must FAIL ==="
cd /tmp/wt67h
python3 - <<'PY'
p="/tmp/wt67h/dag/gunbc/spark/host_commitment.dag"
s=open(p).read()
a1='  spark_host_standings_over_roster(store: store, at: at, units: dgx_spark_reserved_identities(), arm_claims: spark_serving_arm_host_claims)'
a2='  spark_host_standings_current_roster(units: dgx_spark_reserved_identities(), arm_claims: spark_serving_arm_host_claims, current: current, host_claims: host_claims)'
assert s.count(a1)==1 and s.count(a2)==1, ("anchor counts", s.count(a1), s.count(a2))
fxu='["srv9" as HostIdentity, "srv10" as HostIdentity]'
fxa='[ServingArmHostClaim { arm: glm_group_b_serving_arm, host: "srv9" as HostIdentity, basis: "fixture" as NonEmptyStr }]'
s=s.replace(a1, f'  spark_host_standings_over_roster(store: store, at: at, units: {fxu}, arm_claims: {fxa})')
s=s.replace(a2, f'  spark_host_standings_current_roster(units: {fxu}, arm_claims: {fxa}, current: current, host_claims: host_claims)')
open(p,"w").write(s); print("MUT applied (both live delegations now pass fixture rosters)")
PY
timeout 2400 $CB --claim-run --source-root dag --source-root src/v2 --entry $TST --functions "$NAMES" > /tmp/o67m.out 2>&1; echo "[mut] rc=$?"; grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/o67m.out | sed 's/^/  | /'
echo "=== E. restore + post ==="
git checkout -- $HC
git status --porcelain -- dag/gunbc/ | head -2; echo "DIRTY_ABOVE (empty = byte-exact)"
timeout 2400 $CB --claim-run --source-root dag --source-root src/v2 --entry $TST --functions "$NAMES" > /tmp/o67p.out 2>&1; echo "[post] rc=$?"; grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/o67p.out | sed 's/^/  | /'
echo DONE_14767 $(date -u +%H:%M:%SZ)
