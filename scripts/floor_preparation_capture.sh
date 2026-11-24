#!/usr/bin/env bash
# SCAFFOLD: hand-written shell (Rule 8: every consumer modelled). Dissolves when
# v2.workflow.bash_emit (gunbc #5828) emits this file from a modelled script.
#
# THE FLOOR PREPARATION CAPTURE TOOL -- v2.workflow.floor_preparation_split, in one place.
#
# WHAT IT RUNS, AND WHAT IT DOES NOT RUN.
# This tool supplies logs and runs binaries. It folds NOTHING. The measurement -- classifying each
# instrument line, charging it to the preparation side or the claim side, summing it at module grain,
# and REFUSING rows it cannot read -- is the .dag: `v2.workflow.floor_preparation_split`, executed by
# the released binaries. Two executions do all the adjudicating:
#   * the CLAIMS leg runs `claim_batch --claim-run` over
#       dag/test/claim/floor/floor_preparation_split_witness.dag
#     whose claims re-derive the split from the committed rows and refuse an empty, unpaired,
#     verdict-less or unprovenanced capture (`floor_preparation_the_committed_capture_is_measured_not_authored`).
#     The harness refuses unless every one of the eighteen claims below prints its own PASS line.
#   * the REPORT leg runs the released `gunbc run --function floor_preparation_split_report`, whose
#     refusal PRINTS the fold's split String -- the documented convention at
#     src/v2/workflow/floor_cost_debt.dag ("the refusal is the correct behaviour rather than a
#     workaround"). The printed `floor-preparation-split ...` line is the model's own reading, never
#     a shell rendering of it.
# The shell's only gates are supply-integrity gates on the RUNNER: the pinned binary against
# release-bins.manifest, a non-empty log, the argv defect ("has no --function or --functions"),
# the workspace_root refusal, a panic, and that each entry reported the witness count the argv
# declared. A leg that exits 0 without its named lines is a FAIL here, not a pass.
#
# USAGE (run these from anywhere; --checkout is the git checkout that owns the sources):
#   bash scripts/floor_preparation_capture.sh run \
#     --checkout <git-checkout> --binary <claim_batch> --gunbc <released gunbc> \
#     --manifest <release-bins.manifest> --expect-sha256 <sha> --expect-gunbc-sha256 <sha> \
#     --population dag/test/claim/floor/floor_preparation_split_population.txt \
#     --outdir <dir> [--write]
#   bash scripts/floor_preparation_capture.sh report --checkout <git-checkout> --binary <claim_batch> \
#     --gunbc <released gunbc> --report-log <run-log> [--report-label <name>] [--write]
#
# WHY TWO LEGS, AND WHY BOTH ARE KEPT. The untrimmed leg enrolls every candidate AND every extra
# row; closure-authority refusals live there with their cause printed by the fold, never hidden by a
# name pattern. The graded leg enrolls only what the untrimmed run proved it could pass. A split
# computed on the trimmed closure alone understates the cost this node exists to remove, so the
# artifact states both populations and the claims relate them.
set -u

MODE=${1:-}
shift || true

CHECKOUT=""
BINARY=""
GUNBC=""
MANIFEST=""
EXPECT_SHA=""
EXPECT_GUNBC_SHA=""
POPULATION=""
OUTDIR="."
AUTHORITY=""
PROVENANCE=""
WRITE=0
EXPECT_TREE=""
REPORT_LOGS=""
REPORT_LABELS=""

