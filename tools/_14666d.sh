set -euo pipefail
REPO=/root/workspace/repo-root
HEADSHA=5a4840b9d70d3c33904236ec27eb12f6097dcc69
cd $REPO
git fetch --force origin $HEADSHA 2>&1 | tail -1
git worktree add -f /tmp/wt666d $HEADSHA 2>&1 | tail -1
cd /tmp/wt666d
# bounded cgroup wrapper, dispatch-level
CG=/sys/fs/cgroup/ver666d; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs; echo $BASHPID >&12
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/tmp/wt666d/target/release/gunbc
GB_SHA=$(sha256sum $GB | cut -c1-12); BCOM=$(git rev-parse HEAD)
echo "BUILD_COMMIT=$BCOM GB_SHA=$GB_SHA built $(date -u +%H:%M:%SZ)"
run(){ # tag entry
  local tag=$1 entry=$2
  rm -rf /tmp/o666d_$tag; mkdir -p /tmp/o666d_$tag
  timeout 1500 $GB compile --source-root dag --source-root src/v2 --entry $entry --target dag --output-dir /tmp/o666d_$tag > /tmp/d666_${tag}.out 2>&1; local rc=$?
  echo "  [$tag] rc=$rc bytes=$(wc -c < /tmp/d666_${tag}.out) entry=$entry"
  echo "  amb_shape=$(grep -acE 'ambiguous anonymous record literal shape' /tmp/d666_${tag}.out) declref=$(grep -ac 'RustItemDeclarationRef' /tmp/d666_${tag}.out) wire=$(grep -ac 'GcsBucketPolicyWire' /tmp/d666_${tag}.out) urlfile=$(grep -ac 'url_file_acquisition' /tmp/d666_${tag}.out)"
  grep -anE 'ambiguous anonymous record literal|\.dag:[0-9]+.*(ambiguous|error)' /tmp/d666_${tag}.out | head -10 | cut -c1-230 | sed 's/^/  E| /'
}
echo "=== HEAD reaching routes (expect: NO ambiguity error at the two fixed sites) ==="
run rec dag/test/claim/reconcile_in_process_cache.dag
run sto dag/gunbc/auth/gcp_iam_bootstrap.dag
echo DONE_14666_D $(date -u +%H:%M:%SZ)
