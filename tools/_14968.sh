#!/usr/bin/env bash
# #14968 runner: head-built claim_executor + claim_batch + gunbc (all in v1-compiler package),
# base-built claim_executor (head tree minus PR diff, reverse-applied) for the dead-flag RED.
# Arms:
#  R1 head: claim_executor --required-ci --source-root dag --source-root src/v2  => rc 0, prints admitted-module-identities:
#  R2 head: cited --required-lane witnesses form                                => rc 0
#  R3 head: genuinely unknown flag                                              => rc 2 (rc2 stays reserved)
#  R4 BASE: base-built claim_executor, same cited command                       => rc 2 (the dead reference)
#  W  head: witness claims (11 fns) dry, wet if UNOBSERVED
#  C  head: gunbc test //gunbc/instruments:required-lane-resolution-census UNBOUND => rc1 by design,
#     summary + ADMITTED lines + unreached lines; guarantee_probe_corpus_witness spot-check
set -u
cd "$(dirname "$0")/.." || exit 9
mkdir -p /root/workspace/.c4logs968; exec > >(tee /root/workspace/.c4logs968/last.log) 2>&1
DEADLINE=${1:-2400}
ARMS=${2:-all}
START=$(date +%s)
HEAD_SHA=6f21ff1ced525d79d27cf408b0f0f20e1bbce5e8
CKPT=/tmp/c4v/ckpt968; mkdir -p $CKPT/jobs
echo "CKPT=$CKPT deadline=${DEADLINE}s"
sha() { sha256sum "$1" | cut -d' ' -f1; }
git fetch -q origin +refs/pull/14968/head:refs/verifier/f14968 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14968)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED got=$(git rev-parse refs/verifier/f14968) -- STOP"; exit 9; }
git cat-file -e origin/main^{commit} 2>/dev/null || git fetch -q origin main || true
BASE_SHA=$(git merge-base origin/main refs/verifier/f14968)
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
H=/tmp/w968h; B=/tmp/w968b
git worktree prune >/dev/null 2>&1 || true
elapsed () { echo $(( $(date +%s) - START )); }
# ---------- head tree + all three bins
[ -d $H ] || git worktree add -q --force --detach $H $HEAD_SHA || { echo WTH_FAIL; exit 9; }
export CARGO_TARGET_DIR=/root/workspace/.t968h
(cd $H && cargo build --release -p v1-compiler --bins 2>&1 | tail -1) || { echo BUILD_FAIL; exit 9; }
for b in claim_executor claim_batch gunbc; do
  [ -x $CARGO_TARGET_DIR/release/$b ] || { echo "MISSING_BIN $b"; exit 9; }
  echo "HEAD_BIN $b sha256=$(sha $CARGO_TARGET_DIR/release/$b)"
done
mkdir -p $H/target/release
for b in claim_executor claim_batch gunbc; do cp -f $CARGO_TARGET_DIR/release/$b $H/target/release/$b; done
echo "BUILD_COMMIT=$HEAD_SHA"
CEH=$CARGO_TARGET_DIR/release/claim_executor
CBH=$CARGO_TARGET_DIR/release/claim_batch
GBH=$CARGO_TARGET_DIR/release/gunbc
# ---------- base tree + base claim_executor
NEED_B=1; [ -s $CKPT/jobs/R4.out ] && NEED_B=0
echo "NEED_B=$NEED_B"
if [ "$NEED_B" = 1 ]; then
  [ -d $B ] || git worktree add -q --force --detach $B $HEAD_SHA || { echo WTB_FAIL; exit 9; }
  if [ ! -s $CKPT/reverse-applied ]; then
    git diff origin/main...refs/verifier/f14968 > $CKPT/pr.patch
    (cd $B && git apply -R --check $CKPT/pr.patch && git apply -R $CKPT/pr.patch && echo REVERSE_APPLY_OK) || { echo "REVERSE_APPLY_FAIL -- STOP"; exit 9; }
    (cd $B && git status --porcelain | wc -l) | grep -qv '^0$' || { echo "NOOP-REVERSE -- STOP"; exit 9; }
    touch $CKPT/reverse-applied
  fi
  ( export CARGO_TARGET_DIR=/root/workspace/.t968b; cd $B && cargo build --release -p v1-compiler --bin claim_executor 2>&1 | tail -1 ) || { echo BUILD_FAIL_BASE; exit 9; }
  CEB=/root/workspace/.t968b/release/claim_executor
  echo "BASE_BIN claim_executor sha256=$(sha $CEB)"
else
  CEB=/root/workspace/.t968b/release/claim_executor
