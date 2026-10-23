#!/usr/bin/env bash
# #14805 breadth sweep (review REQUEST_CHANGES, DESIGN 4b meta-1): every claim in all 23 migrated witness files.
# Head-first: base pass runs ONLY non-PASS-at-head claims. Checkpointed: run locally-parsed rows, re-derive worklist, re-dispatch.
# Rows printed (the only thing that matters; parse POOLROW on my side):
#   POOLROW <tree> <file> <claim> <verdict> <rc> <peak_kb|na> <cause>
#   verdict: PASS|FAIL|UNOBSERVED   cause: timeout|oom|rc=N|absent-at-base|...
# Robustness: chunks of <=CHUNK claims per claim_batch process, per-process cgroup child with memory.max=PCAP;
# chunk timeout CHUNK_T then bisect; single-claim timeout/OOM -> UNOBSERVED row with cause, never omitted.
# Env: TREE=head|base  DEADLINE_MIN (default 42)  N (default 2)  CHUNK (default 12)  CHUNK_T (default 600)  PCAP (default 26G)
set -u
TREE=${TREE:-head}; WET=${WET:-0}; DEADLINE_MIN=${DEADLINE_MIN:-42}
WETFLAG=""; [ "$WET" = 1 ] && WETFLAG="--wet"; N=${N:-2}; CHUNK=${CHUNK:-12}; CHUNK_T=${CHUNK_T:-600}; PCAP=${PCAP:-26G}
REPO=/tmp/c4v/disp2
[ -d $REPO ] || REPO=/root/workspace/repo-root
cd $REPO || exit 9
HEAD_SHA=${HEAD_SHA:?NEED_HEAD_SHA}
BASETGZ=${BASETGZ:-tools/base14805.tgz}
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver805p 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver805p
echo 51539607552 > /sys/fs/cgroup/ver805p/memory.max
RB=$(cat /sys/fs/cgroup/ver805p/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver805p memory.max=$RB (read-back, bounded); per-proc PCAP=$PCAP N=$N CHUNK=$CHUNK CHUNK_T=$CHUNK_T deadline=${DEADLINE_MIN}m"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14805
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14805/head:refs/verifier/f14805p || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14805p)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
CB=/root/workspace/repo-root/.t14805/release/claim_batch
if [ ! -x $CB ]; then
  echo "CB_MISSING building at head $(date -u +%H:%M:%S)Z"
  git worktree prune >/dev/null 2>&1 || true; rm -rf /tmp/w805pgb; git worktree add -q --force --detach /tmp/w805pgb $HEAD_SHA || { echo WT_FAIL; exit 9; }
  cd /tmp/w805pgb && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
  be=$?; cd $REPO || exit 9
  [ $be -eq 0 ] || { echo "BUILD_FAIL $be -- STOP"; exit 9; }
