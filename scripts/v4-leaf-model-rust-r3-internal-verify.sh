#!/usr/bin/env bash
# scripts/v4-leaf-model-rust-r3-internal-verify.sh
#
# Phase 1 leaf-model R3-internal — Symbol TargetAtomRealization row mutation must change
# both type-projection and value-projection emit together (PR #3971 §4).

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
if [[ ! -f "$fixture_dag" ]]; then
  echo "error: missing fixture authority at $fixture_dag" >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not on PATH" >&2
  exit 1
fi

set +e
cargo test -p v3-compiler --test integration \
  v4_leaf_model_rust_r3_internal \
  -- --exact 2>&1 | tee /tmp/v4-leaf-model-r3-internal-verify.log
status=${PIPESTATUS[0]}
set -e

python3 - <<'PY'
import json
import os
import re

log_path = "/tmp/v4-leaf-model-r3-internal-verify.log"
log = ""
if os.path.isfile(log_path):
    log = open(log_path, encoding="utf-8").read()

passed = sum(1 for _ in re.finditer(r"\btest v4_leaf_model_rust_r3_internal\S+ \.\.\. ok\b", log))
failed = sum(1 for _ in re.finditer(r"\btest v4_leaf_model_rust_r3_internal\S+ \.\.\. FAILED\b", log))

print(
    json.dumps(
        {
            "schema": "scripts/v4-leaf-model-rust-r3-internal-verify.sh::host_receipt_v1",
            "claim_id": "RustR3InternalSymbolEmitCoupling",
            "cargo_exit": int(os.environ.get("V4_R3_INTERNAL_CARGO_EXIT", "1")),
            "tests_passed": passed,
            "tests_failed": failed,
            "proven": os.environ.get("V4_R3_INTERNAL_PROVEN", "false") == "true",
        },
        indent=2,
    )
)
PY

if [[ "$status" -ne 0 ]]; then
  echo "error: leaf-model R3-internal verification failed (cargo test)" >&2
  exit 1
fi

export V4_R3_INTERNAL_CARGO_EXIT=0
export V4_R3_INTERNAL_PROVEN=true

echo "leaf-model R3-internal verification PROVEN"
