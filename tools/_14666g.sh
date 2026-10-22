set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=5a4840b9d70d3c33904236ec27eb12f6097dcc69
cd $REPO
git fetch --force origin $HEADSHA 2>&1 | tail -1
git worktree add -f /tmp/wt666g $HEADSHA 2>&1 | tail -1
cd /tmp/wt666g
export CARGO_TARGET_DIR=/tmp/tf2
CG=/sys/fs/cgroup/ver666g; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs; echo $BASHPID >&12 || echo "NOTE: fd join failed ($?); runner is ExternallyBounded (BUILDBUDDY_RUN_ID=$BUILDBUDDY_RUN_ID)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/tmp/tf2/release/gunbc
echo "BUILD_COMMIT=$(git rev-parse HEAD) GB_SHA=$(sha256sum $GB|cut -c1-12) $(date -u +%H:%M:%SZ)"
MARK='ambiguous anonymous record literal shape; add a nominal type'
run(){ tag=$1; entry=$2; key=$3;
  rm -rf /tmp/g666_$tag; mkdir -p /tmp/g666_$tag
  timeout 900 $GB compile --source-root dag --source-root src/v2 --entry $entry --target dag --output-dir /tmp/g666_$tag > /tmp/g666o_${tag}.out 2>&1; rc=$?
  hits=$(grep -rl "$MARK" /tmp/g666_$tag 2>/dev/null | wc -l)
  hitf=$(grep -rl "$MARK" /tmp/g666_$tag 2>/dev/null | grep -c "$key" || true)
  echo "[$tag] rc=$rc stdout_bytes=$(wc -c < /tmp/g666o_${tag}.out) emit_files=$(find /tmp/g666_$tag -name '*.rs' | wc -l) ambmark_files=$hits ambmark_in_$key=$hitf $(date -u +%H:%M:%SZ)"
  grep -rl "$MARK" /tmp/g666_$tag 2>/dev/null | head -5 | sed 's/^/  F| /'
  grep -rn "$MARK" /tmp/g666_$tag 2>/dev/null | grep "$key" | head -3 | cut -c1-240 | sed 's/^/  L| /'; }
STO=dag/extdeps/cloud/gcp/storage.dag
REC=dag/extdeps/realization/reconcile_in_process.dag
echo "=== storage module-as-entry, EMITTED grep ==="
run sto_head $STO storage
echo "--- MUT2 revert storage body ---"
python3 - <<'PY'
p="/tmp/wt666g/dag/extdeps/cloud/gcp/storage.dag"
s=open(p).read()
a='body: GcpPolicy { bindings: bindings, etag: etag, version: version }'
assert s.count(a)==1, ("anchor count", s.count(a))
open(p,"w").write(s.replace(a,'body: { bindings: bindings, etag: etag, version: version }')); print("MUT2 applied")
PY
run sto_mut $STO storage
git checkout -- $STO; git status --porcelain -- dag/ | head -3; echo "STO_RESTORE (empty above = byte-exact)"
run sto_post $STO storage
echo "=== reconcile module-as-entry, EMITTED grep ==="
run rec_head $REC reconcile_in_process
echo "--- MUT1 revert reconcile authority literal ---"
python3 - <<'PY'
p="/tmp/wt666g/dag/extdeps/realization/reconcile_in_process.dag"
s=open(p).read()
a='import std.decl_ref { DeclarationRef, WholeDeclaration }'
b='authority: DeclarationRef { module_path: "gunbc.typed_module_cache_capacity", decl_name: "typed_module_cache_entry_cap", field: WholeDeclaration }'
assert s.count(a)==1 and s.count(b)==1, ("anchor count", s.count(a), s.count(b))
s=s.replace(a,'import std.decl_ref { WholeDeclaration }').replace(b,'authority: { module_path: "gunbc.typed_module_cache_capacity", decl_name: "typed_module_cache_entry_cap", field: WholeDeclaration }')
open(p,"w").write(s); print("MUT1 applied")
PY
run rec_mut $REC reconcile_in_process
git checkout -- $REC; git status --porcelain -- dag/ | head -3; echo "REC_RESTORE (empty above = byte-exact)"
run rec_post $REC reconcile_in_process
echo DONE_14666_G $(date -u +%H:%M:%SZ)
