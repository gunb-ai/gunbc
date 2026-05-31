#!/usr/bin/env bash
# scripts/v4-leaf-model-python-r3-external-verify.sh
#
# Phase 1 leaf-model R3-external — Symbol atom realization projection vs CPython.

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

happy = extract("python_r3_external_happy_fixture_source")
falsification = extract("python_r3_external_falsification_fixture_source")
print(f"happy_source={shlex.quote(happy)}")
print(f"falsification_source={shlex.quote(falsification)}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-python-r3-external-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

exercise_python_fixture() {
  local label="$1"
  local source="$2"
  local src_path="${scratch}/${label}.py"
  local stderr_path="${scratch}/${label}.stderr"
  printf '%s' "$source" >"$src_path"
  set +e
  python3 -m py_compile "$src_path" 2>"$stderr_path"
  local py_compile_status=$?
  if [[ "$py_compile_status" -ne 0 ]]; then
    echo "$py_compile_status" >"${scratch}/${label}.exit"
    cat "$stderr_path"
    return
  fi
  python3 "$src_path" 2>>"$stderr_path"
  local exec_status=$?
  set -e
  echo "$exec_status" >"${scratch}/${label}.exit"
  cat "$stderr_path"
}

happy_stderr="$(exercise_python_fixture happy "$happy_source")"
happy_status="$(cat "${scratch}/happy.exit")"
falsification_stderr="$(exercise_python_fixture falsification "$falsification_source")"
falsification_status="$(cat "${scratch}/falsification.exit")"

happy_pass=false
falsification_pass=false
[[ "$happy_status" -eq 0 ]] && happy_pass=true
[[ "$falsification_status" -ne 0 ]] && grep -qE 'TypeError.*Symbol\.__init__\(\)' <<<"$falsification_stderr" && falsification_pass=true

proven=false
[[ "$happy_pass" == true && "$falsification_pass" == true ]] && proven=true

export V4_PYTHON_R3_HAPPY_STATUS="$happy_status"
export V4_PYTHON_R3_FALSIFICATION_STATUS="$falsification_status"
export V4_PYTHON_R3_HAPPY_PASS="$happy_pass"
export V4_PYTHON_R3_FALSIFICATION_PASS="$falsification_pass"
export V4_PYTHON_R3_PROVEN="$proven"
export V4_PYTHON_R3_FALSIFICATION_STDERR="$falsification_stderr"

python3 - <<'PY'
import json
import os
import re

stderr = os.environ.get("V4_PYTHON_R3_FALSIFICATION_STDERR", "")
print(
    json.dumps(
        {
            "schema": "scripts/v4-leaf-model-python-r3-external-verify.sh::host_receipt_v1",
            "claim_id": "PythonR3ExternalSymbolProjection",
            "happy": {
                "python_exit": int(os.environ["V4_PYTHON_R3_HAPPY_STATUS"]),
                "verdict": "Pass" if os.environ["V4_PYTHON_R3_HAPPY_PASS"] == "true" else "Fail",
            },
            "falsification": {
                "python_exit": int(os.environ["V4_PYTHON_R3_FALSIFICATION_STATUS"]),
                "expected_diagnostic": "TypeError:Symbol.__init__",
                "type_error_init_observed": bool(
                    re.search(r"TypeError.*Symbol\.__init__\(\)", stderr)
                ),
                "verdict": "Pass"
                if os.environ["V4_PYTHON_R3_FALSIFICATION_PASS"] == "true"
                else "Fail",
            },
            "proven": os.environ["V4_PYTHON_R3_PROVEN"] == "true",
        },
        indent=2,
    )
)
PY

if [[ "$proven" != true ]]; then
  echo "error: leaf-model python R3-external verification failed" >&2
  exit 1
fi

echo "leaf-model python R3-external verification PROVEN"
