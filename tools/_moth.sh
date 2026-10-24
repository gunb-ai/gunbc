set -u
cd /root/workspace/repo-root 2>/dev/null || true
# Generic head-built claim runner (wise-moth-507 C4-G triage, routed by warm-wolf-570 msg_870ffa5c).
# Usage: bash tools/_moth.sh <OID40> <JOBTAG> <wet|dry> <file::fns[;file::fns...]> [DEADLINE_SECS]
#   <OID40>          exact 40-hex SHA (the printed culprit SHA)
#   <JOBTAG>         short label for artifacts/worktree
#   wet|dry          --wet ONLY for ENROLLED witnesses; unenrolled get dry (else rc=2 VOID)
#   claim-sets       semicolon-separated SETS, each FILE::fn1,fn2  (ONE claim-set per --functions!)
# Rows stream to stdout with grep-visible prefixes. rc=2 with zero verdict lines = VOID, never a pass.
OID=$1; TAG=$2; WET=$3; SETS=$4; DEADLINE=${5:-2400}
[ ${#OID} -eq 40 ] || { echo "BAD_OID(len=${#OID}) -- need full 40-hex; STOP"; exit 9; }
START=$(date +%s)
elapsed(){ echo $(( $(date +%s) - START )); }
echo "JOB=$TAG OID=$OID WET=$WET START $(date -u +%H:%M:%S)Z DEADLINE=${DEADLINE}s"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w$TAG
git fetch -q origin $OID 2>&1 | tail -1
git cat-file -e "$OID^{commit}" 2>/dev/null || { echo "SHA_UNFETCHABLE -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w$TAG $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w$TAG
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo WT_MISMATCH; exit 9; }
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) TREE=$(git rev-parse HEAD^{tree})"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t$TAG CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -2
CB=/root/workspace/repo-root/.t$TAG/release/claim_batch
GB=/root/workspace/repo-root/.t$TAG/release/gunbc
[ -x $CB ] || { echo "NO claim_batch BINARY -- nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "CB_SHA=$(sha $CB) built at BUILD_COMMIT above el=$(elapsed)s"
[ -x $GB ] && echo "GB_SHA=$(sha $GB)"
arm_cgroup () {
  local cg=/sys/fs/cgroup/moth-$1
  rm -rf $cg 2>/dev/null || true; mkdir -p $cg || return 9
  echo 51539607552 > $cg/memory.max || return 9
  local RB; RB=$(cat /sys/fs/cgroup/memory.max)
  { [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEM_READBACK_FAIL $1 got=$RB"; return 9; }
  CGPATH=$cg; CGREADBACK=$RB
}
IFS=';' read -ra ARR <<< "$SETS"
for S in "${ARR[@]}"; do
  FILE=${S%%::*}; FNS=${S##*::}
  [ "$FILE" != "$S" ] || { echo "  SET_BAD_FORMAT $S (need file::fns)"; continue; }
  [ -f "$FILE" ] || { echo "  FILE_ABSENT_AT_SHA $FILE"; continue; }
  MISSING=""
  for f in $(echo $FNS | tr ',' ' '); do
    grep -qaE "^test fn[[:space:]]+${f}[[:space:]]*\(" "$FILE" || MISSING="$MISSING $f"
  done
  if [ -n "$MISSING" ]; then echo "  FN_NOT_IN_FILE_AT_SHA file=$FILE missing:$MISSING (refusing to run; unknown fn -> rc=2 VOID)"; continue; fi
  el=$(elapsed)
  if [ "$el" -gt "$DEADLINE" ]; then echo "DEADLINE_REACHED ${el}s before $FILE::$FNS -- REMAINING SETS UNOBSERVED"; break; fi
  N=$(echo $FNS | tr ',' '\n' | wc -l)
  echo "-------- [$TAG] run $FILE ($N fns) $(date -u +%H:%M:%S)Z el=${el}s"
  if [ "$WET" = "wet" ]; then WFLAG="--wet"; else WFLAG=""; fi
  arm_cgroup $TAG-$(echo "$FILE$FNS" | md5sum | cut -c1-8) || { echo "  VOID $FILE (mem) -- cannot bound memory"; continue; }
  cg=$CGPATH
  ( cd $WT && exec 11>$cg/cgroup.procs && echo $BASHPID >&11 && \
    timeout 1500 $CB --claim-run $WFLAG --source-root dag --source-root src/v2 --source-root src/v1 --entry "$FILE" --functions "$FNS" ) > /tmp/moth_${TAG}_out.out 2>/tmp/moth_${TAG}_err.out
  rc=$?
  [ "$rc" = "124" ] && { echo "  rc=124 TIMEOUT -- UNOBSERVED, not a result"; continue; }
  NP=$(grep -ac '^PASS ' /tmp/moth_${TAG}_out.out); NF=$(grep -ac '^FAIL ' /tmp/moth_${TAG}_out.out); NU=$(grep -ac '^UNOBSERVED ' /tmp/moth_${TAG}_out.out)
  echo "  rc=$rc PASS=$NP FAIL=$NF UNOBS=$NU out=$(wc -c < /tmp/moth_${TAG}_out.out)B cgroup=moth-$TAG(mem.max=$CGREADBACK)"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/moth_${TAG}_out.out | sed 's/^/  V| /' | cut -c1-200
  [ "$rc" = "0" ] && [ "$NP" = "0" ] && echo "  !! EXIT 0 HAVING OBSERVED NOTHING -- VOID, not a pass"
  [ "$rc" = "2" ] && [ "$((NP+NF+NU))" = "0" ] && echo "  !! rc=2 zero-verdict VOID: $(head -3 /tmp/moth_${TAG}_err.out | tr '\n' ' ' | cut -c1-200)"
done
echo "JOB=$TAG DONE $(date -u +%H:%M:%S)Z BUILD_COMMIT=$OID CB_SHA=$(sha $CB) el=$(elapsed)s"
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
