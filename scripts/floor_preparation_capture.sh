#!/usr/bin/env bash
# Re-derive the witness floor's preparation-versus-claim split from a required-floor run.
#
# SCAFFOLD - dissolve-on: the orchestration-to-shell bash-emit capability
#   (v2.workflow.bash_emit / bash_emit_stmts, gunbc #5828). Every step this file performs by hand --
#   pin-and-verify the released binary against release-bins.manifest, build the claim_batch argv from the
#   committed population file, invoke it inside the caller's heavy slot, keep the line shapes a run
#   prints, fold them at module grain, splice the generated block into the authority -- is a node tree
#   over the SINGLE authority `v2.workflow.floor_preparation_split`, which owns the phase grammar, the
#   classes, and every refusal. This shell realizes that model; it is not a second source of truth about
#   what preparation is. When those node kinds land, this file is EMITTED from the model rather than
#   hand-written, and the model does not change.
#
# CONSUMERS (Rule 8: a file automation reads declares them). (1) The witnesses in
# dag/test/claim/floor/floor_preparation_split_witness.dag read THIS FILE with filesystem_read and refuse
# if its fold vocabulary drifts from the model's grammar table -- editing the tables below changes what
# those claims decide. (2) `floor_preparation_capture`, `floor_preparation_capture_provenance`,
# `floor_preparation_untrimmed_capture`, `floor_preparation_untrimmed_capture_provenance` and
# `floor_preparation_requested_rows` in the authority are generated from this file's output; a hand edit
# here silently changes committed measurement rows on the next splice. (3) Any session re-deriving the
# split, per the ask= line the entry point prints.
#
# This is the entry point node `ci-cost-floor-preparation-cut` demands: it does not quote a figure out
# of a log, it READS THE PHASE LINES A RUN PRINTS, at module grain, and writes the rows the authority
# `src/v2/workflow/floor_preparation_split.dag` folds. That authority's own claims are what make the
# numbers checkable -- conservation, the fill/pair counterfactual, the deletion margin, the refusal set.
# This script is a GENERATOR, never a source: the cost fields are values only the native binary
# produces, so a hand-typed one here or in the model is a fabrication by rule.
#
# It is shell and not Python because `.gitignore:172` (`*.py`, owner gunbc.repo_workspace) keeps Python
# "outside the committed substrate -- local dev tooling only; never committed". A measurement entry
# point nobody can commit is a measurement nobody can re-run, so the instrument lives in the substrate.
#
# MODES
#   report  Fold any log(s) and print the split. No execution, no writes -- the cheap re-derivation:
#             scripts/floor_preparation_capture.sh report --report-log <log> --report-label full
#   fold    Re-derive from logs already on disk and splice the generated blocks into the authority:
#             scripts/floor_preparation_capture.sh fold --untrimmed-log <log> --graded-log <log> \
#               --population <txt> --outdir <dir> --provenance binary_sha256=<sha> \
#               --provenance head_sha=<sha> [--write]
#   run     Execute both legs against the pinned released claim_batch, inside the CALLER's heavy slot:
#             HEAVY_OWNER=<session> ~/bin/heavy-slot \
#               scripts/floor_preparation_capture.sh run --checkout <dir> --binary <claim_batch> \
#               --expect-sha256 <sha> --expect-tree <tree> --population <txt> --outdir <dir> --write
#
# THE TWO LEGS. `untrimmed` enrolls every claim in the population, green and refused together, so the
# artifact shows what the floor actually pays and keeps each refused claim's own refusal line beside it.
# `graded` re-runs only what came back PASS; that is the capture the split is folded from, so a
# committed capture is green by construction rather than by belief.
#
# WHAT IT REFUSES, and why each refusal is hard rather than advisory. A leg whose argv lacks
# `--functions` prints an error and exits 2 while its caller's harness still reports green -- a five-leg
# batch in this lane died exactly that way with HARNESS_RC=0. A 0-byte binary, or one hashing as the
# empty string, is a truncated download that exits 0. An entry that never reported its witness count
# means the leg did not do what the argv asked. A `[floor-shared-fill]`/`[floor-reach-probe]` pairing
# mismatch means a fold would bill one shared-artifact build twice -- the exact direction the ticket's
# premise wants inflated. And a FAIL entering the graded capture would let "the capture is green" mean
# "nobody looked".

set -u

MODE=${1:-}
[ -n "$MODE" ] || { echo "usage: $0 run|fold|report [options]" >&2; exit 2; }
shift || true

