#!/usr/bin/env bash
# 🟡 P-PROBE-CF-IMPORT resolve-fail repro (adhoc-20b17ff7-932 / zesty-swift-79).
# Asserts dual-root claim_batch still fails resolving dsl/test/claim/probe_selector_compute_fabric_import_repro.dag.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
entry="dsl/test/claim/probe_selector_compute_fabric_import_repro.dag"
function="probe_selector_compute_fabric_import_repro_holds"

bin="${CLAIM_BATCH:-$root/target/release/claim_batch}"
if [[ ! -x "$bin" ]]; then
  bin="${CLAIM_BATCH:-$root/target/debug/claim_batch}"
fi
if [[ ! -x "$bin" ]]; then
  cargo build -p v2-compiler --bin claim_batch
  bin="$root/target/debug/claim_batch"
fi

set +e
out="$("$bin" \
  --source-root "$root/src/v4" \
  --source-root "$root/dsl" \
  --entry "$entry" \
  --function "$function" 2>&1)"
rc=$?
set -e

printf '%s\n' "$out"

if [[ "$rc" -eq 0 ]]; then
  echo "error: P-PROBE-CF-IMPORT repro resolved green — stale falsifier; flip ExpectFail row to ExpectPass" >&2
  exit 1
fi

if ! printf '%s\n' "$out" | grep -q "name 'Option' not found"; then
  echo "error: P-PROBE-CF-IMPORT repro failed but without expected Option-not-found diagnostic" >&2
  exit 1
fi

echo "P-PROBE-CF-IMPORT repro: resolve failed as expected (Option substrate gap)"
