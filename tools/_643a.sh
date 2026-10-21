set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14643 slot-cap stale import owner path. Statics (local): delta base..head = 1 line (import
# owner v2.std.operation_argv -> std.operation_argv); 21 test fns at BOTH revisions; InputTextList
# really used (file lines 217,355); owner module std.operation_argv exists at head; v2.std.* files
# do NOT exist at base.
HEADSHA=deeb798e5512bc0534ef0b7dcbb300bb90c17780   # gh pr view 14643 (post-heal head)
BASESHA=201fe0bd10da30ff0eeabaf2eabfc1b42f5e02f1   # head parent = healed main tip (pre-fix state)
F=dag/test/claim/runner/runner_slot_cap_observe_route_witness.dag
echo "HEADSHA=$HEADSHA BASESHA=$BASESHA START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w643h /tmp/w643b
git fetch -q origin +refs/pull/14643/head:refs/verifier/14643 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14643)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14643) STOP"; exit 9; }
git cat-file -e "$BASESHA^{commit}" || git fetch -q origin $BASESHA
git worktree add -q --detach /tmp/w643h $HEADSHA || { echo WT_H_FAIL; exit 9; }
git worktree add -q --detach /tmp/w643b $BASESHA || { echo WT_B_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
echo "=== build claim_batch at HEAD ==="
( cd /tmp/w643h && CARGO_TARGET_DIR=/tmp/w643h/.t643h cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CBH=/tmp/w643h/.t643h/release/claim_batch; [ -x $CBH ] || { echo NO_CBH; exit 9; }
echo "CBH_SHA=$(sha $CBH | cut -c1-12)"
echo "=== build claim_batch at BASE ==="
( cd /tmp/w643b && CARGO_TARGET_DIR=/tmp/w643b/.t643b cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CBB=/tmp/w643b/.t643b/release/claim_batch; [ -x $CBB ] || { echo NO_CBB; exit 9; }
echo "CBB_SHA=$(sha $CBB | cut -c1-12)"
FNS_H=$(grep -aoE '^test fn [a-zA-Z0-9_]+' /tmp/w643h/$F | sed 's/^test fn //' | sort | paste -sd,)
FNS_B=$(grep -aoE '^test fn [a-zA-Z0-9_]+' /tmp/w643b/$F | sed 's/^test fn //' | sort | paste -sd,)
echo "fns head=$(echo $FNS_H|tr ',' '\n'|wc -l) base=$(echo $FNS_B|tr ',' '\n'|wc -l) identical=$([ "$FNS_H" = "$FNS_B" ] && echo YES || echo NO)"
mkdir -p /sys/fs/cgroup/verif64x
echo "+memory" > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
grep -qw memory /sys/fs/cgroup/cgroup.subtree_control || echo "CGROUP-NOMEM-WARN"
echo 17179869184 > /sys/fs/cgroup/verif64x/memory.max 2>/dev/null || true
[ "$(cat /sys/fs/cgroup/verif64x/memory.max 2>/dev/null)" != "max" ] && echo "CGROUP 16G ok ($(cat /sys/fs/cgroup/verif64x/memory.max))" || echo "CGROUP_FAIL arms-may-panic"

runw () { exec 11>>/sys/fs/cgroup/verif64x/cgroup.procs; echo $BASHPID >&11; exec 11>&-
  timeout 1500 $3 --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry "$1" --functions "$5" > "$4" 2>&1; echo $?; }
echo "=== A. HEAD hermetic: 21 by name, expect rc=0 21 PASS ==="
cd /tmp/w643h || exit 9
rc=$(runw $F dag $CBH /tmp/c643_head.out "$FNS_H"); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c643_head.out) FAIL=$(grep -ac '^FAIL ' /tmp/c643_head.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c643_head.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/c643_head.out | sed 's/^/    V| /' | cut -c1-160
[ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c643_head.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
echo "=== B. BASE hermetic: 21 by name, expect rc=0 21 PASS (green both sides) ==="
cd /tmp/w643b || exit 9
rc=$(runw $F dag $CBB /tmp/c643_base.out "$FNS_B"); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c643_base.out) FAIL=$(grep -ac '^FAIL ' /tmp/c643_base.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c643_base.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/c643_base.out | sed 's/^/    V| /' | cut -c1-160
echo "  base_stale_import_present=$(git show $BASESHA:$F | grep -c 'v2.std.operation_argv') (static expect 1)"
echo "=== C. MUTATION at head copy: restore stale owner path -> batch must go dark with the named cause ==="
cd /tmp/w643h || exit 9
cp $F /tmp/c643_f.orig
A=$(grep -c '^import std.operation_argv { InputTextList }$' $F)
echo "  ANCHOR=$A (expect 1)"
[ "$A" = "1" ] || { echo "  ANCHOR_BAD skip"; echo DONE_643_A; exit 0; }
sed -i 's/^import std.operation_argv { InputTextList }$/import v2.std.operation_argv { InputTextList }/' $F
rc=$(runw $F dag $CBH /tmp/c643_mut.out "$FNS_H"); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c643_mut.out) FAIL=$(grep -ac '^FAIL ' /tmp/c643_mut.out)"
echo "  named_cause=$(grep -ac "no declaration is indexed at 'v2.std.operation_argv.InputTextList'" /tmp/c643_mut.out) (PR-claimed message)"
grep -aiE 'error|refus|no declaration' /tmp/c643_mut.out | head -4 | sed 's/^/    E| /' | cut -c1-190
cp /tmp/c643_f.orig $F
cmp -s $F /tmp/c643_f.orig && echo "  RESTORE_IDENTICAL=YES" || echo "  RESTORE_IDENTICAL=NO"
git status --porcelain -- $F | sed 's/^/    PORCELAIN| /'
echo DONE_643_A
cd /root/workspace/repo-root
git worktree remove --force /tmp/w643h >/dev/null 2>&1 || true
git worktree remove --force /tmp/w643b >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