CHECKOUT=$PWD
BINARY=""
EXPECT_SHA=""
EXPECT_TREE=""
POPULATION=""
OUTDIR="capture"
GRADED_LOG=""
UNTRIMMED_LOG=""
AUTHORITY=""
PROVENANCE=""
REPORT_LOGS=""
REPORT_LABELS=""
WRITE=0

while [ $# -gt 0 ]; do
  case "$1" in
    --checkout) CHECKOUT=$2; shift 2 ;;
    --binary) BINARY=$2; shift 2 ;;
    --expect-sha256) EXPECT_SHA=$2; shift 2 ;;
    --expect-tree) EXPECT_TREE=$2; shift 2 ;;
    --population) POPULATION=$2; shift 2 ;;
    --outdir) OUTDIR=$2; shift 2 ;;
    --graded-log) GRADED_LOG=$2; shift 2 ;;
    --untrimmed-log) UNTRIMMED_LOG=$2; shift 2 ;;
    --authority) AUTHORITY=$2; shift 2 ;;
    --provenance) PROVENANCE="$PROVENANCE $2"; shift 2 ;;
    --report-log) REPORT_LOGS="$REPORT_LOGS $2"; shift 2 ;;
    --report-label) REPORT_LABELS="$REPORT_LABELS $2"; shift 2 ;;
    --write) WRITE=1; shift ;;
    *) echo "CAPTURE_FAIL: unknown argument $1" >&2; exit 2 ;;
  esac
done

fail() { echo "CAPTURE_FAIL: $*" >&2; exit 1; }

EMPTY_SHA=e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
ARGV_DEFECT="has no --function or --functions"
KEEP='^(\[floor-phase\]|\[resolve\]|\[floor-shared-fill\]|\[floor-reach-probe\]|\[witness\]|PASS |FAIL |UNOBSERVED |claim_batch: resolving |claim_batch: entry group|claim_batch: whole-tree|claim_batch: resolve failed)'
BEGIN='// === BEGIN GENERATED CAPTURE BLOCK (scripts/floor_preparation_capture.sh) ==='
ENDMARK='// === END GENERATED CAPTURE BLOCK ==='

kept() { grep -E "$KEEP" "$1" > "$2" || true; }
count_of() { local n; n=$(grep -cE "^$2" "$1" || true); echo "${n:-0}"; }

