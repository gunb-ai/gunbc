#!/usr/bin/env bash
# Run the expected-red failure census remotely and build the identity-grain partition.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

CENSUS_TSV="${1:-docs/probes/floor_cut_name_resolution_census.tsv}"
PARTITION_TSV="${2:-docs/probes/floor_cut_name_resolution_partition.tsv}"

ctrl-build --remote -- bash -lc "
  set -euo pipefail
  cd \"\$PWD\"
  export CTRL_BUILD_BYPASS_SHIMS=1
  export PATH=/opt/cargo/bin:\$PATH
  cargo build --release -p v1-compiler --bin required_floor_failure_census
  ./target/release/required_floor_failure_census \"$CENSUS_TSV\"
"

python3 docs/probes/floor_cut_name_resolution_partition.py \
  "$CENSUS_TSV" \
  -o "$PARTITION_TSV"

echo "partition: $PARTITION_TSV"
