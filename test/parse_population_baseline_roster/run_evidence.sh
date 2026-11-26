#!/usr/bin/env bash
# Route (i): the ONLY executed tie between the 170 declared roster rows and the real source tree.
#
# WHAT THIS RUNNER IS. The .dag carrier (dag/gunbc/parse_population_baseline_roster.dag) is the single
# typed authority for the two fixed parse populations (planning_has_one_home). The .dag comparison
# predicate parse_population_unreconciled_row_count MODELS the "does every declared path AND byte size
# equal the tree" comparison, and its claim arms (parse_population_reconcile_accepts_correct_slice /
# ..._refuses_mistyped_slice) prove that predicate DISCRIMINATES. But a .dag claim CANNOT read the tree:
# the language has no filesystem, and gunbc CI executes no claims, so a claim row over a tree read would
# sit off the required path and its red would be attested, not executed (witness_one_interface: supplying
# inputs may never remove the last execution of the real path). THIS RUNNER is that last execution: it
# feeds the tree to nothing but `git ls-tree`, reads the sha and the rows OUT OF THE CARRIER, and fails by
# name on any row that does not match.
#
# WHY THE CARRIER (NOT A COPY) IS READ. This script holds no roster table of its own: it extracts
# parse_population_baseline_rows FROM the .dag source and compares it to `git ls-tree`. A second table here
# would be a second authority (fail_closed: construction over validation). It also reads
# parse_population_baseline_pinned_sha OUT OF THE CARRIER and prints it BY NAME, so the constant's consumer
# is an executed leg, not a claim about a run (review 98730 / warm-fox).
#
# SCAFFOLD - dissolve-on (the removal marker review 98730 asked for): retire this file when EITHER
#   (1) v2.extdeps.languages.bash_build emitted node kinds can render extract / ls-tree / diff as an
#       emitted step (same capability and rationale as test/compiler_gate_contract_stamp/run_evidence.sh), or
#   (2) an executed claim evaluator for tree-reading claims lands on the required pull_request path, so this
#       comparison becomes a `gunbc test` row in gunbc.instrument_targets. When (2) lands this file is
#       DELETED, not emitted. Cited so the carrier's reconcile comment and this runner retire together.
set -u

run_token="ppbr-$(date -u +%Y%m%dT%H%M%SZ)-$$"
repo="$(cd "$(dirname "$0")/../.." && pwd)"
carrier="dag/gunbc/parse_population_baseline_roster.dag"
expected_rows=170
scratch="$(mktemp -d)"; trap 'rm -rf "$scratch"' EXIT
carrier_rows="$scratch/carrier_rows.tsv"; tree_top="$scratch/tree_top.tsv"
mistyped="$scratch/mistyped.tsv"; red_diff="$scratch/red_diff.txt"

echo "== run token: $run_token"
echo "== host: $(hostname)  boot: $(cat /proc/sys/kernel/random/boot_id)"
cd "$repo" || { echo "no repo root"; exit 2; }
[ -f "$carrier" ] || { echo "carrier missing: $carrier"; exit 2; }

# 1. pinned_sha read BY NAME out of the carrier (its consumer is THIS executed leg).
sha="$(awk '/^data parse_population_baseline_pinned_sha: String =/{getline; gsub(/[ "]/,""); print; exit}' "$carrier")"
[ -n "$sha" ] || { echo "could not read parse_population_baseline_pinned_sha from carrier"; exit 1; }
echo "== parse_population_baseline_pinned_sha (read from carrier): $sha"

# 2. the declared rows read out of the SAME carrier: "<bytes>\t<path>", in declared (rank-desc) order.
awk '/^data parse_population_baseline_rows: List<ParsePopulationRow> = \[/{f=1;next} f&&/^\]/{exit} f' "$carrier" \
  | grep -oE 'path: "[^"]+", bytes: byte_size\(count: [0-9]+\)' \
  | sed -E 's/.*path: "([^"]+)", bytes: byte_size\(count: ([0-9]+)\)/\2\t\1/' > "$carrier_rows"
carrier_n="$(wc -l < "$carrier_rows")"

# 3. the tree at that sha: top-170 .dag blobs under the two governed roots, byte-size desc then path.
git ls-tree -r -l "$sha" -- dag src/v2 \
  | awk '$4 ~ /^[0-9]+$/ && $5 ~ /\.dag$/ {print $4"\t"$5}' \
  | sort -t"$(printf '\t')" -k1,1nr -k2,2 | head -"$expected_rows" > "$tree_top"
tree_n="$(wc -l < "$tree_top")"

# fail-closed on counts before any content diff (a short side is itself red, not a passing empty diff).
if [ "$carrier_n" -ne "$expected_rows" ] || [ "$tree_n" -ne "$expected_rows" ]; then
  echo "FAIL [$run_token]: count mismatch carrier=$carrier_n tree=$tree_n expected=$expected_rows"; exit 1
fi

# 4. GREEN: every declared path AND byte size equals the tree.
if diff -u "$carrier_rows" "$tree_top"; then
  echo "PASS [$run_token]: all $expected_rows roster rows reconcile to git ls-tree at $sha (rc 0)"
else
  echo "FAIL [$run_token]: roster rows do NOT reconcile to the tree (diff nonzero)"; exit 1
fi

# 5. RED (executed, not attested): mistype the FIRST declared row's byte size by one digit. The leg MUST
#    refuse, and must print that row's path by name. If injecting a real wrong row does NOT fail, the leg
#    is vacuous and this runner exits 1 -- that is the discriminating control review 98443/98730 demanded.
sed -E '1s/^[0-9]+/0/' "$carrier_rows" > "$mistyped"
bad_path="$(head -1 "$carrier_rows" | cut -f2)"
if diff -u "$mistyped" "$tree_top" > "$red_diff"; then
  echo "FAIL [$run_token]: mistyped row NOT detected -- reconciliation leg is vacuous"; exit 1
fi
if grep -q "$bad_path" "$red_diff"; then
  echo "PASS [$run_token]: red control discriminates -- mistyped byte size on $bad_path turns the leg red"
else
  echo "FAIL [$run_token]: leg turned red but did NOT name the mistyped row $bad_path"; exit 1
fi
echo "== EVIDENCE VERDICT: reconciliation leg executed (green reconcile + discriminating red), token $run_token"