fi
arm_cgroup () {
  local cg=/sys/fs/cgroup/ver968-$1
  rm -rf $cg 2>/dev/null || true; mkdir -p $cg || return 9
  echo 51539607552 > $cg/memory.max || return 9
  local RB; RB=$(cat $cg/memory.max)
  { [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEM_READBACK_FAIL $1 got=$RB"; return 9; }
  CGPATH=$cg
}
fns_of2 () { perl -0777 -ne 'while(/^test fn\s+([A-Za-z0-9_]+)\s*\(/mg){print "$1\n"}' $1 | sort -u; }
ce_run () { # ce_run <tag> <bin> <tree> <args...>
  local tag=$1 bin=$2 tree=$3; shift 3
  if [ "$ARMS" != all ]; then case ",$ARMS," in *",$tag,"*) ;; *) echo "ROW $tag SKIPPED-ARGLIST"; return 0;; esac; fi
  [ -s $CKPT/jobs/$tag.out ] && { echo "ROW $tag SKIPPED-CHECKPOINT"; return 0; }
  local el; el=$(elapsed)
  [ "$el" -gt "$DEADLINE" ] && { echo "DEADLINE_REACHED ${el}s before $tag"; return 1; }
  local d=/tmp/968j-$tag; rm -rf $d; mkdir -p $d
  arm_cgroup $tag || { echo "VOID $tag (mem)"; return 9; }
  local cg=$CGPATH
  ( cd $tree && exec 11>$cg/cgroup.procs && echo $BASHPID >&11 && \
    timeout 1800 $bin "$@" ) > $d/out.txt 2>&1
  local rc=$?
  echo "ROW $tag rc=$rc lines=$(wc -l < $d/out.txt) admitted=$(grep -c '^admitted-module-identities:' $d/out.txt) el=${el}s"
  head -4 $d/out.txt | sed "s|^|  $tag |"
  grep -m2 "admitted-module-identities:\|unknown\|error\|refus" $d/out.txt | sed "s|^|  $tag note: |" | head -3
  mv $d/out.txt $CKPT/jobs/$tag.out
}
claim_run () { # claim_run <tag> <tree> <bin> <mode> <entry> <fns-csv>
  local tag=$1 tree=$2 bin=$3 mode=$4 entry=$5 fns=$6
  if [ "$ARMS" != all ]; then case ",$ARMS," in *",$tag,"*) ;; *) echo "ROW $tag SKIPPED-ARGLIST"; return 0;; esac; fi
  [ -s $CKPT/jobs/$tag.out ] && { echo "ROW $tag SKIPPED-CHECKPOINT"; return 0; }
  local el; el=$(elapsed)
  [ "$el" -gt "$DEADLINE" ] && { echo "DEADLINE_REACHED ${el}s before $tag"; return 1; }
  local d=/tmp/968j-$tag; rm -rf $d; mkdir -p $d
  arm_cgroup $tag || { echo "VOID $tag (mem)"; return 9; }
  local cg=$CGPATH extra=""
  [ "$mode" = wet ] && extra=--wet
  ( cd $tree && exec 11>$cg/cgroup.procs && echo $BASHPID >&11 && \
    timeout 1800 $bin --claim-run $extra --source-root dag --source-root src/v2 --entry $entry --functions "$fns" ) > $d/out.txt 2>&1
  local rc=$?
  echo "ROW $tag tree=$(basename $tree) mode=$mode rc=$rc P=$(grep -c '^PASS ' $d/out.txt) F=$(grep -c '^FAIL ' $d/out.txt) U=$(grep -c '^UNOBSERVED ' $d/out.txt) el=${el}s"
  grep -E '^(PASS|FAIL|UNOBSERVED) ' $d/out.txt | sed "s|^|  $tag |" | head -24
  mv $d/out.txt $CKPT/jobs/$tag.out
}
rc=0
CITED="claim_executor --required-ci --source-root dag --source-root src/v2"
echo "=== R1 head cited producer $(date -u +%H:%M:%S)Z"
ce_run R1 $CEH $H --required-ci --source-root dag --source-root src/v2 || rc=$?
echo "=== R2 cited --required-lane witnesses form $(date -u +%H:%M:%S)Z"
ce_run R2 $CEH $H --required-ci --required-lane witnesses --source-root dag --source-root src/v2 || rc=$?
echo "=== R3 unknown flag rc2 $(date -u +%H:%M:%S)Z"
ce_run R3 $CEH $H --required-ci-but-typo --source-root dag --source-root src/v2 || rc=$?
echo "=== R4 BASE dead flag $(date -u +%H:%M:%S)Z"
[ -x "$CEB" ] && ce_run R4 $CEB $B --required-ci --source-root dag --source-root src/v2 || true
echo "=== W witness claims dry $(date -u +%H:%M:%S)Z"
W=dag/test/claim/required_lane_resolution_census_witness.dag
WF=$(fns_of2 $H/$W | paste -sd, -)
claim_run Wd $H $CBH dry $W "$WF" || rc=$?
echo "=== C census instrument (unbound) $(date -u +%H:%M:%S)Z"
if [ "$ARMS" != all ]; then case ",$ARMS," in *,C,*) ;; *) echo "ROW C SKIPPED-ARGLIST"; CKPT_SKIP_C=1;; esac; fi
if [ -z "${CKPT_SKIP_C:-}" ] && [ ! -s $CKPT/jobs/C.out ]; then
  ( cd $H && timeout 2400 $GBH test //gunbc/instruments:required-lane-resolution-census ) > $CKPT/C.raw 2>&1
  echo "ROW C rc=$? lines=$(wc -l < $CKPT/C.raw)"
  grep -m3 '"kind":"admitted"' $CKPT/C.raw | sed 's|^|  C |'
  grep -m3 '"kind":"unreached"' $CKPT/C.raw | sed 's|^|  C |'
  grep -m2 "summary\|unreached_count\|admitted_count" $CKPT/C.raw | sed 's|^|  C |'
  echo "  C guarantee_probe admitted=$(grep -c 'guarantee_probe_corpus_witness.*admitted\|admitted.*guarantee_probe_corpus_witness' $CKPT/C.raw) unreached=$(grep -c 'unreached.*guarantee_probe_corpus_witness\|guarantee_probe_corpus_witness.*unreached' $CKPT/C.raw) anyline=$(grep -c 'guarantee_probe_corpus_witness' $CKPT/C.raw)"
  echo "  C approval_ntfy anyline=$(grep -c 'approval_ntfy_access_readback_witness' $CKPT/C.raw)"
  cp $CKPT/C.raw $CKPT/jobs/C.out
else
  echo "ROW C SKIPPED-CHECKPOINT"
fi
echo "ALL_DONE_14968 $(date -u +%H:%M:%S)Z rc-so-far=$rc"