while [ $# -gt 0 ]; do
  case "$1" in
    --checkout) CHECKOUT=$2; shift 2 ;;
    --binary) BINARY=$2; shift 2 ;;
    --gunbc) GUNBC=$2; shift 2 ;;
    --manifest) MANIFEST=$2; shift 2 ;;
    --expect-sha256) EXPECT_SHA=$2; shift 2 ;;
    --expect-gunbc-sha256) EXPECT_GUNBC_SHA=$2; shift 2 ;;
    --population) POPULATION=$2; shift 2 ;;
    --outdir) OUTDIR=$2; shift 2 ;;
    --authority) AUTHORITY=$2; shift 2 ;;
    --provenance) PROVENANCE="$PROVENANCE $2"; shift 2 ;;
    --expect-tree) EXPECT_TREE=$2; shift 2 ;;
    --report-log) REPORT_LOGS="$REPORT_LOGS $2"; REPORT_LABELS="$REPORT_LABELS ${REPORT_LABEL_N:-run}"; shift 2 ;;
    --report-label) REPORT_LABELS="$REPORT_LABELS $2"; shift 2 ;;
    --write) WRITE=1; shift ;;
    *) echo "CAPTURE_FAIL: unknown option $1" >&2; exit 1 ;;
  esac
done

fail() { echo "CAPTURE_FAIL: $*" >&2; exit 1; }

# The claim roster. Eighteen names, and they are the roster that runs: the witness module's header
# lists the same set and audit_roster() below refuses if the two ever disagree.
WITNESS_MODULE=dag/test/claim/floor/floor_preparation_split_witness.dag
WITNESS_ROSTER="floor_preparation_capture_grammar_is_closed,floor_preparation_split_conserves_to_the_independent_total,floor_preparation_deleting_a_costed_line_moves_the_total_by_its_own_cost,floor_preparation_the_fold_bills_the_reach_probe_fill_once,floor_preparation_committed_fill_rows_are_paired,floor_preparation_the_fold_refuses_what_it_cannot_read,floor_preparation_module_attribution_accounts_for_every_non_phase_costed_row,floor_preparation_verdict_rows_match_an_independent_prefix_scan,floor_preparation_preparation_side_dominates_the_claim_side,floor_preparation_entry_resolve_is_charged_to_the_module_it_names,floor_preparation_preparation_sides_partition_by_scale,floor_preparation_the_committed_census_is_honest,floor_preparation_the_excluded_population_is_stated_with_its_causes,floor_preparation_the_untrimmed_leg_is_the_same_floor,floor_preparation_the_split_fold_is_not_the_cost_debt_fold,floor_preparation_the_committed_capture_is_measured_not_authored,floor_preparation_the_capture_tool_runs_the_fold_and_holds_none,floor_preparation_the_roadmap_names_the_floor_fold"
WITNESS_ROSTER_COUNT=18

# Transport only: which lines of a run log are instrument lines. The MEANING of every one of these
# lines -- phase vocabulary, cost fields, side of the split, refusals -- is decided by the model, and
# claim 18 reads this file to make sure the model's vocabulary was never re-implemented here.
KEEP='^\[floor-phase\]|^\[resolve\]|^\[witness\]|^\[floor-reach-probe\]|^\[floor-shared-fill\]|^(PASS|FAIL|UNOBSERVED) '
EMPTY_SHA=e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
ARGV_DEFECT="has no --function or --functions"
BEGIN='// BEGIN GENERATED CAPTURE BLOCK'
ENDMARK='// END GENERATED CAPTURE BLOCK'

kept() { grep -E "$KEEP" "$1" > "$2" || true; }
count_of() { local n; n=$(grep -cE "^$2" "$1" || true); echo "${n:-0}"; }

# ----------------------------------------------------------------------------------------------
# THE PINNED BINARIES. Every leg runs the pinned claim_batch; every report runs the released gunbc.
# A 0-byte download that exits 0 has fooled this lane, so the guards are explicit and they are
# printed per invocation, beside the manifest line that pins the same digest.
# ----------------------------------------------------------------------------------------------

