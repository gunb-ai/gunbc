#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: Wave 1 name-resolution debt repaid (qualification or binding-wall
# lands and floor_expected_red shrinks) OR a floor-enrolled census lens subsumes the
# required_floor_failure_census host transport.
# Hand-shell transport (operator-scoped diagnostic for Wave 1 partition lane); modeled
# intent authority is docs/probes/floor_cut_name_resolution_partition.md until bash-emit.
# dissolve-on alt: modeled census entry in .dag via gunbc bash-emit / host_effect_apply (#5828).
# Authority: docs/probes/floor_cut_name_resolution_partition.md;
#   src/v1/stage0/src/bin/required_floor_failure_census.rs
#   (CLI_RUN_REQUIRED_FLOOR_FAILURE_CENSUS_SCAFFOLD_MARKER in cli_run.rs);
#   post-process via docs/probes/floor_cut_name_resolution_partition.py.
# Receipt: `rg required_floor_failure_census src/v1/stage0` == 1 until deletion.
#
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
