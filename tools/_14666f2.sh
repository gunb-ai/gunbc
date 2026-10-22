set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=5a4840b9d70d3c33904236ec27eb12f6097dcc69
cd $REPO
git fetch --force origin $HEADSHA 2>&1 | tail -1
git worktree add -f /tmp/wt666f2 $HEADSHA 2>&1 | tail -1
cd /tmp/wt666f2
CG=/sys/fs/cgroup/ver666f2; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs; echo $BASHPID >&12 || echo "NOTE: fd join failed ($?); relying on runner ExternallyBounded"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/tmp/wt666f2/target/release/gunbc
echo "BUILD_COMMIT=$(git rev-parse HEAD) GB_SHA=$(sha256sum $GB|cut -c1-12) $(date -u +%H:%M:%SZ)"
run(){ tag=$1; entry=$2;
  rm -rf /tmp/o666f2_$tag; mkdir -p /tmp/o666f2_$tag
  timeout 1200 $GB compile --source-root dag --source-root src/v2 --entry $entry --target dag --output-dir /tmp/o666f2_$tag > /tmp/f2_666_${tag}.out 2>&1; rc=$?
  echo "[$tag] rc=$rc bytes=$(wc -c < /tmp/f2_666_${tag}.out) entry=$entry $(date -u +%H:%M:%SZ)"
  echo "  amb_shape=$(grep -acE 'ambiguous anonymous record literal shape' /tmp/f2_666_${tag}.out) declref=$(grep -ac 'RustItemDeclarationRef' /tmp/f2_666_${tag}.out) wire=$(grep -ac 'GcsBucketPolicyWire' /tmp/f2_666_${tag}.out) urlfile=$(grep -ac 'url_file_acquisition' /tmp/f2_666_${tag}.out)"
  grep -anE 'ambiguous anonymous record literal' /tmp/f2_666_${tag}.out | head -8 | cut -c1-300 | sed 's/^/  E| /'; }
STO=dag/extdeps/cloud/gcp/storage.dag
REC=dag/extdeps/realization/reconcile_in_process.dag
echo "=== A: module-as-entry, storage (expect clean at head) ==="
run sto_head $STO
echo "=== B: MUT2 revert storage body to pre-fix anonymous form (expect GcpPolicy-vs-wire ambiguity APPEARS) ==="
python3 - <<'PY'
p="/tmp/wt666f2/dag/extdeps/cloud/gcp/storage.dag"
s=open(p).read()
a='body: GcpPolicy { bindings: bindings, etag: etag, version: version }'
assert s.count(a)==1, ("anchor count", s.count(a))
open(p,"w").write(s.replace(a,'body: { bindings: bindings, etag: etag, version: version }')); print("MUT2 applied")
PY
run sto_mut $STO
git checkout -- $STO
git status --porcelain -- dag/ | head -3; echo "STO_RESTORE_DIRTY_ABOVE (empty = byte-exact)"
run sto_post $STO
echo "=== C: module-as-entry, reconcile (expect clean at head) ==="
run rec_head $REC
echo "=== D: MUT1 revert reconcile to pre-fix (expect DeclarationRef-vs-RustItemDeclarationRef APPEARS) ==="
python3 - <<'PY'
p="/tmp/wt666f2/dag/extdeps/realization/reconcile_in_process.dag"
s=open(p).read()
a='import std.decl_ref { DeclarationRef, WholeDeclaration }'
b='authority: DeclarationRef { module_path: "gunbc.typed_module_cache_capacity", decl_name: "typed_module_cache_entry_cap", field: WholeDeclaration }'
assert s.count(a)==1 and s.count(b)==1, ("anchor count", s.count(a), s.count(b))
s=s.replace(a,'import std.decl_ref { WholeDeclaration }').replace(b,'authority: { module_path: "gunbc.typed_module_cache_capacity", decl_name: "typed_module_cache_entry_cap", field: WholeDeclaration }')
open(p,"w").write(s); print("MUT1 applied")
PY
run rec_mut $REC
git checkout -- $REC
git status --porcelain -- dag/ | head -3; echo "REC_RESTORE_DIRTY_ABOVE (empty = byte-exact)"
run rec_post $REC
echo DONE_14666_F2 $(date -u +%H:%M:%SZ)
