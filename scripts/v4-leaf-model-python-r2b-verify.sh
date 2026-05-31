#!/usr/bin/env bash
# scripts/v4-leaf-model-python-r2b-verify.sh
#
# Phase 1 leaf-model R2b — int arbitrary-precision add beyond fixed-width.
# Authority: src/v4/lens/leaf_model_verification.dag + python_r2b.dag.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
if [[ ! -f "$fixture_dag" ]]; then
  echo "error: missing fixture authority at $fixture_dag" >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "error: python3 not on PATH" >&2
  exit 1
fi

eval "$(python3 - "$fixture_dag" <<'PY'
from __future__ import annotations

import re
import shlex
import sys
from pathlib import Path

path = Path(sys.argv[1])
text = path.read_text()

def extract(name: str) -> str:
    pattern = rf'^data {name}: String = "(.*)"\s*$'
    for line in text.splitlines():
        m = re.match(pattern, line)
        if m:
            return bytes(m.group(1), "utf-8").decode("unicode_escape")
    raise SystemExit(f"error: {path}: missing data {name}: String = ...")

print(f"source={shlex.quote(extract('python_r2b_runtime_fixture_source'))}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-python-r2b-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

src_path="${scratch}/r2b.py"
stderr_path="${scratch}/r2b.stderr"
printf '%s' "$source" >"$src_path"

set +e
python3 -m py_compile "$src_path" 2>"$stderr_path"
compile_status=$?
run_status=127
if [[ "$compile_status" -eq 0 ]]; then
  python3 "$src_path" 2>>"$stderr_path"
  run_status=$?
fi
set -e

runtime_happy=false
[[ "$compile_status" -eq 0 && "$run_status" -eq 0 ]] && runtime_happy=true

proven=false
[[ "$runtime_happy" == true ]] && proven=true

export V4_PYTHON_R2B_COMPILE="$compile_status"
export V4_PYTHON_R2B_RUN="$run_status"
export V4_PYTHON_R2B_RUNTIME_HAPPY="$runtime_happy"
export V4_PYTHON_R2B_PROVEN="$proven"

python3 - <<'PY'
import json
import os

print(
    json.dumps(
        {
            "schema": "scripts/v4-leaf-model-python-r2b-verify.sh::host_receipt_v1",
            "claim_id": "PythonR2bIntArbitraryPrecisionAdd",
            "runtime": {
                "compile_exit": int(os.environ["V4_PYTHON_R2B_COMPILE"]),
                "run_exit": int(os.environ["V4_PYTHON_R2B_RUN"]),
                "expected_outcome": "ArbitraryPrecisionAdd",
                "verdict": "Pass"
                if os.environ["V4_PYTHON_R2B_RUNTIME_HAPPY"] == "true"
                else "Fail",
            },
            "proven": os.environ["V4_PYTHON_R2B_PROVEN"] == "true",
        },
        indent=2,
    )
)
PY

if [[ "$proven" != true ]]; then
  echo "error: leaf-model python R2b verification failed (runtime_happy=${runtime_happy})" >&2
  cat "$stderr_path" >&2
  exit 1
fi

echo "leaf-model python R2b verification PROVEN (arbitrary-precision add beyond 2**63)"
