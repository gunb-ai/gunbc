#!/usr/bin/env bash
# GATE INSTRUMENT (stern-bat-401, msg_79410475): "hard-diagnostic count per module over BOTH emitted
# closures (self-host and v2-native-cli), compared with main by identity, so any other module whose
# refusals your Acc surfaces is listed and dispositioned, not just source_root_eval."
#
# Why a dispatch at all: CI answered self-host for both sides, but the v2-native-cli floor step only ever
# RAN on main's green job -- the head's job died in the self-host step. A closure whose log never arrives is
# indistinguishable from one that emitted nothing, and that reads as green. So BOTH closures are measured
# here for BOTH sides by ONE method, in fresh clones at printed SHAs (never the shared runner workspace).
#
# CONTROL BUILT IN: main's self-host is re-measured here even though CI already said 0. If my local main run
# is not 0, the instrument or the method is wrong, and no head-side number from it means anything.
#
# CI's invocation is reproduced VERBATIM (witnesses.yml lines 76/104/132):
#   cargo build --release -p v1-compiler --bin gunbc
#   target/release/gunbc test //gunbc/instruments:self-host
#   target/release/gunbc test //gunbc/instruments:v2-native-cli
set -u
LOGS=${LOGS:?LOGS required}
mkdir -p $LOGS
TARGETS=${TARGETS:-"self-host v2-native-cli"}
REPO=https://github.com/gunb-ai/gunbc
HEAD_REF=${HEAD_REF:-refs/heads/session/gentle-carp-16}
BASE_REF=${BASE_REF:-refs/heads/main}

# --- every SHA resolved FROM A REF, never typed. Typed SHAs have cost this lane two dispatches. ---
HEAD_SHA=$(git ls-remote $REPO $HEAD_REF | awk '{print $1}')
MAIN_SHA=$(git ls-remote $REPO $BASE_REF | awk '{print $1}')
echo "$HEAD_SHA" > $LOGS/head_sha.txt; echo "$MAIN_SHA" > $LOGS/main_sha.txt
[ ${#HEAD_SHA} -ge 40 ] || { echo "VOID: head sha unresolved from $HEAD_REF"; exit 40; }
[ ${#MAIN_SHA} -ge 40 ] || { echo "VOID: main sha unresolved from $BASE_REF"; exit 40; }
echo "PINNED head=$HEAD_SHA"
echo "PINNED main=$MAIN_SHA"

# --- the comparator travels from its own branch by ref, and must be non-empty AND callable ---
IREF=refs/heads/scratch/rebind-census
I_SHA=$(git ls-remote $REPO $IREF | awk '{print $1}'); echo "PINNED instrument=$I_SHA"
rm -rf $LOGS/inst; git init -q $LOGS/inst
( cd $LOGS/inst && git fetch -q --depth 1 $REPO $IREF && git show FETCH_HEAD:instruments/hard_diag_by_identity.py > $LOGS/hard_diag_by_identity.py ) || { echo "VOID: comparator fetch"; exit 41; }
[ -s $LOGS/hard_diag_by_identity.py ] || { echo "VOID: comparator file empty -- a silent zero is worse than a failure"; exit 41; }
grep -q 'usage: hard_diag_by_identity.py HEAD.log MAIN.log' $LOGS/hard_diag_by_identity.py || { echo "VOID: comparator is not the two-log CLI this script calls it as"; exit 41; }
echo "comparator bytes: $(wc -c < $LOGS/hard_diag_by_identity.py)"

run_side () {
  side=$1; sha=$2; d=$LOGS/clone_$side
  rm -rf $d
  git clone -q $REPO $d >> $LOGS/clone_$side.log 2>&1 || { echo "VOID: clone failed for $side"; return 5; }
  ( cd $d && git fetch -q --depth 1 origin $sha && git checkout -q FETCH_HEAD ) >> $LOGS/clone_$side.log 2>&1 || { echo "VOID: checkout failed for $side"; return 5; }
  echo "[$side] HEAD=$(git -C $d rev-parse HEAD) tracked_clean=$(git -C $d status --porcelain | wc -l) (want 0)"
  echo "[$side] mirror seed=$(python3 -c "import hashlib,sys;print(hashlib.sha256(open(sys.argv[1],'rb').read()).hexdigest()[:16])" $d/src/v1/stage0/src/v1_compiler_infer.rs) fold=$(grep -c UnifyGenericsAcc $d/src/v1/stage0/src/v1_compiler_infer.rs) arms=$(grep -ac 'binding_is_generic_declaration_own_form' $d/src/v1/stage0/src/v1_compiler_infer.rs)"
  ( cd $d && cargo build --release -p v1-compiler --bin gunbc ) > $LOGS/build_$side.log 2>&1
  brc=$?
  echo "[$side] build rc=$brc errors=$(grep -acE '^error' $LOGS/build_$side.log) binary=$([ -f $d/target/release/gunbc ] && echo yes || echo NO)"
  { [ $brc -eq 0 ] && [ -f $d/target/release/gunbc ]; } || { echo "VOID: no gunbc for $side -- its counts would be false zeros, not greens"; return 6; }
  for target in $TARGETS; do
    ( cd $d && timeout 3000 sh -e -c 'target/release/gunbc' 'test' "//gunbc/instruments:$target" ) > $LOGS/${side}_${target}.log 2>&1
    rc=$?
    echo "[$side] $target rc=$rc lines=$(wc -l < $LOGS/${side}_${target}.log) type-mismatch=$(grep -ac 'type mismatch' $LOGS/${side}_${target}.log) emitted=$(grep -acE 'emit and build completed|closure=' $LOGS/${side}_${target}.log) (emitted=0 means the log never reached an emit -- NOT a green)"
  done
  return 0
}

run_side head "$HEAD_SHA" || echo "SIDE head ABORTED rc=$?"
run_side main "$MAIN_SHA" || echo "SIDE main ABORTED rc=$?"

# --- the comparator, once per closure, called with BOTH logs at its real CLI ---
for target in $TARGETS; do
  hl=$LOGS/head_${target}.log; ml=$LOGS/main_${target}.log
  echo "\n##### CLOSURE $target : head vs main #####"
  if [ -s "$hl" ] && [ -s "$ml" ]; then
    python3 $LOGS/hard_diag_by_identity.py "$hl" "$ml" $LOGS/cmp_${target}.hd.json
    echo "comparator rc=$? json=$( [ -s $LOGS/cmp_${target}.hd.json ] && wc -c < $LOGS/cmp_${target}.hd.json || echo MISSING)"
  else
    echo "CANNOT COMPARE $target: head log $([ -s "$hl" ] && echo present || echo MISSING/empty), main log $([ -s "$ml" ] && echo present || echo MISSING/empty). NOT reported as green."
  fi
done
echo "GATE DONE"
( cd $LOGS && tar -c head_*.log main_*.log cmp_*.hd.json head_sha.txt main_sha.txt 2>/dev/null | gzip -9 | base64 -w0 ) | sed 's/^/=== BEGIN-TAR ===\n/;s/$/\n=== END-TAR ===/'
exit 0