verify_pinned() {
  local kind=$1 path=$2 want=$3 manifest_line=$4
  [ -n "$path" ] || fail "$kind mode needs --${kind}"
  [ -f "$path" ] || fail "no $kind binary at $path"
  local bytes
  bytes=$(stat -c %s "$path")
  [ "$bytes" != "0" ] || fail "$kind binary is 0 bytes (a truncated download that still exits 0)"
  local sha
  sha=$(sha256sum "$path" | cut -d' ' -f1)
  [ "$sha" != "$EMPTY_SHA" ] || fail "$kind binary hashes as the empty string"
  if [ -n "$want" ]; then
    [ "$sha" = "$want" ] || fail "$kind binary sha $sha deviates from the pinned $want"
  fi
  echo "${kind^^}_SHA256=$sha"
  echo "${kind^^}_BYTES=$bytes"
  if [ "$kind" = "claim_batch" ]; then
    local manifest="${MANIFEST:-}" d d2 mh
    if [ -n "$manifest" ]; then
      [ -f "$manifest" ] || fail "no manifest at the given path: $manifest"
    else
      d=$(dirname "$path"); d2=$(dirname "$d")
      manifest="$d/release-bins.manifest"
      [ -f "$manifest" ] || manifest="$d2/release-bins.manifest"
      [ -f "$manifest" ] || manifest="$d2/bins/release-bins.manifest"
      [ -f "$manifest" ] || fail "no release-bins.manifest beside the binary, beside its parent, or in its bins directory; provenance is unverifiable"
    fi
    echo "MANIFEST=$manifest"
    mh=$(awk '$2=="claim_batch"{print $1}' "$manifest" | head -1)
    echo "MANIFEST_SHA_claim_batch=$mh"
    [ "$mh" = "$sha" ] || fail "release-bins.manifest does not pin this claim_batch"
    SHA256=$sha
  fi
}

verify_binary() {
  verify_pinned claim_batch "$BINARY" "$EXPECT_SHA"
  [ -n "$GUNBC" ] || fail "every mode needs --gunbc: the reading is printed by the released gunbc's refusal, not by this shell"
  verify_pinned gunbc "$GUNBC" "$EXPECT_GUNBC_SHA"
}

# ----------------------------------------------------------------------------------------------
# THE POPULATION. A committed text file so both legs are built from ONE artifact:
#   candidate|<module>|<claim>   eligible for the graded capture
#   extra|<module>|<claim>       enrolled only so its refusal stays in the artifact
# An entry can carry candidate rows AND extra rows; emitting one --entry block per kind made
# claim_batch resolve the same entry twice -- preparation cost for nothing, the exact expense this
# node exists to remove. So the untrimmed argv aggregates an entry's claims across kinds into a
# single block, and the per-entry witness audit sees one declared count per entry.
# ----------------------------------------------------------------------------------------------

entries_of_all() {
  awk -F'|' '
    $1 == "candidate" || $1 == "extra" {
      if (!($2 in seen)) { seen[$2] = 1; order[++n] = $2 }
      rows[$2] = (rows[$2] == "" ? $3 : rows[$2] "," $3)
    }
    END { for (i = 1; i <= n; i++) print order[i] "|" rows[order[i]] }
  ' "$POPULATION"
}

entries_of() {
  awk -F'|' -v kind="$1" '
    $1 == kind {
      if (!($2 in seen)) { seen[$2] = 1; order[++n] = $2 }
      rows[$2] = (rows[$2] == "" ? $3 : rows[$2] "," $3)
    }
    END { for (i = 1; i <= n; i++) print order[i] "|" rows[order[i]] }
  ' "$POPULATION"
}

write_argv_header() {
  local out=$1
  : > "$out"
  local root
  for root in dag src/v1 src/v2; do
    printf '%s\n%s\n' "--source-root" "$root" >> "$out"
  done
  printf '%s\n' "--claim-run" >> "$out"
}

build_argv_all() {
  local out=$1 kind entry functions
  write_argv_header "$out"
  while IFS='|' read -r entry functions; do
    [ -n "$entry" ] || continue
    [ -n "$functions" ] || fail "entry $entry enrolled with an empty --functions list"
    printf '%s\n%s\n%s\n%s\n' "--entry" "$entry" "--functions" "$functions" >> "$out"
  done < <(entries_of_all)
  echo "ARGV_WRITTEN=$out ARGS=$(wc -l < "$out")"
}

