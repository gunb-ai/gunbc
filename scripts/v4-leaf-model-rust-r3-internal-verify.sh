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

cargo_bin="${CARGO:-/opt/cargo/bin/cargo}"
if [[ "$(command -v "$cargo_bin")" != "$cargo_bin" && ! -x "$cargo_bin" ]]; then
  cargo_bin="$(command -v cargo)"
fi

set +e
"$cargo_bin" test -p v3-compiler --test integration -- v4_leaf_model_rust_r3_internal \
  2>&1 | tee /tmp/v4-leaf-model-r3-internal-verify.log
status=${PIPESTATUS[0]}
set -e

passed=0
failed=0
if [[ -f /tmp/v4-leaf-model-r3-internal-verify.log ]]; then
  passed="$(grep -cE 'test v4_leaf_model_rust_r3_internal[^ ]+ \.\.\. ok' /tmp/v4-leaf-model-r3-internal-verify.log || true)"
  failed="$(grep -cE 'test v4_leaf_model_rust_r3_internal[^ ]+ \.\.\. FAILED' /tmp/v4-leaf-model-r3-internal-verify.log || true)"
fi

proven=false
[[ "$status" -eq 0 && "$failed" -eq 0 && "$passed" -gt 0 ]] && proven=true

export V4_R3_INTERNAL_CARGO_EXIT="$status"
export V4_R3_INTERNAL_PROVEN="$proven"

python3 - <<'PY'
import json
import os

print(
    json.dumps(
        {
            "schema": "scripts/v4-leaf-model-rust-r3-internal-verify.sh::host_receipt_v1",
            "claim_id": "RustR3InternalSymbolEmitCoupling",
            "cargo_exit": int(os.environ["V4_R3_INTERNAL_CARGO_EXIT"]),
            "tests_passed": int(os.environ.get("V4_R3_INTERNAL_TESTS_PASSED", "0")),
            "tests_failed": int(os.environ.get("V4_R3_INTERNAL_TESTS_FAILED", "0")),
            "proven": os.environ["V4_R3_INTERNAL_PROVEN"] == "true",
        },
        indent=2,
    )
)
PY

export V4_R3_INTERNAL_TESTS_PASSED="$passed"
export V4_R3_INTERNAL_TESTS_FAILED="$failed"

if [[ "$proven" != true ]]; then
  echo "error: leaf-model R3-internal verification failed (cargo test)" >&2
  exit 1
fi

echo "leaf-model R3-internal verification PROVEN"