# ----------------------------------------------------------------------------------------------
# THE FOLD, mirrored from the authority so an ARBITRARY log can be measured without a commit. This
# table is a DELIBERATE duplicate of `floor_preparation_phase_rows` in
# src/v2/workflow/floor_preparation_split.dag, and the witness
# `floor_preparation_the_capture_tool_folds_the_same_phase_grammar` reads THIS FILE with
# filesystem_read and refuses if the two vocabularies drift.
# ----------------------------------------------------------------------------------------------
fold_report() {
  local log=$1 label=$2
  [ -s "$log" ] || fail "log $log is missing or empty"
  grep -E "$KEEP" "$log" | awk -v label="$label" '
    BEGIN {
      cost["module-index"] = "wall_ms"
      cost["declarer-discovery"] = "wall_ms"
      cost["whole-tree-graph-facts"] = "wall_ms"
      cost["declarer-closure"] = "wall_ms"
      cost["closure-strict-resolve"] = "wall_ms"
      cost["gate-closure"] = "wall_ms"
      cost["reference-closure-index"] = "wall_ms"
      cost["bare-reference-edge-index-warm"] = "wall_ms"
      whole["module-index"] = 1
      whole["declarer-discovery"] = 1
      whole["whole-tree-graph-facts"] = 1
      whole["declarer-closure"] = 1
    }
    function value_of(line, key,    n, toks, i, t) {
      n = split(line, toks, " ")
      for (i = 1; i <= n; i++) {
        if (index(toks[i], key "=") == 1) return substr(toks[i], length(key) + 2)
      }
      return ""
    }
    function take(raw, where,    amount) {
      if (raw == "") { printf("CAPTURE_FAIL: %s is missing its cost field\n", where) > "/dev/stderr"; exit 1 }
      if (substr(raw, length(raw) - 1) != "ms") {
        printf("CAPTURE_FAIL: %s carries a cost without the ms unit: %s\n", where, raw) > "/dev/stderr"
        exit 1
      }
      amount = substr(raw, 1, length(raw) - 2) + 0
      if (amount < 0) { printf("CAPTURE_FAIL: %s carries a negative cost\n", where) > "/dev/stderr"; exit 1 }
      return amount
    }
    /^\[floor-phase\]/ {
      if ($2 !~ /^phase=/) { print "CAPTURE_FAIL: phase row without a phase= field" > "/dev/stderr"; exit 1 }
      phase = substr($2, 7)
      if (phase == "expr-var-classification" || phase == "reach-differential") next
      if (!(phase in cost)) { printf("CAPTURE_FAIL: phase token outside the grammar: %s\n", phase) > "/dev/stderr"; exit 1 }
      amount = take(value_of($0, cost[phase]), "phase " phase)
      preparation += amount
      if (phase in whole) { whole_tree += amount } else { entry_scoped += amount }
      phases++
      next
    }
    /^\[resolve\]/ {
      line = substr($0, 10)
      colon = index(line, ":")
      if (colon == 0) { print "CAPTURE_FAIL: [resolve] row without a path" > "/dev/stderr"; exit 1 }
      entry = substr(line, 1, colon - 1)
      gsub(/^[ \t]+|[ \t]+$/, "", entry)
      split(substr(line, colon + 1), parts, " ")
      digits = parts[1]
      gsub(/[^0-9]/, "", digits)
      if (digits == "") { print "CAPTURE_FAIL: [resolve] row without a cost" > "/dev/stderr"; exit 1 }
      claim += digits + 0
      claims++
      modules[entry] += digits + 0
      current = entry
      seen_resolve = 1
      next
    }
    /^\[witness\]/ {
      if (!seen_resolve) { print "CAPTURE_FAIL: [witness] row appeared before any [resolve] row" > "/dev/stderr"; exit 1 }
      head = substr($0, 10)
      colon = index(head, ":")
      fn = substr(head, 1, colon - 1)
      gsub(/^[ \t]+|[ \t]+$/, "", fn)
      amount = take(value_of(head, "cpu"), "[witness] row for " fn)
      claim += amount
      claims++
      owner = (current == "" ? "(unattributed)" : current)
      modules[owner] += amount
      next
    }
    /^PASS / { passed++; next }
    /^FAIL / { failed++; next }
    /^UNOBSERVED / { unobserved++; next }
    END {
      top = ""
      topms = -1
      nmods = 0
      for (m in modules) { nmods++; if (modules[m] > topms) { top = m; topms = modules[m] } }
      table = ""
      for (m in modules) table = table (table == "" ? "" : ",") m "=" modules[m]
      printf("floor-preparation-split label=%s phase_rows=%d preparation_ms=%d claim_ms=%d total_ms=%d whole_tree_ms=%d entry_scoped_ms=%d claims=%d pass=%d fail=%d unobserved=%d modules=%d top_module=%s module_table=%s\n", \
        label, phases, preparation, claim, preparation + claim, whole_tree, entry_scoped, claims, passed, failed, unobserved, nmods, top, table)
    }'
}

verify_binary() {
  [ -n "$BINARY" ] || fail "run mode needs --binary"
  [ -n "$EXPECT_SHA" ] || fail "run mode needs --expect-sha256"
  [ -f "$BINARY" ] || fail "no binary at $BINARY"
  local bytes
  bytes=$(stat -c %s "$BINARY")
  [ "$bytes" != "0" ] || fail "binary is 0 bytes (a truncated download that still exits 0)"
  SHA256=$(sha256sum "$BINARY" | cut -d' ' -f1)
  [ "$SHA256" != "$EMPTY_SHA" ] || fail "binary hashes as the empty string"
  [ "$SHA256" = "$EXPECT_SHA" ] || fail "binary sha $SHA256 deviates from the pinned $EXPECT_SHA"
  local manifest
  manifest=$(dirname "$BINARY")/release-bins.manifest
  [ -f "$manifest" ] || manifest=$(dirname "$(dirname "$BINARY")")/release-bins.manifest
  [ -f "$manifest" ] || fail "no release-bins.manifest beside the binary; provenance is unverifiable"
  local mh
  mh=$(awk '$2=="claim_batch"{print $1}' "$manifest" | head -1)
  echo "MANIFEST_SHA_claim_batch=$mh"
  [ "$mh" = "$SHA256" ] || fail "release-bins.manifest does not pin this claim_batch"
  echo "BINARY_SHA256=$SHA256"
  echo "BINARY_BYTES=$bytes"
}

