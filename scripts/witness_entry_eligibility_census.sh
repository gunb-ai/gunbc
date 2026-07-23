#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: Filesystem.List walk + pure `.dag` census row fold replaces
# this bash runner and the `witness_entry_eligibility_census_emit` HAND-RUST transport;
# classification authority is `v2.compiler.self_host.witness_entry_eligibility_census`.
#
# Entry-grain witness eligibility census (ROADMAP 2a Lane B): mechanical transport only —
# enumerates discovery witness entry closures and delegates every disposition/leg column to
# the `.dag` authority via `witness_entry_eligibility_census_emit`.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Default carrier paths mirror `witness_entry_eligibility_census_{tsv,histogram}_path`
# in witness_entry_eligibility_census.dag (§3 dissolve-on: review 41784).
TSV="${1:-$ROOT/docs/probes/witness_entry_eligibility_census.tsv}"
HIST="${2:-$ROOT/docs/probes/witness_entry_eligibility_histogram.txt}"

if [[ -x "$ROOT/target/release/witness_entry_eligibility_census_emit" ]]; then
  EMIT="$ROOT/target/release/witness_entry_eligibility_census_emit"
elif [[ -x "$ROOT/target/debug/witness_entry_eligibility_census_emit" ]]; then
  EMIT="$ROOT/target/debug/witness_entry_eligibility_census_emit"
else
  echo "[witness_entry_eligibility_census] building emit transport (one-time)..."
  CTRL_BUILD_WRAP_CARGO=0 cargo build -p v1-compiler --bin witness_entry_eligibility_census_emit
  EMIT="$ROOT/target/debug/witness_entry_eligibility_census_emit"
fi

"$EMIT" "$TSV" "$HIST"
