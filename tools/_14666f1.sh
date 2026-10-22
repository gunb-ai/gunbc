set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=5a4840b9d70d3c33904236ec27eb12f6097dcc69
cd $REPO
git fetch --force origin $HEADSHA 2>&1 | tail -1
git worktree remove -f /tmp/wt666e 2>/dev/null || true
git worktree add -f /tmp/wt666f1 $HEADSHA 2>&1 | tail -1
cd /tmp/wt666f1
CG=/sys/fs/cgroup/ver666f1; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs; echo $BASHPID >&12
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/tmp/wt666f1/target/release/gunbc
echo "BUILD_COMMIT=$(git rev-parse HEAD) GB_SHA=$(sha256sum $GB|cut -c1-12) $(date -u +%H:%M:%SZ)"
run(){ tag=$1; entry=$2;
  rm -rf /tmp/o666f1_$tag; mkdir -p /tmp/o666f1_$tag
  timeout 1200 $GB compile --source-root dag --source-root src/v2 --entry $entry --target dag --output-dir /tmp/o666f1_$tag > /tmp/f1_666_${tag}.out 2>&1; rc=$?
  echo "[$tag] rc=$rc bytes=$(wc -c < /tmp/f1_666_${tag}.out) entry=$entry $(date -u +%H:%M:%SZ)"
  echo "  amb_shape=$(grep -acE 'ambiguous anonymous record literal shape' /tmp/f1_666_${tag}.out) declref=$(grep -ac 'RustItemDeclarationRef' /tmp/f1_666_${tag}.out) wire=$(grep -ac 'GcsBucketPolicyWire' /tmp/f1_666_${tag}.out) urlfile=$(grep -ac 'url_file_acquisition' /tmp/f1_666_${tag}.out)"
  grep -anE 'ambiguous anonymous record literal' /tmp/f1_666_${tag}.out | head -8 | cut -c1-300 | sed 's/^/  E| /'; }
echo "=== HEAD: reconcile-reaching route (expect NO ambiguity at typed_module_cache_facts) ==="
run rec_head dag/test/claim/reconcile_in_process_cache.dag
echo "=== MUT1: revert reconcile literal to pre-fix anonymous form ==="
python3 - <<'PY'
p="/tmp/wt666f1/dag/extdeps/realization/reconcile_in_process.dag"
s=open(p).read()
a='import std.decl_ref { DeclarationRef, WholeDeclaration }'
b='authority: DeclarationRef { module_path: "gunbc.typed_module_cache_capacity", decl_name: "typed_module_cache_entry_cap", field: WholeDeclaration }'
assert s.count(a)==1 and s.count(b)==1, ("anchor count", s.count(a), s.count(b))
s=s.replace(a,'import std.decl_ref { WholeDeclaration }').replace(b,'authority: { module_path: "gunbc.typed_module_cache_capacity", decl_name: "typed_module_cache_entry_cap", field: WholeDeclaration }')
open(p,"w").write(s); print("MUT1 applied")
PY
run rec_mut dag/test/claim/reconcile_in_process_cache.dag
echo "=== restore ==="
git checkout -- dag/extdeps/realization/reconcile_in_process.dag
git status --porcelain -- dag/ | head -3; echo "DIRTY_ABOVE (empty = byte-exact)"
run rec_post dag/test/claim/reconcile_in_process_cache.dag
echo DONE_14666_F1 $(date -u +%H:%M:%SZ)
