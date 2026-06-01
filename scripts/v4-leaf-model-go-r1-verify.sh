#!/usr/bin/env bash
# scripts/v4-leaf-model-go-r1-verify.sh
#
# Phase 1 leaf-model verification host runner — go.dag R1 (int surface spelling).
# Authority: src/v4/lens/leaf_model_verification.dag + go_r1.dag claim wiring.
#
# Dissolve-on-arrival: delete when T-22 modeled `run_target_verification` owns
# go build invocation and structured TestClaimRun verdicts replace this host bridge.

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

print(f"happy_source={shlex.quote(extract('go_r1_happy_fixture_source'))}")
print(f"falsification_source={shlex.quote(extract('go_r1_falsification_fixture_source'))}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-go-r1-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

exercise_go_build() {
  local label="$1"
  local source="$2"
  local src_path="${scratch}/${label}.go"
  local stderr_path="${scratch}/${label}.stderr"
  printf '%s' "$source" >"$src_path"
  set +e
  GO111MODULE=off go build -o "${scratch}/${label}.out" "$src_path" 2>"$stderr_path"
  local status=$?
  set -e
  echo "$status" >"${scratch}/${label}.exit"
  cat "$stderr_path"
}

happy_stderr="$(exercise_go_build happy "$happy_source")"
happy_status="$(cat "${scratch}/happy.exit")"
falsification_stderr="$(exercise_go_build falsification "$falsification_source")"
falsification_status="$(cat "${scratch}/falsification.exit")"

happy_pass=false
falsification_pass=false
[[ "$happy_status" -eq 0 ]] && happy_pass=true
[[ "$falsification_status" -ne 0 ]] && grep -qE 'undefined: i32|undefined type: i32' <<<"$falsification_stderr" && falsification_pass=true

proven=false
[[ "$happy_pass" == true && "$falsification_pass" == true ]] && proven=true

export V4_GO_R1_HAPPY_STATUS="$happy_status"
export V4_GO_R1_FALSIFICATION_STATUS="$falsification_status"
export V4_GO_R1_HAPPY_PASS="$happy_pass"
export V4_GO_R1_FALSIFICATION_PASS="$falsification_pass"
export V4_GO_R1_PROVEN="$proven"
export V4_GO_R1_FALSIFICATION_STDERR="$falsification_stderr"

python3 - <<'PY'
import json
import os
import re

stderr = os.environ.get("V4_GO_R1_FALSIFICATION_STDERR", "")
print(json.dumps({
    "schema": "scripts/v4-leaf-model-go-r1-verify.sh::host_receipt_v1",
    "claim_id": "GoR1IntSurfaceSpelling",
    "happy": {
        "go_build_exit": int(os.environ["V4_GO_R1_HAPPY_STATUS"]),
        "verdict": "Pass" if os.environ["V4_GO_R1_HAPPY_PASS"] == "true" else "Fail",
    },
    "falsification": {
        "go_build_exit": int(os.environ["V4_GO_R1_FALSIFICATION_STATUS"]),
        "expected_diagnostic": "undefined:i32",
        "undefined_type_observed": bool(re.search(r"undefined: i32|undefined type: i32", stderr)),
        "verdict": "Pass" if os.environ["V4_GO_R1_FALSIFICATION_PASS"] == "true" else "Fail",
    },
    "proven": os.environ["V4_GO_R1_PROVEN"] == "true",
}, indent=2))
PY

if [[ "$proven" != true ]]; then
  echo "error: leaf-model Go R1 verification failed" >&2
  exit 1
fi

echo "leaf-model Go R1 verification PROVEN"
