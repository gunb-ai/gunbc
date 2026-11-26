#!/usr/bin/env bash
# Route (a): staging that READS roster.path — the roster's measurement-level consumer.
#
# WHY THIS EXISTS. The roadmap row fixes two populations AS A BENCHMARK; a benchmark nobody reads is
# decoration (review 98443-2 / 98730). The parse-population verb measures cost over STAGED ROOTS, not over
# a row list (a row list cannot be timed). So the roster earns its place at the level the row's claims are
# made by being the SOURCE of those roots: this script builds each named population's roots from
# parse_population_baseline_rows out of the carrier — it does not restate the roster (no second table,
# fail_closed: construction over validation) and reads parse_population_baseline_pinned_sha and
# parse_population_five_largest_count BY NAME out of the carrier. A declared path that does not exist at the
# pinned sha, or whose actual blob byte size differs from the declared bytes, makes THIS leg refuse and name
# that path — so a mistyped row turns the verb input red at the measurement level, not only at the reconcile
# level. The verb run then parses exactly the rostered set: roots come from roster paths, byte-checked first.
#
# HOW IT RELATES TO run_evidence.sh. run_evidence.sh proves the roster reconciles to the tree (declared
# byte == tree byte for all 170). THIS script consumes that roster to produce the verb's staged roots and
# re-asserts byte+path existence while staging, so the SAME wrong row that fails the reconcile also aborts
# staging before the verb runs. Two legs, one roster, one authority.
#
# REGEX READ = DECLARED RUNG DROP (review 98809 F2); RANK SELECTION = MODELLED RULE (review 98809 F1).
# Like run_evidence.sh, the rows/constants are read from the .dag source by awk/grep/sed -- a second reader
# beside the grammar whose gap (a MISPARSED FIELD, which the count guard does not catch) is declared as the
# typed rung drop parse_population_carrier_reader_rung_drop in the carrier with a bounded trigger. The rank
# split below (first parse_population_five_largest_count = FiveLargest, the rest = size-tail) APPLIES the
# selection order that is modelled and checked in the carrier (parse_population_selection_precedes + its
# top-N/path-tiebreak discriminators), so this script is transport, not the authority for the rule.
#
# SCAFFOLD - dissolve-on (removal marker; both triggers named, per the accepted precedent
# test/compiler_gate_contract_stamp/run_evidence.sh which is about ITS case, as this header is about ours):
#   (1) RESIDUAL SHELL capability: once v2.extdeps.languages.bash_build emitted node kinds can render
#       "for each roster row: verify git blob byte size, then copy the blob into a governed tree dir," this
#       becomes an emitted step. Everything expressible that way already is; git-blob-read + path copy are
#       the residual.
#   (2) TEST-PROCESS capability (restoration_trigger): once an executed claim evaluator for fs/roster-driven
#       staging lands on the required pull_request path, staging becomes a gunbc instrument row that reads
#       roster.path directly and DELETES this file (not emitted).
set -u
set -o pipefail

pop="${1:-}"; mistype=0; out_dir=""
shift || true
for arg in "$@"; do
  case "$arg" in
    --selftest-mistype) mistype=1 ;;
    --out) : ;; # handled below via positional after pop
    *) [ -z "$out_dir" ] && out_dir="$arg" ;;
  esac
done
case "$pop" in five|tail165|all) : ;; *) echo "usage: $0 <five|tail165|all> [--out DIR] [--selftest-mistype]"; exit 2 ;; esac

run_token="ppstage-$(date -u +%Y%m%dT%H%M%SZ)-$$"
repo="$(cd "$(dirname "$0")/../.." && pwd)"
carrier="dag/gunbc/parse_population_baseline_roster.dag"
[ -f "$repo/$carrier" ] || { echo "carrier missing"; exit 2; }
cd "$repo" || exit 2

echo "== run token: $run_token"
echo "== host: $(hostname)  boot: $(cat /proc/sys/kernel/random/boot_id)"

