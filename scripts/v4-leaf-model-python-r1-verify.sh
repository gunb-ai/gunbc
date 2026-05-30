#!/usr/bin/env bash
# scripts/v4-leaf-model-python-r1-verify.sh
#
# Phase 1 leaf-model verification host runner — python.dag R1 (int surface spelling).
# Authority: src/v4/lens/leaf_model_verification.dag (fixture sources) +
#   src/v4/test/claim/language_model/python_r1.dag (claim + receipt shape).
#
# Exercises CPython on happy + falsification fixtures; emits JSON receipt.
# Falsification probe expects NameError (undefined `i32` annotation) per W2.6 mirror of
# rust.dag R1 (wrong surface spelling for the claimed primitive).
#
# Dissolve-on-arrival: delete when T-22 modeled `run_target_verification` owns python3
# invocation and structured TestClaimRun verdicts replace this host bridge (same
# trigger as src/v3/compiler/tests/boundary/v4_leaf_model_python_r1_test.rs).

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
if [[ ! -f "$fixture_dag" ]]; then
  echo "error: missing fixture authority at $fixture_dag" >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "error: python3 not on PATH — required for leaf-model python R1 verification" >&2
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

happy = extract("python_r1_happy_fixture_source")
falsification = extract("python_r1_falsification_fixture_source")
print(f"happy_source={shlex.quote(happy)}")
print(f"falsification_source={shlex.quote(falsification)}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
tmp_root="${RUNNER_TEMP:-/tmp}"
scratch="${tmp_root}/v4-leaf-model-python-r1-${run_suffix}"
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

echo "=== leaf-model R1: python3 happy fixture (python.dag int surface spelling) ==="
happy_stderr="$(exercise_python_fixture happy "$happy_source")"
happy_status="$(cat "${scratch}/happy.exit")"

echo "=== leaf-model R1: python3 falsification fixture (NameError probe) ==="
falsification_stderr="$(exercise_python_fixture falsification "$falsification_source")"
falsification_status="$(cat "${scratch}/falsification.exit")"

happy_pass=false
falsification_pass=false
if [[ "$happy_status" -eq 0 ]]; then
  happy_pass=true
fi
if [[ "$falsification_status" -ne 0 ]] && grep -qE 'NameError: name .i32. is not defined' <<<"$falsification_stderr"; then
  falsification_pass=true
fi

proven=false
if [[ "$happy_pass" == true && "$falsification_pass" == true ]]; then
  proven=true
fi

export V4_PYTHON_R1_HAPPY_STATUS="$happy_status"
export V4_PYTHON_R1_FALSIFICATION_STATUS="$falsification_status"
export V4_PYTHON_R1_HAPPY_PASS="$happy_pass"
export V4_PYTHON_R1_FALSIFICATION_PASS="$falsification_pass"
export V4_PYTHON_R1_PROVEN="$proven"
export V4_PYTHON_R1_FALSIFICATION_STDERR="$falsification_stderr"

python3 - <<'PY'
import json
import os
import re

stderr = os.environ.get("V4_PYTHON_R1_FALSIFICATION_STDERR", "")
print(
    json.dumps(
        {
            "schema": "scripts/v4-leaf-model-python-r1-verify.sh::host_receipt_v1",
            "claim_id": "PythonR1IntSurfaceSpelling",
            "happy": {
                "python_exit": int(os.environ["V4_PYTHON_R1_HAPPY_STATUS"]),
                "verdict": "Pass"
                if os.environ["V4_PYTHON_R1_HAPPY_PASS"] == "true"
                else "Fail",
            },
            "falsification": {
                "python_exit": int(os.environ["V4_PYTHON_R1_FALSIFICATION_STATUS"]),
                "expected_diagnostic": "NameError:i32",
                "name_error_i32_observed": bool(
                    re.search(r"NameError: name 'i32' is not defined", stderr)
                ),
                "verdict": "Pass"
                if os.environ["V4_PYTHON_R1_FALSIFICATION_PASS"] == "true"
                else "Fail",
            },
            "proven": os.environ["V4_PYTHON_R1_PROVEN"] == "true",
        },
        indent=2,
    )
)
PY

if [[ "$proven" != true ]]; then
  echo "error: leaf-model python R1 verification failed (happy_pass=${happy_pass} falsification_pass=${falsification_pass})" >&2
  if [[ "$happy_pass" != true ]]; then
    echo "--- happy python stderr ---" >&2
    printf '%s\n' "$happy_stderr" >&2
  fi
  if [[ "$falsification_pass" != true ]]; then
    echo "--- falsification python stderr ---" >&2
    printf '%s\n' "$falsification_stderr" >&2
  fi
  exit 1
fi

if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  {
    echo "### Leaf-model verification — python.dag R1 (int surface spelling)"
    echo ""
    echo "| path | verdict |"
    echo "| --- | --- |"
    echo "| happy \`def r1_test() -> int: return 0\` | Pass (python3 clean) |"
    echo "| falsification \`def r1_test() -> i32: ...\` | Pass (NameError on i32) |"
    echo ""
    echo "_Authority: \`${fixture_dag}\` + \`src/v4/test/claim/language_model/python_r1.dag\`_"
  } >> "$GITHUB_STEP_SUMMARY"
fi

echo "leaf-model python R1 verification PROVEN: happy runs clean; falsification rejected with NameError on i32"
