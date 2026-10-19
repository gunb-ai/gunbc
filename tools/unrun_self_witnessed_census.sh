#!/usr/bin/env bash
set -euo pipefail

# Census instrument for the recurring-failure-mode row
# `gunbc.recurring_failure_mode.unrun_self_witnessed_module_reads_as_coverage`: the population of
# SELF-WITNESSED PRODUCTION MODULES -- `.dag` files that carry `test fn` bodies but sit outside the
# directories CI witness discovery actually scans, so their tests are never executed as checks.
#
# WHY A SHELL INSTRUMENT AND NOT A `gunbc test <label>`: a label is a row in `gunbc.instrument_targets`
# PLUS a producer that runs it, and no producer exists for corpus enumeration --
# `v2.lens.mandatory_tag.corpus_scan` carries the modules it scans as written data, and there is no typed
# corpus walk to build a real one on. A label with no producer would be an instrument reporting coverage
# it never performs, which is this row's own subject. When a corpus-enumeration producer exists, the label
# supersedes this script and this comment goes with it.
#
# IT READS ITS OWN FILTERS RATHER THAN COPYING THEM. `witness_layer_roots` and `witness_discovery_scan_dirs`
# are parsed out of `gunbc.ci.ci_layer_roots` at run time, and the census REFUSES (exit 3) if either is
# unreadable or if a named layer root is absent from the tree. Enrolling a directory in that authority
# therefore changes this measurement with no edit here -- which is what `--enrol` is for: it shows an
# operator what an enrolment decision would measure before the decision is made.
#
# Usage (from the repo root):
#   tools/unrun_self_witnessed_census.sh                        population: bodies + path per module, then totals
#   tools/unrun_self_witnessed_census.sh --include-test-family  drop the `*/test/*` exclusion: the wider gap
#   tools/unrun_self_witnessed_census.sh --enrol DIR            treat DIR as enrolled, on top of the authority
#   tools/unrun_self_witnessed_census.sh --cost-basis           cost-basis coverage OF THE POPULATION (see below)
# Exit: 0 measured; 2 bad argument; 3 refused (authority row unreadable, or a layer root is absent).
#
# --cost-basis answers a different question and must not be read as execution evidence: a row in
# `dag/gunbc/witness/witness_row_cost_basis.tsv` records a cost basis established by first fleet observation, not
# that anyone ran the test as a check. It prints, for the population only, how many test bodies have such a row,
# and how many distinct run_ref / host_class values back the whole table -- a table with one run_ref is a seeding
# batch, which is why a basis row is not a receipt of execution.

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"
AUTH=dag/gunbc/ci/ci_layer_roots.dag

include_test_family=1
enrol_extra=""
cost_basis=0
while [ $# -gt 0 ]; do
  case "$1" in
    --include-test-family) include_test_family=0 ;;
    --cost-basis) cost_basis=1 ;;
    --enrol)
      [ $# -ge 2 ] || { echo "Refused: --enrol needs a directory" >&2; exit 2; }
      enrol_extra="$2"; shift ;;
    *) echo "Refused: unknown argument $1" >&2; exit 2 ;;
  esac
  shift
done

[ -f "$AUTH" ] || { echo "Refused: $AUTH unreadable" >&2; exit 3; }
read_list() {
  sed -n "s/^data $1: List<String> = \[\([^]]*\)\].*/\1/p" "$AUTH" | tr -d '"' | tr ',' ' '
}
grep -q '^data witness_layer_roots: List<String> = \[' "$AUTH"        || { echo "Refused: witness_layer_roots unreadable" >&2; exit 3; }
grep -q '^data witness_discovery_scan_dirs: List<String> = \[' "$AUTH" || { echo "Refused: witness_discovery_scan_dirs unreadable" >&2; exit 3; }
roots=$(read_list witness_layer_roots)
scan_dirs=$(read_list witness_discovery_scan_dirs)
[ -n "$roots" ] || { echo "Refused: witness_layer_roots is empty" >&2; exit 3; }
[ -n "$scan_dirs" ] || { echo "Refused: witness_discovery_scan_dirs is empty" >&2; exit 3; }
for r in $roots; do
  [ -d "$r" ] || { echo "Refused: layer root $r absent from the tree" >&2; exit 3; }
done
[ -z "$enrol_extra" ] || scan_dirs="$scan_dirs $enrol_extra"

rows=$(
  for r in $roots; do find "$r" -name '*.dag'; done | sort | while IFS= read -r f; do
    n=$(grep -c '^test fn ' "$f" || true)
    [ "$n" -gt 0 ] || continue
    skip=0
    for d in $scan_dirs; do case "$f" in "$d"/*) skip=1 ;; esac; done
    if [ "$include_test_family" -eq 1 ]; then
      case "$f" in */test/*) skip=1 ;; esac
    fi
    [ "$skip" -eq 0 ] || continue
    printf '%s\t%s\n' "$n" "$f"
  done | sort -rn
)

printf '%s\n' "$rows"
awk -F'\t' 'NF{f++;s+=$1} END{printf "MODULES=%d TEST_BODIES=%d\n", f+0, s+0}' <<< "$rows"

if [ "$cost_basis" -eq 1 ]; then
  # Same population, different question: how much of it has a cost-basis row, and what backs the table.
  BASIS=dag/gunbc/witness/witness_row_cost_basis.tsv
  [ -f "$BASIS" ] || { echo "Refused: $BASIS unreadable" >&2; exit 3; }
  basis_functions=$(awk -F'\t' '!/^#/ && NF>1 {print $2}' "$BASIS" | sort -u)
  total=0; with_basis=0
  TAB=$(printf '\t')
  while IFS="$TAB" read -r _ f; do
    [ -n "${f:-}" ] || continue
    while IFS= read -r name; do
      [ -n "$name" ] || continue
      total=$((total+1))
      if grep -qxF "$name" <<< "$basis_functions"; then with_basis=$((with_basis+1)); fi
    done < <(sed -n 's/^test fn \([A-Za-z_][A-Za-z0-9_]*\).*/\1/p' "$f")
  done <<< "$rows"
  data_rows=$(grep -vc '^#' "$BASIS")
  data_rows=$((data_rows - 1))  # minus the header line
  run_refs=$(awk -F'\t' '!/^#/ && NR>0 && $4!="" && $4!="run_ref" {print $4}' "$BASIS" | sort -u | wc -l | tr -d ' ')
  hosts=$(awk -F'\t' '!/^#/ && $5!="" && $5!="host_class" {print $5}' "$BASIS" | sort -u | tr '\n' ' ')
  echo "COST BASIS OF THE POPULATION: $with_basis of $total test bodies have a row in $BASIS"
  echo "TABLE: DATA_ROWS=$data_rows DISTINCT_RUN_REF=$run_refs HOST_CLASS=[$hosts]"
  echo "A basis row is established by first fleet observation; it is not a receipt that the test was run as a check."
fi
