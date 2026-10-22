set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=5a4840b9d70d3c33904236ec27eb12f6097dcc69
BASESHA=201fe0bd10da30ff0eeabaf2eabfc1b42f5e02f1
LC=dag/test/claim/ledger_row_coherence.dag
cd $REPO
export CARGO_TARGET_DIR=/tmp/tf2
git fetch --force origin $HEADSHA $BASESHA 2>&1 | tail -1
git worktree add -f /tmp/wt666h $HEADSHA 2>&1 | tail -1
cd /tmp/wt666h
CG=/sys/fs/cgroup/ver666h; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs 2>/dev/null && echo $BASHPID >&12 2>/dev/null || echo "NOTE: fd join unavailable; runner ExternallyBounded (BUILDBUDDY_RUN_ID=$BUILDBUDDY_RUN_ID)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/tmp/tf2/release/gunbc
echo "BUILD_COMMIT=$(git rev-parse HEAD) GB_SHA=$(sha256sum $GB|cut -c1-12) $(date -u +%H:%M:%SZ)"
authrun(){ tag=$1;
  rm -rf /tmp/h666_$tag; mkdir -p /tmp/h666_$tag
  # the author's EXACT body command, stdout/stderr SEPARATED
  timeout 2400 $GB compile --entry $LC --source-root dag --source-root src/v2 > /tmp/h666_${tag}.out 2> /tmp/h666_${tag}.err; rc=$?
  echo "[$tag] rc=$rc out_bytes=$(wc -c < /tmp/h666_${tag}.out) err_bytes=$(wc -c < /tmp/h666_${tag}.err) $(date -u +%H:%M:%SZ)"
  echo "  matchesNstructs=$(grep -acE 'ambiguous anonymous record literal shape matches [0-9]+ structs' /tmp/h666_${tag}.out /tmp/h666_${tag}.err | paste -sd+ - )"
  echo "  declref=$(grep -ac 'RustItemDeclarationRef' /tmp/h666_${tag}.out /tmp/h666_${tag}.err | paste -sd+ -) wire=$(grep -ac 'GcsBucketPolicyWire' /tmp/h666_${tag}.out /tmp/h666_${tag}.err | paste -sd+ -) urlfile=$(grep -ac 'url_file_entry calls itself' /tmp/h666_${tag}.out /tmp/h666_${tag}.err | paste -sd+ -)"
  echo "  --- stderr head 25 ---"; head -25 /tmp/h666_${tag}.err | cut -c1-240 | sed 's/^/  s| /'
  echo "  --- 'matches N structs' lines ---"; grep -anE 'matches [0-9]+ structs' /tmp/h666_${tag}.out /tmp/h666_${tag}.err | head -8 | cut -c1-240 | sed 's/^/  M| /'
  echo "  --- url_file_entry context ---"; grep -anE 'url_file_entry calls itself' /tmp/h666_${tag}.out /tmp/h666_${tag}.err | head -3 | cut -c1-240 | sed 's/^/  U| /'
}
echo "=== A. HEAD tree, author's exact command ==="
authrun head
echo "=== B. BASE tree (201fe0bd), SAME head-built binary ==="
git worktree remove -f /tmp/wt666hb 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt666hb; git worktree add -f --detach /tmp/wt666hb $BASESHA 2>&1 | tail -1
cd /tmp/wt666hb
git rev-parse HEAD
authrun base
echo DONE_14666_H $(date -u +%H:%M:%SZ)