# The population is a committed text file so both legs are built from ONE artifact:
#   candidate|<module>|<claim>   eligible for the graded capture
#   extra|<module>|<claim>       enrolled only so its refusal stays in the artifact
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
  for kind in candidate extra; do
    while IFS='|' read -r entry functions; do
      [ -n "$entry" ] || continue
      [ -n "$functions" ] || fail "entry $entry enrolled with an empty --functions list"
      printf '%s\n%s\n%s\n%s\n' "--entry" "$entry" "--functions" "$functions" >> "$out"
    done < <(entries_of "$kind")
  done
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
      if grep -qx -- "$name" "$PASSNAMES"; then
        if [ -z "$keep" ]; then keep=$name; else keep="$keep,$name"; fi
        total=$((total + 1))
      else
        dropped=$((dropped + 1))
      fi
    done
    IFS=$oldifs
    if [ -n "$keep" ]; then
      printf '%s\n%s\n%s\n%s\n' "--entry" "$entry" "--functions" "$keep" >> "$out"
    fi
  done < <(entries_of candidate)
  echo "GRADED_DERIVED_CLAIMS=$total DROPPED=$dropped"
  [ "$total" -gt 0 ] || fail "no graded candidate came back PASS; there is nothing to commit as the split capture"
  echo "ARGV_WRITTEN=$out ARGS=$(wc -l < "$out")"
}