fi
[ -x $CB ] || { echo "NO_BINARY -- STOP (never run without head-built CB)"; exit 9; }
echo "BUILD_COMMIT=$HEAD_SHA CB_SHA=$(sha $CB) TREE=$TREE WET=$WETFLAG"
WT=/tmp/w805p_$TREE
git worktree prune >/dev/null 2>&1 || true; rm -rf $WT; git worktree add -q --force --detach $WT $HEAD_SHA || { echo WT_FAIL; exit 9; }
if [ "$TREE" = base ]; then tar xzf $BASETGZ -C $WT || exit 9; echo BASE_OVERLAY_OK; fi
WL=tools/wl14805_${TREE}${WLFILE:+_$WLFILE}.tsv
[ -s $WL ] || { echo "EMPTY_WORKLIST $WL -- STOP"; exit 9; }
END=$(( $(date +%s) + DEADLINE_MIN*60 - 60 ))   # stop 60s before own deadline
mkdir -p /tmp/w805p_out; rm -f /tmp/w805p_out/*.out
mempeak_kb() { local cg=$1; [ -f $cg/memory.peak ] && awk '{print $1}' $cg/memory.peak || echo na; }
run_chunk() { # tag entry fns... ; prints POOLROW lines for its claims; rc 0=verdicts-for-all 1=partial/none
  local tag=$1; shift; local entry=$1; shift
  local cg=/sys/fs/cgroup/ver805p/$tag; rm -rf $cg 2>/dev/null; mkdir -p $cg 2>/dev/null || true
  echo $PCAP > $cg/memory.max 2>/dev/null || true
  local out=/tmp/w805p_out/$tag.out rc=124
  ( echo $BASHPID > $cg/cgroup.procs 2>/dev/null
    cd $WT && timeout $CHUNK_T $CB --claim-run $WETFLAG --source-root dag --source-root src/v2 --entry $entry --functions "$(printf '%s,' "$@" | sed 's/,$//')" > $out 2>&1 ) &
  local pid=$!
  # memory.current sampler (memory.peak absent on this kernel): max seen while the proc lives
  local pkmax=0
  [ -f $cg/memory.current ] || echo "SAMPLER_NO_FILE $cg/memory.current" >> /tmp/w805p_out/sampler.diag
  while kill -0 $pid 2>/dev/null; do
    if [ -f $cg/memory.current ]; then cur=$(cat $cg/memory.current 2>/dev/null || echo 0); else cur=0; fi
    [ "$cur" -gt "$pkmax" ] 2>/dev/null && pkmax=$cur
    sleep 0.5
  done
  wait $pid; rc=$?
  local pk=$((pkmax/1024)); rm -rf $cg 2>/dev/null || true
  local got=0 line nm
  for nm in "$@"; do
    line=$(grep -aE "^(PASS|FAIL|UNOBSERVED) $nm( |\$)" $out | head -1)
    if [ -n "$line" ]; then
      got=$((got+1))
      echo "POOLROW $TREE $entry $nm ${line%% *} $rc $pk -"
    else
      local cause=-; [ $rc -eq 124 ] && cause=timeout; [ $rc -eq 137 ] && cause=oom; [ $rc -gt 0 ] && [ "$cause" = - ] && cause=rc=$rc
      echo "POOLROW $TREE $entry $nm UNOBSERVED $rc $pk $cause"
    fi
  done
  [ $got -eq $# ] && return 0 || return 1
}
export -f run_chunk mempeak_kb 2>/dev/null || true
# worklist: file|claim per line; group by file
mapfile -t LINES < $WL
echo "WORKLIST $TREE entries=${#LINES[@]} start=$(date -u +%H:%M:%S)Z"
# split into per-file ordered lists
FILES=$(printf '%s\n' "${LINES[@]}" | cut -d'|' -f1 | awk '!seen[$0]++')
for f in $FILES; do
  [ $(date +%s) -ge $END ] && { echo "DEADLINE_HIT stop-file $f"; break; }
  mapfile -t CLAIMS < <(printf '%s\n' "${LINES[@]}" | grep "^$f|" | cut -d'|' -f2)
  i=0; tagn=0
  while [ $i -lt ${#CLAIMS[@]} ]; do
    [ $(date +%s) -ge $END ] && break
    # launch up to N chunks concurrently; each writes its own out file + cgroup child
    pids=(); tags=(); batches=()
    for j in $(seq 0 $((N-1))); do
      b=("${CLAIMS[@]:i+j*CHUNK:CHUNK}")
      [ ${#b[@]} -eq 0 ] && break
      tag="${TREE}_$(basename $f .dag)_${tagn}_j${j}_n${#b[@]}"
      ( run_chunk "$tag" "$f" "${b[@]}" > /tmp/w805p_out/$tag.rows 2>/dev/null ) &
      pids+=($!); tags+=("$tag"); batches+=("${#b[@]}")
    done
    for p in "${pids[@]}"; do wait $p || true; done
    for tag in "${tags[@]}"; do cat /tmp/w805p_out/$tag.rows 2>/dev/null; done
    # requeue sweep: any UNOBSERVED rows with cause=timeout/oom in multi-claim chunks -> rerun singly, once
    for tag in "${tags[@]}"; do
      grep -E ' UNOBSERVED [0-9]+ (na|[0-9]+) (timeout|oom|rc=137|rc=1)$' /tmp/w805p_out/$tag.rows 2>/dev/null | while read -r _ _ ent nm _ _ _; do
        echo "RETRY1 $TREE $ent $nm"
        run_chunk "${tag}_rt_$(echo $nm | cut -c1-24)" "$ent" "$nm"
      done
    done
    step=$((N*CHUNK)); if [ $step -gt ${#CLAIMS[@]} ]; then step=${#CLAIMS[@]}; fi; i=$((i+step)); tagn=$((tagn+1))
    true
  done
  echo "FILEDONE $TREE $f $(date -u +%H:%M:%S)Z"
done
echo DONE_POOL_$TREE $(date -u +%H:%M:%S)Z
