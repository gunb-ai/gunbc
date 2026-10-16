#!/usr/bin/env bash
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT/src/v1/stage0"
cargo build --offline --bin gunbc
BIN="$ROOT/src/v1/stage0/target/debug/gunbc"
if [ ! -x "$BIN" ]; then
  BIN="$(find "$ROOT/src/v1/stage0" -name gunbc -type f | head -1)"
fi
echo "BIN=$BIN"
export GUNBC_GAP_ACCOUNT=1
cd "$ROOT"
for d in 1 2 3 4 5 6; do
  echo "===== AFTER depth $d ====="
  "$BIN" compile --source-root "$ROOT" --entry "dag/test/gap_depth_${d}.dag" 2>&1 | grep -E 'generic-arg-pass-excl|reconcile done|error:' || true
done
echo "===== WITNESS ====="
"$BIN" run --source-root "$ROOT" --entry dag/test/claim/generic_argument_recheck_witness.dag --claim-run 2>&1 | tail -60