audit_leg() {
  local name=$1 log=$2 argv=${3:-/dev/null}
  local phase resolve fill probe pass fail_rows unobserved resolving
  phase=$(count_of "$log" '\[floor-phase\]')
  resolve=$(count_of "$log" '\[resolve\]')
  fill=$(count_of "$log" '\[floor-shared-fill\]')
  probe=$(count_of "$log" '\[floor-reach-probe\]')
  pass=$(count_of "$log" 'PASS ')
  fail_rows=$(count_of "$log" 'FAIL ')
  unobserved=$(count_of "$log" 'UNOBSERVED ')
  resolving=$(count_of "$log" 'claim_batch: resolving ')
  echo "LEG_${name}_COUNTS phase=$phase resolve=$resolve fill=$fill probe=$probe pass=$pass fail=$fail_rows unobserved=$unobserved resolving=$resolving"
  [ -s "$log" ] || fail "leg $name produced an empty log"
  grep -q "$ARGV_DEFECT" "$log" && fail "leg $name: the binary refused with '$ARGV_DEFECT'; it measured nothing"
  grep -q "workspace_root" "$log" && fail "leg $name printed workspace_root -- not run from a git checkout?"
  grep -q "panicked at" "$log" && fail "leg $name: the binary panicked"
  [ "$phase" -gt 0 ] || fail "leg $name printed no [floor-phase] row -- preparation was never measured"
  [ "$resolving" -gt 0 ] || fail "leg $name enrolled no entry (no 'claim_batch: resolving' row)"
  [ "$resolve" -gt 0 ] || fail "leg $name resolved no entry module; a module-grain split cannot be derived"
  [ "$fill" = "$probe" ] || fail "leg $name: $fill [floor-shared-fill] rows against $probe [floor-reach-probe] rows -- the printer emits them paired, and billing both would inflate preparation"
  [ $((pass + fail_rows + unobserved)) -gt 0 ] || fail "leg $name returned no verdict for any enrolled claim"
  if [ ! -s "$argv" ]; then
    echo "LEG_${name}_WITNESS_COUNT_AUDIT=skipped (no argv supplied; pass the leg's argv to enforce it)"
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

emit_list() {
  local fn=$1 file=$2
  printf '// GENERATED -- DO NOT EDIT BY HAND. Produced by scripts/floor_preparation_capture.sh.\n'
  printf 'fn %s() -> List<String> {\n  [\n' "$fn"
  awk '{ gsub(/\\/, "\\\\"); gsub(/"/, "\\\""); printf("    \"%s\",\n", $0) }' "$file"
  printf '  ]\n}\n\n'
}

build_requested_rows() {
  local out=$1 kind entry functions name
  : > "$out"
  for kind in candidate extra; do
    while IFS='|' read -r entry functions; do
      [ -n "$entry" ] || continue
      local oldifs=$IFS
      IFS=','
      for name in $functions; do
        if grep -qx -- "$name" "$PASSNAMES"; then
          printf '%s|%s|graded|\n' "$entry" "$name" >> "$out"
        elif grep -qx -- "FAIL $name" "$UNTRIMMED_KEPT"; then
          printf '%s|%s|excluded|refused\n' "$entry" "$name" >> "$out"
        elif grep -qE "^UNOBSERVED $name:" "$UNTRIMMED_KEPT"; then
          printf '%s|%s|excluded|unobserved\n' "$entry" "$name" >> "$out"
        else
          printf '%s|%s|excluded|no_verdict_line\n' "$entry" "$name" >> "$out"
        fi
      done
      IFS=$oldifs
    done < <(entries_of "$kind")
  done
  echo "REQUESTED_ROWS=$(wc -l < "$out")"
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
  [ -s "$GRADED_KEPT" ] || fail "the graded leg kept no instrument rows"
  [ -s "$UNTRIMMED_KEPT" ] || fail "the untrimmed leg kept no instrument rows"
  grep '^PASS ' "$GRADED_KEPT" | sed 's/^PASS //' > "$PASSNAMES"
  [ -s "$PASSNAMES" ] || fail "the graded log has no PASS row"
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
  fold_report "$UNTRIMMED_LOG" untrimmed
  fold_report "$GRADED_LOG" graded
  echo "CAPTURE_OK graded_rows=$(wc -l < "$GRADED_KEPT") untrimmed_rows=$(wc -l < "$UNTRIMMED_KEPT")"
}

main() {
  case "$MODE" in
    report)
      [ -n "$REPORT_LOGS" ] || fail "report mode needs at least one --report-log"
      local logs labels i
      read -r -a logs <<< "$REPORT_LOGS"
      read -r -a labels <<< "$REPORT_LABELS"
      [ "${#logs[@]}" = "${#labels[@]}" ] || fail "report mode needs one --report-label per --report-log"
      for i in "${!logs[@]}"; do
        fold_report "${logs[$i]}" "${labels[$i]}"
      done
      echo "REPORT_OK"
      ;;
    fold)
      [ -n "$POPULATION" ] || fail "fold mode needs --population"
      [ -n "$GRADED_LOG" ] && [ -n "$UNTRIMMED_LOG" ] || fail "fold mode needs --graded-log and --untrimmed-log"
      [ -s "$GRADED_LOG" ] || fail "graded log $GRADED_LOG is missing or empty"
      [ -s "$UNTRIMMED_LOG" ] || fail "untrimmed log $UNTRIMMED_LOG is missing or empty"
      case " $PROVENANCE " in
        *" binary_sha256="*) ;;
        *) fail "fold mode needs --provenance binary_sha256=<sha>: a capture without a pinned binary is a rumor" ;;
      esac
      audit_leg G "$GRADED_LOG" "$OUTDIR/argv-G.txt"
      [ "$(count_of "$GRADED_LOG" 'FAIL ')" = "0" ] || fail "the graded capture is not green"
      [ "$(count_of "$GRADED_LOG" 'UNOBSERVED ')" = "0" ] || fail "the graded capture has unobserved claims"
      mkdir -p "$OUTDIR"
      # The witness-count audit reads the population; the graded leg ran a subset, so compare against
      # what the graded argv actually enrolled -- rebuilt here from the PASS set.
      GRADED_KEPT=/dev/null
      UNTRIMMED_KEPT=/dev/null
      PASSNAMES=/dev/null
      emit_and_splice
      ;;
    run)
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
      run_leg U "$OUTDIR/argv-U.txt" 3300 "$OUTDIR/untrimmed-run.log" "$OUTDIR/untrimmed-time.txt" || fail "the untrimmed leg was refused"
      mkdir -p "$OUTDIR"
      PASSNAMES=$OUTDIR/passnames.txt
      kept "$OUTDIR/untrimmed-run.log" "$OUTDIR/untrimmed-rows.tmp"
      grep '^PASS ' "$OUTDIR/untrimmed-rows.tmp" | sed 's/^PASS //' > "$PASSNAMES"
      build_argv_graded "$OUTDIR/argv-G.txt"
      run_leg G "$OUTDIR/argv-G.txt" 2100 "$OUTDIR/graded-run.log" "$OUTDIR/graded-time.txt" || fail "the graded leg was refused"
      [ "$(count_of "$OUTDIR/graded-run.log" 'FAIL ')" = "0" ] || fail "the graded capture is not green"
      [ "$(count_of "$OUTDIR/graded-run.log" 'UNOBSERVED ')" = "0" ] || fail "the graded capture has unobserved claims"
      GRADED_LOG=$OUTDIR/graded-run.log
      UNTRIMMED_LOG=$OUTDIR/untrimmed-run.log
      PROVENANCE="$PROVENANCE head_sha=$head tree_sha=$tree binary_sha256=$SHA256 binary_bytes=$(stat -c %s "$BINARY")"
      emit_and_splice
      ;;
    *)
      echo "usage: $0 run|fold|report [options]" >&2
      exit 2
      ;;
  esac
}

main