build_argv_graded() {
  local out=$1 entry functions name keep total dropped
  write_argv_header "$out"
  total=0
  dropped=0
  while IFS='|' read -r entry functions; do
    [ -n "$entry" ] || continue
    keep=""
    local oldifs=$IFS
    IFS=','
    for name in $functions; do
      if grep -qx "$name" "$PASSNAMES"; then
        keep="$keep${keep:+,}$name"
        total=$((total + 1))
      else
        dropped=$((dropped + 1))
      fi
    done
    IFS=$oldifs
    [ -n "$keep" ] || continue
    printf '%s\n%s\n%s\n%s\n' "--entry" "$entry" "--functions" "$keep" >> "$out"
  done < <(entries_of candidate)
  echo "ARGV_WRITTEN=$out ARGS=$(wc -l < "$out") GRADED_CLAIMS=$total DROPPED=$dropped"
}

# ----------------------------------------------------------------------------------------------
# SUPPLY-INTEGRITY GATES ON THE RUNNER. These check that the leg DID what the argv asked; they
# decide nothing about the measurement. The counts are printed for the artifact and never compared
# here: phase presence, fill/probe pairing, verdict presence and census honesty are claims
# (floor_preparation_the_committed_capture_is_measured_not_authored and neighbours) and the claim
# leg's rc is what refuses.
# ----------------------------------------------------------------------------------------------