# Read the provenance constants BY NAME out of the carrier (their consumer is THIS executed leg).
sha="$(awk '/^data parse_population_baseline_pinned_sha: String =/{getline; gsub(/[ "]/,""); print; exit}' "$carrier")"
five_n="$(grep -E '^data parse_population_five_largest_count: Int = ' "$carrier" | sed -E 's/.*= *([0-9]+).*/\1/')"
total_n="$(grep -E '^data parse_population_expected_total_count: Int = ' "$carrier" | sed -E 's/.*= *([0-9]+).*/\1/')"
[ -n "$sha" ] && [ -n "$five_n" ] && [ -n "$total_n" ] || { echo "could not read pinned_sha / five_largest_count / expected_total_count by name"; exit 1; }
echo "== read from carrier: pinned_sha=$sha five_largest_count=$five_n expected_total_count=$total_n"

# rows "<bytes>\t<path>" in rank order, straight out of the carrier.
awk '/^data parse_population_baseline_rows: List<ParsePopulationRow> = \[/{f=1;next} f&&/^\]/{exit} f' "$carrier" \
  | grep -oE 'path: "[^"]+", bytes: byte_size\(count: [0-9]+\)' \
  | sed -E 's/.*path: "([^"]+)", bytes: byte_size\(count: ([0-9]+)\)/\2\t\1/' > /tmp/pp_rows.$$.tsv
rows_n="$(wc -l < /tmp/pp_rows.$$.tsv)"
[ "$rows_n" -eq "$total_n" ] || { echo "FAIL [$run_token]: roster row count $rows_n != expected_total_count $total_n"; rm -f /tmp/pp_rows.$$.tsv; exit 1; }

# select the population slice by rank (five = first $five_n; tail165 = the rest; all = full).
case "$pop" in
  five)    sed -n "1,${five_n}p" /tmp/pp_rows.$$.tsv > /tmp/pp_pop.$$.tsv ;;
  tail165) sed -n "$((five_n+1)),${total_n}p" /tmp/pp_rows.$$.tsv > /tmp/pp_pop.$$.tsv ;;
  all)     cp /tmp/pp_rows.$$.tsv /tmp/pp_pop.$$.tsv ;;
esac
want="$(wc -l < /tmp/pp_pop.$$.tsv)"

if [ "$mistype" -eq 1 ]; then sed -E '1s/^[0-9]+/0/' /tmp/pp_pop.$$.tsv > /tmp/pp_pop.$$.tsv.m; mv /tmp/pp_pop.$$.tsv.m /tmp/pp_pop.$$.tsv; fi

: "${out_dir:=$(mktemp -d)/roots_${pop}}"
mkdir -p "$out_dir"
staged=0; mismatches=0; total_bytes=0
while IFS=$'\t' read -r bytes path; do
  # path existence + ACTUAL blob byte size at the pinned sha (measurement-level path/byte check).
  actual="$(git ls-tree -l "$sha" -- "$path" | awk '{print $4}')"
  if [ -z "$actual" ]; then
    echo "FAIL [$run_token]: declared path absent at $sha: $path"; mismatches=$((mismatches+1)); continue
  fi
  if [ "$actual" != "$bytes" ]; then
    echo "FAIL [$run_token]: byte mismatch on $path (declared $bytes != tree $actual)"; mismatches=$((mismatches+1)); continue
  fi
  case "$path" in
    dag/*)     dest="$out_dir/dag/${path#dag/}" ;;
    src/v2/*)  dest="$out_dir/src/v2/${path#src/v2/}" ;;
    *)         echo "FAIL [$run_token]: $path not under a governed root (dag/ or src/v2/)"; mismatches=$((mismatches+1)); continue ;;
  esac
  mkdir -p "$(dirname "$dest")"
  git show "$sha:$path" > "$dest" || { echo "FAIL [$run_token]: could not read blob $sha:$path"; mismatches=$((mismatches+1)); continue; }
  staged=$((staged+1)); total_bytes=$((total_bytes+actual))
done < /tmp/pp_pop.$$.tsv
rm -f /tmp/pp_rows.$$.tsv /tmp/pp_pop.$$.tsv

echo "== staged population=$pop requested=$want staged=$staged bytes=$total_bytes sha_mismatches=$mismatches out=$out_dir"
if [ "$mismatches" -ne 0 ] || [ "$staged" -ne "$want" ]; then
  echo "FAIL [$run_token]: staging refused for population $pop ($mismatches mismatches of $want)"; exit 1
fi
echo "== EVIDENCE VERDICT: population $pop staged from roster at $sha (all paths present, byte sizes match), token $run_token"
