#!/usr/bin/env bash
# LOCAL-DEV-ONLY SCAFFOLD — dissolve-on: GATE 3 CI self-demonstration of scoped
# compile-clean in budget retires this script (see tools.dag_compile_clean_scope
# lever_a_local_verify_scaffold_note). Pure sequencer: claim_batch verdict only.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
BIN="${CLAIM_BATCH:-$ROOT/target/release/claim_batch}"
if [[ ! -x "$BIN" ]]; then
  CTRL_BUILD_BYPASS_SHIMS=1 cargo build -p v1-compiler --release --bin claim_batch
fi
exec "$BIN" \
  --source-root dag --source-root src/v2 \
  --entry dag/test/claim/lever_a_local_receipt_witness_test.dag \
  --function lever_a_local_receipts_hold \
  --claim-run --wet
