set -u
cd /root/workspace/repo-root 2>/dev/null || true
# wise-moth-507 C4-G triage runner (routed by warm-wolf-570 msg_870ffa5c; sheet msg_1399e55d).
# Usage: bash tools/_moth2.sh <OID40> <TAG> <arms-csv> [DEADLINE]
#   arms: A1d A1w A2 A3 B1 B2 C1 C2 D1 D2  (comma-separated; each SHA runs only its arms)
# Build: cargo build --release -p v1-compiler --features text_lookup_work_counter,interp_test_witness,test_hooks --bin claim_batch --bin gunbc
# Rows stream with grep-visible prefixes. rc=2 zero-verdict = VOID. UNOBSERVED printed, never omitted.
OID=$1; TAG=$2; ARMS=$3; DEADLINE=${4:-2400}
[ ${#OID} -eq 40 ] || { echo "BAD_OID(len=${#OID}) STOP"; exit 9; }
START=$(date +%s); elapsed(){ echo $(( $(date +%s) - START )); }
want(){ case ",$ARMS," in *",$1,"*) return 0;; *) echo "ROW $1 SKIPPED-ARGLIST"; return 1;; esac; }
echo "JOB=$TAG OID=$OID ARMS=$ARMS START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w$TAG
git fetch -q origin $OID 2>&1 | tail -1
git cat-file -e "$OID^{commit}" 2>/dev/null || { echo "SHA_UNFETCHABLE STOP"; exit 9; }
git worktree add -q --detach /tmp/w$TAG $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w$TAG
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo WT_MISMATCH; exit 9; }
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) TREE=$(git rev-parse HEAD^{tree})"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.mt$TAG CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --features text_lookup_work_counter,interp_test_witness,test_hooks --bin claim_batch --bin gunbc 2>&1 | tail -2
CB=/root/workspace/repo-root/.mt$TAG/release/claim_batch
GB=/root/workspace/repo-root/.mt$TAG/release/gunbc
[ -x $CB ] && [ -x $GB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
CB_SHA=$(sha $CB); GB_SHA=$(sha $GB)
echo "CB_SHA=$CB_SHA GB_SHA=$GB_SHA built=$(elapsed)s (features: text_lookup_work_counter,interp_test_witness,test_hooks)"
arm_cgroup () {
  # READBACK MUST BE $cg/memory.max (the child), not the root -- smoke-tested bug class.
  local cg=/sys/fs/cgroup/moth-$TAG-$1
  CGPATH=""; CGREADBACK="unbound"; CGFLAG=0
  [ -w /sys/fs/cgroup/cgroup.subtree_control ] && echo "+memory" > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
  rm -rf $cg 2>/dev/null || true
  if mkdir -p "$cg" 2>/dev/null && echo 51539607552 > $cg/memory.max 2>/dev/null; then
    local RB; RB=$(cat $cg/memory.max 2>/dev/null || true)
    if [ -n "$RB" ] && [ "$RB" != "max" ]; then CGPATH=$cg; CGREADBACK=$RB; CGFLAG=1; fi
  fi
  [ "$CGFLAG" = "1" ] || echo "  CGNOTE $1 UNBOUND (no cgroup delegation on this runner instance)"
  return 0
}
run_bound () { # <out> <err> <timeout-s> <cmd...>
  local o=$1 e=$2; shift 2
  if [ "${CGFLAG:-0}" = "1" ]; then
    ( cd $WT && exec 11>$CGPATH/cgroup.procs && echo $BASHPID >&11 && timeout "$@" ) > $o 2>$e
  else
    ( cd $WT && timeout "$@" ) > $o 2>$e
  fi
}
claim_arm () { # <name> <dry|wet> <file> <fns-csv>
  local name=$1 mode=$2 file=$3 fns=$4
  want $name || return 0
  [ -f "$file" ] || { echo "ROW $name FILE_ABSENT_AT_SHA $file"; return 0; }
  local fnflag="--functions $fns"
  if [ "$fns" = "ALL" ]; then
    fnflag=""
    local cnt; cnt=$(grep -acE '^test fn ' "$file"); echo "  ROW $name ALL fns at SHA: count=$cnt"
  else
    for f in $(echo $fns | tr ',' ' '); do
      grep -qaE "^test fn[[:space:]]+${f}[[:space:]]*\(" "$file" || { echo "ROW $name FN_NOT_IN_FILE $f ($file) -- refuse, not a result"; return 0; }
    done
  fi
  local el; el=$(elapsed); [ "$el" -gt "$DEADLINE" ] && { echo "DEADLINE_REACHED ${el}s before $name"; return 1; }
  local wf=""; [ "$mode" = wet ] && wf="--wet"
  arm_cgroup $name
  run_bound /tmp/m2_$name.out /tmp/m2_$name.err 1200 $CB --claim-run $wf --source-root dag --source-root src/v2 --source-root src/v1 --entry "$file" $fnflag
  local rc=$?
  [ "$rc" = "124" ] && { echo "ROW $name rc=124 TIMEOUT UNOBSERVED (not a result)"; return 0; }
  local NP NF NU; NP=$(grep -ac '^PASS ' /tmp/m2_$name.out); NF=$(grep -ac '^FAIL ' /tmp/m2_$name.out); NU=$(grep -ac '^UNOBSERVED ' /tmp/m2_$name.out)
  echo "ROW $name rc=$rc mode=$mode PASS=$NP FAIL=$NF UNOBS=$NU cgroup=$CGREADBACK el=${el}s"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/m2_$name.out | sed 's/^/  V| /' | cut -c1-200
  grep -aiE 'refus|error|panic|not indexed|not found' /tmp/m2_$name.err | head -4 | sed 's/^/  E| /' | cut -c1-200
  [ "$rc" = "0" ] && [ "$((NP+NF+NU))" = "0" ] && echo "  ROW $name !! EXIT 0 HAVING OBSERVED NOTHING -- VOID, not a pass"
  [ "$rc" = "2" ] && [ "$((NP+NF+NU))" = "0" ] && echo "  ROW $name !! rc=2 zero-verdict VOID: $(head -3 /tmp/m2_$name.err | tr '\n' ' ' | cut -c1-200)"
  return 0
}
compile_arm () { # <name> <entry-file> <outdir>
  local name=$1 file=$2 od=$3
  want $name || return 0
  [ -f "$file" ] || { echo "ROW $name FILE_ABSENT_AT_SHA $file"; return 0; }
  local el; el=$(elapsed); [ "$el" -gt "$DEADLINE" ] && { echo "DEADLINE_REACHED ${el}s before $name"; return 1; }
  rm -rf $od; mkdir -p $od
  arm_cgroup $name
  run_bound /tmp/m2_$name.out /tmp/m2_$name.err 1650 $GB compile --source-root dag --source-root src/v2 --entry "$file" --output-dir $od --target rust
  local rc=$?
  [ "$rc" = "124" ] && { echo "ROW $name rc=124 TIMEOUT UNOBSERVED (not a result)"; return 0; }
  local nout; nout=$(find $od -type f | wc -l)
  echo "ROW $name rc=$rc outfiles=$nout cgroup=$CGREADBACK el=${el}s"
  tail -6 /tmp/m2_$name.out | sed 's/^/  O| /' | cut -c1-200
  grep -aiE 'not found|refus|error' /tmp/m2_$name.err | head -6 | sed 's/^/  E| /' | cut -c1-200
  return 0
}
RA=dag/test/claim/roadmap/roadmap_authority.dag
RT=dag/test/claim/roadmap/roadmap_authority_test.dag
RJ=dag/test/claim/react_jsx_emit.dag
HV=dag/test/claim/roadmap/roadmap_hierarchy_view_witness.dag
ALL4=witness_browse_page_jsx_exact_bytes,witness_fn_name_in_output,witness_hostile_href_rejected,witness_safe_page_emits_ok
claim_arm A1d dry  "$RA" witness_accepted_parents_precede_dependents
claim_arm A1w wet  "$RA" witness_accepted_parents_precede_dependents
claim_arm A2  dry  "$RJ" "$ALL4"
compile_arm A3 "$HV" /tmp/out-a3
claim_arm A2w wet "$RJ" "$ALL4"
compile_arm B1 "$HV" /tmp/out-b1
claim_arm B2 dry "$RJ" "$ALL4"
compile_arm C1 "$HV" /tmp/out-c1
claim_arm C2 dry "$RJ" "$ALL4"
GHA=dag/test/claim/gha_expression_roundtrip.dag
AU=dag/test/claim/action_use_admission_witness.dag
claim_arm A4d dry "$AU" every_executed_workflow_file_has_a_census_claim_and_no_claim_names_a_missing_file
claim_arm A4w wet "$AU" every_executed_workflow_file_has_a_census_claim_and_no_claim_names_a_missing_file
claim_arm A5  dry "$GHA" ALL
claim_arm E1  dry "$GHA" ALL
claim_arm E2  dry "$GHA" ALL
claim_arm D1d dry "$RT" witness_accepted_parents_precede_dependents
claim_arm D1w wet "$RT" witness_accepted_parents_precede_dependents
claim_arm D2d dry "$RT" witness_accepted_parents_precede_dependents
claim_arm D2w wet "$RT" witness_accepted_parents_precede_dependents
echo "JOB=$TAG DONE $(date -u +%H:%M:%S)Z BUILD_COMMIT=$OID CB_SHA=$CB_SHA GB_SHA=$GB_SHA el=$(elapsed)s"
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