audit_leg() {
  local name=$1 log=$2 argv=${3:-/dev/null}
  echo "LEG_${name}_COUNTS phase=$(count_of "$log" '\[floor-phase\]') resolve=$(count_of "$log" '\[resolve\]') fill=$(count_of "$log" '\[floor-shared-fill\]') probe=$(count_of "$log" '\[floor-reach-probe\]') pass=$(count_of "$log" 'PASS ') fail=$(count_of "$log" 'FAIL ') unobserved=$(count_of "$log" 'UNOBSERVED ') resolving=$(count_of "$log" 'claim_batch: resolving ')"
  [ -s "$log" ] || fail "leg $name produced an empty log"
  [ "$(sha256sum "$log" | cut -d' ' -f1)" != "$EMPTY_SHA" ] || fail "leg $name's log is the empty string"
  grep -q "$ARGV_DEFECT" "$log" && fail "leg $name: the binary refused with '$ARGV_DEFECT'; it measured nothing"
  grep -q "workspace_root" "$log" && fail "leg $name printed workspace_root -- not run from a git checkout?"
  grep -q "panicked at" "$log" && fail "leg $name: the binary panicked"
  grep -q '^\[floor-phase\]' "$log" || fail "leg $name never measured preparation: no [floor-phase] row in $log (a refused module index prints 'module index refused' and runs no claims)"
  grep -q 'claim_batch: resolving ' "$log" || fail "leg $name enrolled no entry (no 'claim_batch: resolving' row)"
  if [ ! -s "$argv" ]; then
    echo "LEG_${name}_WITNESS_COUNT_AUDIT=skipped (no argv supplied)"
    return 0
  fi
  local entry functions declared
  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    functions=${entry#*|}
    entry=${entry%%|*}
    declared=$(printf '%s\n' "$functions" | tr ',' '\n' | grep -c . || true)
    if ! grep -qF "claim_batch: resolving $entry ($declared witness" "$log"; then
      fail "leg $name: entry $entry never reported $declared witness(es) -- the leg did not do what the argv asked"
    fi
  done < <(awk 'prev=="--entry"{e=$0} prev=="--functions"{print e"|" $0} {prev=$0}' "$argv")
}

run_leg() {
  local name=$1 argv=$2 timeout_s=$3 log=$4 tfile=$5
  local args=() a
  while IFS= read -r a; do args+=("$a"); done < "$argv"
  echo "LEG_${name}_ARGV_COUNT=${#args[@]}"
  local i=0
  for a in "${args[@]}"; do echo "ARGV[$i]=$a"; i=$((i + 1)); done
  [ "${#args[@]}" -gt 0 ] || fail "leg $name has an empty argv"
  cd "$CHECKOUT" || fail "cannot cd to $CHECKOUT"
  local start end rc
  start=$(date +%s)
  /usr/bin/time -v -o "$tfile" timeout "$timeout_s" "$BINARY" "${args[@]}" > "$log" 2>&1
  rc=$?
  end=$(date +%s)
  echo "LEG_${name}_RC=$rc WALL=$((end - start))s"
  audit_leg "$name" "$log" "$argv"
  return $rc
}

# ----------------------------------------------------------------------------------------------
# THE TWO ADJUDICATING EXECUTIONS.
# ----------------------------------------------------------------------------------------------

audit_roster() {
  local declared
  declared=$(grep -c '^test fn ' "$CHECKOUT/$WITNESS_MODULE" || true)
  [ "$declared" = "$WITNESS_ROSTER_COUNT" ] || fail "roster drift: $WITNESS_MODULE declares $declared test fns, this tool passes $WITNESS_ROSTER_COUNT names"
  echo "ROSTER_OK declared=$declared"
}

run_witness_claims() {
  local out=$1
  audit_roster
  cd "$CHECKOUT" || fail "cannot cd to $CHECKOUT"
  local start end rc
  start=$(date +%s)
  /usr/bin/time -v -o "$out.time.txt" timeout "${FLOOR_CAPTURE_TIMEOUT_W:-1800}" "$BINARY" \
    --source-root dag --source-root src/v1 --source-root src/v2 \
    --claim-run --entry "$WITNESS_MODULE" --functions "$WITNESS_ROSTER" > "$out" 2>&1
  rc=$?
  end=$(date +%s)
  echo "CLAIMS_RC=$rc WALL=$((end - start))s OUT=$out"
  audit_leg CLAIMS "$out" /dev/null
  local passes
  passes=$(count_of "$out" 'PASS ')
  echo "CLAIMS_PASS=$passes CLAIMS_FAIL=$(count_of "$out" 'FAIL ') CLAIMS_UNOBSERVED=$(count_of "$out" 'UNOBSERVED ') EXPECTED=$WITNESS_ROSTER_COUNT"
  [ "$rc" = "0" ] || fail "the claims leg exited $rc: a claim of the fold was refused on the revision that lands"
  grep -q '^PASS floor_preparation_' "$out" || fail "the claims leg printed no floor_preparation_ PASS line at all"
  [ "$passes" = "$WITNESS_ROSTER_COUNT" ] || fail "the claims leg printed $passes PASS lines of $WITNESS_ROSTER_COUNT -- a claim that never printed is a FAIL, not a pass"
  local name
  for name in $(printf '%s' "$WITNESS_ROSTER" | tr ',' ' '); do
    grep -q "^PASS $name" "$out" || fail "the claims leg never printed PASS for $name"
  done
  echo "CLAIMS_OK all $WITNESS_ROSTER_COUNT claims printed their own PASS"
}

run_report() {
  local label=$1 out=$2
  cd "$CHECKOUT" || fail "cannot cd to $CHECKOUT"
  "$GUNBC" run \
    --source-root dag --source-root src/v1 --source-root src/v2 \
    --entry src/v2/workflow/floor_preparation_split.dag \
    --function floor_preparation_split_report > "$out" 2>&1
  local status=$?
  echo "REPORT_RC=$status LABEL=$label OUT=$out BYTES=$(stat -c %s "$out")"
  [ "$status" != "0" ] || fail "report leg $label exited 0; the report entry returns a String and must arrive as a refusal that prints it"
  grep -q "floor-preparation-split" "$out" || fail "report leg $label printed no split line: $out"
  ! grep -q "population=EMPTY" "$out" || fail "report leg $label: the committed capture is empty -- a figure-less split is not a measurement"
  ! grep -q "REFUSED" "$out" || fail "report leg $label: the fold refused the committed rows; see $out"
  grep '^floor-preparation-split' "$out"
  echo "REPORT_OK label=$label"
}

# ----------------------------------------------------------------------------------------------
# THE COMMITTED BLOCK: rows are copied verbatim from the logs (transport), quoted as string
# literals, and spliced between the authority's markers. Nothing is computed on the way.
# ----------------------------------------------------------------------------------------------

emit_list() {
  local fn=$1 file=$2
  printf 'fn %s() -> List<String> {\n  [' "$fn"
  awk '{ printf "\"%s\",", $0 }' "$file" | sed 's/,$//'
  printf '\n  ]\n}\n'
}

build_requested_rows() {
  local out=$1
  awk -F'|' '$1 == "candidate" || $1 == "extra" { print $1 "|" $2 "|" $3 }' "$POPULATION" > "$out"
}

provenance_lines() {
  local kind=$1 tfile=$2 pair
  printf 'kind=%s\n' "$kind"
  for pair in $PROVENANCE; do printf '%s\n' "$pair"; done
  local wall
  wall=$(awk '/Elapsed \(wall clock time\)/{print $NF}' "$tfile" 2>/dev/null || true)
  printf 'elapsed=%s\n' "${wall:-unknown}"
  local rss
  rss=$(awk '/Maximum resident set size/{print $NF}' "$tfile" 2>/dev/null || true)
  printf 'peak_rss_bytes=%s\n' "${rss:-unknown}"
}

splice_block() {
  local block=$1 updated
  [ -f "$AUTHORITY" ] || fail "no authority at $AUTHORITY"
  grep -qF "$BEGIN" "$AUTHORITY" || fail "the authority has no BEGIN marker to splice into"
  grep -qF "$ENDMARK" "$AUTHORITY" || fail "the authority carries no END marker; refusing to guess"
  updated=$AUTHORITY.updated
  awk -v begin="$BEGIN" -v endmark="$ENDMARK" '
    !printing && index($0, begin) == 1 { printing = 1; skip = 1; next }
    skip && index($0, endmark) == 1 { skip = 0; print "CAPTURE_BLOCK_END"; next }
    skip { next }
    !printing { print }
  ' "$AUTHORITY" > "$updated.hole"
  grep -q '^CAPTURE_BLOCK_END$' "$updated.hole" || fail "the markers did not produce a hole to fill"
  {
    awk '/^CAPTURE_BLOCK_END$/{exit} {print}' "$updated.hole"
    cat "$block"
    awk 'found {print} /^CAPTURE_BLOCK_END$/{found=1}' "$updated.hole"
  } > "$updated"
  rm -f "$updated.hole"
  if [ "$WRITE" = "1" ]; then
    cp "$AUTHORITY" "$AUTHORITY.pre-capture"
    mv "$updated" "$AUTHORITY"
    echo "AUTHORITY_WRITTEN=$AUTHORITY"
  else
    echo "DRY_RUN: pass --write to splice $(wc -l < "$block") lines into $AUTHORITY"
    rm -f "$updated"
  fi
}

emit_and_splice() {
  [ -n "$AUTHORITY" ] || AUTHORITY="$CHECKOUT/src/v2/workflow/floor_preparation_split.dag"
  mkdir -p "$OUTDIR"
  GRADED_KEPT=$OUTDIR/graded-rows.txt
  UNTRIMMED_KEPT=$OUTDIR/untrimmed-rows.txt
  PASSNAMES=$OUTDIR/passnames.txt
  kept "$GRADED_LOG" "$GRADED_KEPT"
  kept "$UNTRIMMED_LOG" "$UNTRIMMED_KEPT"
  grep '^PASS ' "$GRADED_KEPT" | sed 's/^PASS //' > "$PASSNAMES"
  [ -s "$PASSNAMES" ] || fail "the graded log has no PASS row -- the leg that was supposed to grade claims graded none"
  provenance_lines graded "$OUTDIR/graded-time.txt" > "$OUTDIR/graded-provenance.txt"
  provenance_lines untrimmed "$OUTDIR/untrimmed-time.txt" > "$OUTDIR/untrimmed-provenance.txt"
  build_requested_rows "$OUTDIR/requested-rows.txt"
  {
    emit_list floor_preparation_capture "$GRADED_KEPT"
    emit_list floor_preparation_capture_provenance "$OUTDIR/graded-provenance.txt"
    emit_list floor_preparation_untrimmed_capture "$UNTRIMMED_KEPT"
    emit_list floor_preparation_untrimmed_capture_provenance "$OUTDIR/untrimmed-provenance.txt"
    emit_list floor_preparation_requested_rows "$OUTDIR/requested-rows.txt"
  } > "$OUTDIR/block.txt"
  splice_block "$OUTDIR/block.txt"
  echo "CAPTURE_SPLICED graded_rows=$(wc -l < "$GRADED_KEPT") untrimmed_rows=$(wc -l < "$UNTRIMMED_KEPT")"
}

restore_authority() {
  if [ -f "${AUTHORITY:-}.pre-capture" ] && [ "$WRITE" != "1" ]; then
    : # dry run never wrote the authority
  elif [ -f "${AUTHORITY:-}.pre-capture" ] && [ "${RESTORE:-0}" = "1" ]; then
    mv "$AUTHORITY.pre-capture" "$AUTHORITY"
    echo "AUTHORITY_RESTORED=$AUTHORITY"
  fi
}

main() {
  case "$MODE" in
    report)
      # RE-DERIVE THE SPLIT FROM ANY LOG: copy that log's instrument rows into the authority's
      # capture block (transport), then let the two adjudicating executions decide what the log
      # measured. Without --write the authority is restored afterwards; the committed capture is
      # only ever changed by an operator-visible --write.
      [ -n "$REPORT_LOGS" ] || fail "report mode needs at least one --report-log"
      [ -n "$BINARY" ] && [ -n "$GUNBC" ] || fail "report mode needs --binary and --gunbc"
      [ -n "$POPULATION" ] || fail "report mode needs --population: the requested-row block is re-derived from the committed population, never from a log"
      verify_binary
      [ -n "$AUTHORITY" ] || AUTHORITY="$CHECKOUT/src/v2/workflow/floor_preparation_split.dag"
      mkdir -p "$OUTDIR"
      local logs log n=0
      read -r -a logs <<< "$REPORT_LOGS"
      for log in "${logs[@]}"; do
        [ -s "$log" ] || fail "report log $log is missing or empty"
        n=$((n + 1))
        kept "$log" "$OUTDIR/report-rows-$n.txt"
        [ -s "$OUTDIR/report-rows-$n.txt" ] || fail "report log $log holds no instrument rows"
      done
      local tree head
      tree=$(git -C "$CHECKOUT" rev-parse 'HEAD^{tree}')
      head=$(git -C "$CHECKOUT" rev-parse HEAD)
      echo "TREE_SHA=$tree"
      echo "HEAD_SHA=$head"
      PROVENANCE="$PROVENANCE head_sha=$head tree_sha=$tree binary_sha256=$SHA256 binary_bytes=$(stat -c %s "$BINARY")"
      provenance_lines report /dev/null > "$OUTDIR/report-provenance.txt"
      build_requested_rows "$OUTDIR/report-requested.txt"
      {
        emit_list floor_preparation_capture "$OUTDIR/report-rows-1.txt"
        emit_list floor_preparation_capture_provenance "$OUTDIR/report-provenance.txt"
        emit_list floor_preparation_untrimmed_capture "$OUTDIR/report-rows-1.txt"
        emit_list floor_preparation_untrimmed_capture_provenance "$OUTDIR/report-provenance.txt"
        emit_list floor_preparation_requested_rows "$OUTDIR/report-requested.txt"
      } > "$OUTDIR/report-block.txt"
      RESTORE=0
      [ "$WRITE" = "1" ] || RESTORE=1
      trap restore_authority EXIT
      splice_block "$OUTDIR/report-block.txt"
      run_witness_claims "$OUTDIR/report-claims.log"
      run_report report "$OUTDIR/report-print.txt"
      ;;
    run)
      [ "$WRITE" = "1" ] || fail "run mode requires --write: the claims leg and the report leg read the spliced rows from the authority on disk"
      [ -n "$POPULATION" ] || fail "run mode needs --population"
      verify_binary
      mkdir -p "$OUTDIR"
      local tree head
      tree=$(git -C "$CHECKOUT" rev-parse 'HEAD^{tree}')
      head=$(git -C "$CHECKOUT" rev-parse HEAD)
      echo "TREE_SHA=$tree"
      echo "HEAD_SHA=$head"
      if [ -n "$EXPECT_TREE" ]; then
        [ "$tree" = "$EXPECT_TREE" ] || fail "checkout tree $tree is not the expected $EXPECT_TREE"
      fi
      build_argv_all "$OUTDIR/argv-U.txt"
      run_leg U "$OUTDIR/argv-U.txt" "${FLOOR_CAPTURE_TIMEOUT_U:-5400}" "$OUTDIR/untrimmed-run.log" "$OUTDIR/untrimmed-time.txt" || fail "the untrimmed leg was refused"
      PASSNAMES=$OUTDIR/passnames.txt
      kept "$OUTDIR/untrimmed-run.log" "$OUTDIR/untrimmed-rows.tmp"
      grep '^PASS ' "$OUTDIR/untrimmed-rows.tmp" | sed 's/^PASS //' > "$PASSNAMES"
      [ -s "$PASSNAMES" ] || fail "the untrimmed leg produced no PASS rows: nothing could be graded, so nothing was measured"
      EXPECTED_PASS=$(wc -l < "$PASSNAMES")
      echo "EXPECTED_PASS_LINES=$EXPECTED_PASS"
      build_argv_graded "$OUTDIR/argv-G.txt"
      run_leg G "$OUTDIR/argv-G.txt" "${FLOOR_CAPTURE_TIMEOUT_G:-4200}" "$OUTDIR/graded-run.log" "$OUTDIR/graded-time.txt" || fail "the graded leg was refused"
      GRADED_PASS=$(count_of "$OUTDIR/graded-run.log" 'PASS ')
      echo "GRADED_PASS_LINES=$GRADED_PASS EXPECTED=$EXPECTED_PASS"
      [ "$GRADED_PASS" = "$EXPECTED_PASS" ] || fail "the graded leg printed $GRADED_PASS PASS lines where the untrimmed leg's PASS set named $EXPECTED_PASS"
      GRADED_LOG=$OUTDIR/graded-run.log
      UNTRIMMED_LOG=$OUTDIR/untrimmed-run.log
      PROVENANCE="$PROVENANCE head_sha=$head tree_sha=$tree binary_sha256=$SHA256 binary_bytes=$(stat -c %s "$BINARY")"
      emit_and_splice
      run_witness_claims "$OUTDIR/claims.log"
      run_report graded "$OUTDIR/report-print.txt"
      rm -f "$AUTHORITY.pre-capture"
      echo "TERMINAL_GATE expectations=ALL_MET claims=$WITNESS_ROSTER_COUNT graded_pass=$GRADED_PASS expected_pass=$EXPECTED_PASS"
      ;;
    *)
      echo "usage: $0 run|report [options]   (see the header block for the full invocations)" >&2
      exit 2
      ;;
  esac
}

main
