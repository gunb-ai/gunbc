#!/usr/bin/env bash
# 🟡 P-PROBE-CF-IMPORT repro (adhoc-20b17ff7-932 / zesty-swift-79).
# Documents v4→dsl/std/compute_fabric import failure without living in src/v4
# (M1 full-tree emit probe compiles all src/v4/*.dag — a broken import there breaks CI).
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

repro_dag="$tmpdir/probe_selector_compute_fabric_import_repro.dag"
cat >"$repro_dag" <<'EOF'
module v4.test.claim.workflow.probe_selector_compute_fabric_import_repro

import v4.std.logic { Bool }
import std.compute_fabric { supply_srv1_offer }

fn probe_selector_compute_fabric_import_repro_holds() -> Bool {
  supply_srv1_offer.provider == supply_srv1_offer.provider
}
EOF

bin="${CLAIM_BATCH:-$root/target/debug/claim_batch}"
if [[ ! -x "$bin" ]]; then
  cargo build -p v2-compiler --bin claim_batch
fi

set +e
"$bin" \
  --source-root "$root/src/v4" \
  --source-root "$root/dsl" \
  --entry "$repro_dag" \
  --function probe_selector_compute_fabric_import_repro_holds
rc=$?
set -e

if [[ "$rc" -eq 0 ]]; then
  echo "error: expected resolve failure (Option vs Optional substrate gap)" >&2
  exit 1
fi

echo "P-PROBE-CF-IMPORT repro: resolve failed as expected (exit $rc)"
