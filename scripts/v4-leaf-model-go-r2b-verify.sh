#!/usr/bin/env bash
# scripts/v4-leaf-model-go-r2b-verify.sh
#
# Phase 1 leaf-model R2b — int64 overflow wrap at runtime.
# Authority: src/v4/lens/leaf_model_verification.dag + go_r2b.dag.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
if [[ ! -f "$fixture_dag" ]]; then
  echo "error: missing fixture authority at $fixture_dag" >&2
  exit 1
fi

if ! command -v go >/dev/null 2>&1; then
  echo "error: go not on PATH" >&2
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

print(f"source={shlex.quote(extract('go_r2b_runtime_fixture_source'))}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-go-r2b-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"
src_path="${scratch}/main.go"
diag_path="${scratch}/diag"
printf '%s' "$source" >"$src_path"

set +e
(cd "$scratch" && GO111MODULE=off go run main.go) >"$diag_path" 2>&1
runtime_status=$?
set -e

runtime_happy=false
[[ "$runtime_status" -eq 0 ]] && runtime_happy=true

export V4_GO_R2B_RUNTIME_STATUS="$runtime_status"
export V4_GO_R2B_RUNTIME_HAPPY="$runtime_happy"

python3 - <<'PY'
import json
import os

print(json.dumps({
    "schema": "scripts/v4-leaf-model-go-r2b-verify.sh::host_receipt_v1",
    "claim_id": "GoR2bInt64OverflowWrap",
    "runtime": {
        "go_run_exit": int(os.environ["V4_GO_R2B_RUNTIME_STATUS"]),
        "verdict": "Pass" if os.environ["V4_GO_R2B_RUNTIME_HAPPY"] == "true" else "Fail",
    },
    "proven": os.environ["V4_GO_R2B_RUNTIME_HAPPY"] == "true",
}, indent=2))
PY

if [[ "$runtime_happy" != true ]]; then
  cat "$diag_path" >&2
  echo "error: leaf-model go R2b verification failed" >&2
  exit 1
fi

echo "leaf-model go R2b verification PROVEN (int64 overflow wraps)"
