#!/usr/bin/env bash
# scripts/v4-leaf-model-go-r3-external-verify.sh
#
# Phase 1 leaf-model R3-external — Symbol atom realization projection vs Go compiler.
# Authority: src/v4/lens/leaf_model_verification.dag + go_r3_external.dag.

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

print(f"happy_source={shlex.quote(extract('go_r3_external_happy_fixture_source'))}")
print(f"falsification_source={shlex.quote(extract('go_r3_external_falsification_fixture_source'))}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-go-r3-external-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

exercise_go_test() {
  local label="$1"
  local source="$2"
  local dir="${scratch}/${label}"
  local diag_path="${scratch}/${label}.diag"
  mkdir -p "$dir"
  printf '%s' "$source" >"${dir}/main.go"
  set +e
  (cd "$dir" && GO111MODULE=off go test ./...) >"$diag_path" 2>&1
  local status=$?
  set -e
  echo "$status" >"${scratch}/${label}.exit"
  cat "$diag_path"
}

happy_diag="$(exercise_go_test happy "$happy_source")"
happy_status="$(cat "${scratch}/happy.exit")"
falsification_diag="$(exercise_go_test falsification "$falsification_source")"
falsification_status="$(cat "${scratch}/falsification.exit")"

happy_pass=false
falsification_pass=false
[[ "$happy_status" -eq 0 ]] && happy_pass=true
[[ "$falsification_status" -ne 0 ]] && grep -qE 'too many arguments in conversion to string' <<<"$falsification_diag" && falsification_pass=true

proven=false
[[ "$happy_pass" == true && "$falsification_pass" == true ]] && proven=true

export V4_GO_R3_HAPPY_STATUS="$happy_status"
export V4_GO_R3_FALSIFICATION_STATUS="$falsification_status"
export V4_GO_R3_HAPPY_PASS="$happy_pass"
export V4_GO_R3_FALSIFICATION_PASS="$falsification_pass"
export V4_GO_R3_PROVEN="$proven"
export V4_GO_R3_FALSIFICATION_DIAG="$falsification_diag"

python3 - <<'PY'
import json
import os
import re

diag = os.environ.get("V4_GO_R3_FALSIFICATION_DIAG", "")
print(json.dumps({
    "schema": "scripts/v4-leaf-model-go-r3-external-verify.sh::host_receipt_v1",
    "claim_id": "GoR3ExternalSymbolProjection",
    "happy": {
        "go_test_exit": int(os.environ["V4_GO_R3_HAPPY_STATUS"]),
        "verdict": "Pass" if os.environ["V4_GO_R3_HAPPY_PASS"] == "true" else "Fail",
    },
    "falsification": {
        "go_test_exit": int(os.environ["V4_GO_R3_FALSIFICATION_STATUS"]),
        "expected_diagnostic": "too_many_arguments_in_conversion_to_string",
        "observed": bool(re.search(r"too many arguments in conversion to string", diag)),
        "verdict": "Pass" if os.environ["V4_GO_R3_FALSIFICATION_PASS"] == "true" else "Fail",
    },
    "proven": os.environ["V4_GO_R3_PROVEN"] == "true",
}, indent=2))
PY

if [[ "$proven" != true ]]; then
  echo "error: leaf-model go R3-external verification failed" >&2
  exit 1
fi

echo "leaf-model go R3-external verification PROVEN"
