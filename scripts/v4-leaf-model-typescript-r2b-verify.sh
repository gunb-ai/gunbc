#!/usr/bin/env bash
# scripts/v4-leaf-model-typescript-r2b-verify.sh
#
# Phase 1 leaf-model R2b — bigint runtime add beyond safe integer vs number-lane divergence.
# Authority: src/v4/lens/leaf_model_verification.dag + typescript_r2b.dag.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
if [[ ! -f "$fixture_dag" ]]; then
  echo "error: missing fixture authority at $fixture_dag" >&2
  exit 1
fi

if ! command -v node >/dev/null 2>&1; then
  echo "error: node not on PATH" >&2
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

print(f"happy_source={shlex.quote(extract('typescript_r2b_runtime_happy_fixture_source'))}")
print(f"falsification_source={shlex.quote(extract('typescript_r2b_runtime_falsification_fixture_source'))}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-typescript-r2b-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

exercise_node_fixture() {
  local label="$1"
  local source="$2"
  local src_path="${scratch}/${label}.mjs"
  local stderr_path="${scratch}/${label}.stderr"
  printf '%s' "$source" >"$src_path"
  set +e
  node "$src_path" 2>"$stderr_path"
  local status=$?
  set -e
  echo "$status" >"${scratch}/${label}.exit"
  cat "$stderr_path"
}

happy_stderr="$(exercise_node_fixture happy "$happy_source")"
happy_status="$(cat "${scratch}/happy.exit")"
falsification_stderr="$(exercise_node_fixture falsification "$falsification_source")"
falsification_status="$(cat "${scratch}/falsification.exit")"

runtime_happy=false
falsification_pass=false
[[ "$happy_status" -eq 0 ]] && runtime_happy=true
[[ "$falsification_status" -eq 0 ]] && falsification_pass=true

proven=false
[[ "$runtime_happy" == true && "$falsification_pass" == true ]] && proven=true

export V4_TS_R2B_HAPPY_STATUS="$happy_status"
export V4_TS_R2B_FALSIFICATION_STATUS="$falsification_status"
export V4_TS_R2B_RUNTIME_HAPPY="$runtime_happy"
export V4_TS_R2B_FALSIFICATION_PASS="$falsification_pass"
export V4_TS_R2B_PROVEN="$proven"

python3 - <<'PY'
import json
import os

print(
    json.dumps(
        {
            "schema": "scripts/v4-leaf-model-typescript-r2b-verify.sh::host_receipt_v1",
            "claim_id": "TsR2bBigintBeyondSafeInteger",
            "runtime": {
                "happy_exit": int(os.environ["V4_TS_R2B_HAPPY_STATUS"]),
                "falsification_exit": int(os.environ["V4_TS_R2B_FALSIFICATION_STATUS"]),
                "verdict": "Pass"
                if os.environ["V4_TS_R2B_PROVEN"] == "true"
                else "Fail",
            },
            "proven": os.environ["V4_TS_R2B_PROVEN"] == "true",
        },
        indent=2,
    )
)
PY

if [[ "$proven" != true ]]; then
  echo "error: leaf-model typescript R2b verification failed (runtime_happy=${runtime_happy})" >&2
  cat "$happy_stderr" "$falsification_stderr" >&2
  exit 1
fi

echo "leaf-model typescript R2b verification PROVEN (bigint add beyond 2**63)"
