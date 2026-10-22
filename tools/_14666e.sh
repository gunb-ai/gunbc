set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=5a4840b9d70d3c33904236ec27eb12f6097dcc69
LOGF=tools/666d.log
cd $REPO
git fetch --force origin $HEADSHA 2>&1 | tail -1
git fetch --force origin session/nimble-badger-800 2>&1 | tail -1
git worktree add -f /tmp/wt666e $HEADSHA 2>&1 | tail -1
cd /tmp/wt666e
CG=/sys/fs/cgroup/ver666e; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs; echo $BASHPID >&12
flush(){ git add -f $LOGF; git commit -qm "666e progress $(date -u +%H:%M:%SZ)" >/dev/null 2>&1; git push -f origin HEAD:session/nimble-badger-800 >/dev/null 2>&1 || true; }
: > $LOGF
echo "start $(date -u +%H:%M:%SZ)" >> $LOGF; flush
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1 >> $LOGF
GB=/tmp/wt666e/target/release/gunbc
echo "BUILD_COMMIT=$(git rev-parse HEAD) GB_SHA=$(sha256sum $GB|cut -c1-12) $(date -u +%H:%M:%SZ)" >> $LOGF; flush
run(){ tag=$1; entry=$2;
  rm -rf /tmp/o666e_$tag; mkdir -p /tmp/o666e_$tag
  timeout 900 $GB compile --source-root dag --source-root src/v2 --entry $entry --target dag --output-dir /tmp/o666e_$tag > /tmp/e666_${tag}.out 2>&1; rc=$?
  { echo "[$tag] rc=$rc bytes=$(wc -c < /tmp/e666_${tag}.out) entry=$entry $(date -u +%H:%M:%SZ)"
    echo "  amb_shape=$(grep -acE 'ambiguous anonymous record literal shape' /tmp/e666_${tag}.out) declref=$(grep -ac 'RustItemDeclarationRef' /tmp/e666_${tag}.out) wire=$(grep -ac 'GcsBucketPolicyWire' /tmp/e666_${tag}.out) urlfile=$(grep -ac 'url_file_acquisition' /tmp/e666_${tag}.out)"
    grep -anE 'ambiguous anonymous record literal' /tmp/e666_${tag}.out | head -6 | cut -c1-260 | sed 's/^/  E| /'; } >> $LOGF
  flush; }
REC=dag/extdeps/realization/reconcile_in_process.dag
STO=dag/extdeps/cloud/gcp/storage.dag
echo "=== HEAD reaching routes ===" >> $LOGF; flush
run rec dag/test/claim/reconcile_in_process_cache.dag
run sto dag/gunbc/auth/gcp_iam_bootstrap.dag
echo "=== MUT1 revert reconcile (pre-fix anonymous literal; expect declref ambiguity APPEARS on rec) ===" >> $LOGF; flush
python3 - <<'PY' >> $LOGF
import sys
p="/tmp/wt666e/"+ "dag/extdeps/realization/reconcile_in_process.dag"
s=open(p).read()
a='import std.decl_ref { DeclarationRef, WholeDeclaration }'
b='authority: DeclarationRef { module_path: "gunbc.typed_module_cache_capacity", decl_name: "typed_module_cache_entry_cap", field: WholeDeclaration }'
assert s.count(a)==1 and s.count(b)==1, ("anchor count", s.count(a), s.count(b))
s=s.replace(a,'import std.decl_ref { WholeDeclaration }').replace(b,'authority: { module_path: "gunbc.typed_module_cache_capacity", decl_name: "typed_module_cache_entry_cap", field: WholeDeclaration }')
open(p,"w").write(s)
print("MUT1 applied")
PY
flush
run rec_mut dag/test/claim/reconcile_in_process_cache.dag
run sto_mut dag/gunbc/auth/gcp_iam_bootstrap.dag
echo "=== restore reconcile, apply MUT2 revert storage (expect wire ambiguity APPEARS on sto) ===" >> $LOGF; flush
git checkout -- $REC
python3 - <<'PY' >> $LOGF
p="/tmp/wt666e/dag/extdeps/cloud/gcp/storage.dag"
s=open(p).read()
a='body: GcpPolicy { bindings: bindings, etag: etag, version: version }'
assert s.count(a)==1, ("anchor count", s.count(a))
s=s.replace(a,'body: { bindings: bindings, etag: etag, version: version }')
open(p,"w").write(s)
print("MUT2 applied")
PY
flush
run sto_mut2 dag/gunbc/auth/gcp_iam_bootstrap.dag
run rec_mut2 dag/test/claim/reconcile_in_process_cache.dag
echo "=== restore storage; post-restore head re-run ===" >> $LOGF; flush
git checkout -- $STO
git diff --stat HEAD -- dag/ | tail -2 >> $LOGF
echo "POST_RESTORE_DIRTY_ABOVE (empty = byte-exact)" >> $LOGF; flush
run rec_post dag/test/claim/reconcile_in_process_cache.dag
run sto_post dag/gunbc/auth/gcp_iam_bootstrap.dag
echo DONE_14666_E $(date -u +%H:%M:%SZ) | tee -a $LOGF; flush
