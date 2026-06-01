#!/usr/bin/env bash
# scripts/v4-leaf-model-go-r3-external-verify.sh
#
<<<<<<< HEAD
# Phase 1 leaf-model R3-external — Symbol atom realization projection vs Go compiler.
# Authority: src/v4/lens/leaf_model_verification.dag + go_r3_external.dag.
=======
# Phase 1 leaf-model R3-external — Symbol atom realization projection vs go build.
# Authority: src/v4/lens/leaf_model_verification.dag + go_r3_external.dag.
#
# Dissolve-on-arrival: delete when T-22 modeled `run_target_verification` owns
# go build invocation and structured TestClaimRun verdicts replace this host bridge.
>>>>>>> 3b5433c445 (Go leaf-model: R1/R2a/R2b/R3-external claims, lens fixtures, verify scripts)

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

<<<<<<< HEAD
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
=======
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
>>>>>>> 3b5433c445 (Go leaf-model: R1/R2a/R2b/R3-external claims, lens fixtures, verify scripts)
falsification_status="$(cat "${scratch}/falsification.exit")"

happy_pass=false
falsification_pass=false
[[ "$happy_status" -eq 0 ]] && happy_pass=true
<<<<<<< HEAD
[[ "$falsification_status" -ne 0 ]] && grep -qE 'too many arguments in conversion to string' <<<"$falsification_diag" && falsification_pass=true
=======
[[ "$falsification_status" -ne 0 ]] && grep -qE 'cannot convert|invalid operation|not a type|too many arguments' <<<"$falsification_stderr" && falsification_pass=true
>>>>>>> 3b5433c445 (Go leaf-model: R1/R2a/R2b/R3-external claims, lens fixtures, verify scripts)

proven=false
[[ "$happy_pass" == true && "$falsification_pass" == true ]] && proven=true

<<<<<<< HEAD
export V4_GO_R3_HAPPY_STATUS="$happy_status"
export V4_GO_R3_FALSIFICATION_STATUS="$falsification_status"
export V4_GO_R3_HAPPY_PASS="$happy_pass"
export V4_GO_R3_FALSIFICATION_PASS="$falsification_pass"
export V4_GO_R3_PROVEN="$proven"
export V4_GO_R3_FALSIFICATION_DIAG="$falsification_diag"
=======
export V4_GO_R3_EXTERNAL_HAPPY_STATUS="$happy_status"
export V4_GO_R3_EXTERNAL_FALSIFICATION_STATUS="$falsification_status"
export V4_GO_R3_EXTERNAL_HAPPY_PASS="$happy_pass"
export V4_GO_R3_EXTERNAL_FALSIFICATION_PASS="$falsification_pass"
export V4_GO_R3_EXTERNAL_PROVEN="$proven"
export V4_GO_R3_EXTERNAL_FALSIFICATION_STDERR="$falsification_stderr"
>>>>>>> 3b5433c445 (Go leaf-model: R1/R2a/R2b/R3-external claims, lens fixtures, verify scripts)

python3 - <<'PY'
import json
import os
import re

<<<<<<< HEAD
diag = os.environ.get("V4_GO_R3_FALSIFICATION_DIAG", "")
=======
stderr = os.environ.get("V4_GO_R3_EXTERNAL_FALSIFICATION_STDERR", "")
>>>>>>> 3b5433c445 (Go leaf-model: R1/R2a/R2b/R3-external claims, lens fixtures, verify scripts)
print(json.dumps({
    "schema": "scripts/v4-leaf-model-go-r3-external-verify.sh::host_receipt_v1",
    "claim_id": "GoR3ExternalSymbolProjection",
    "happy": {
<<<<<<< HEAD
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
=======
        "go_build_exit": int(os.environ["V4_GO_R3_EXTERNAL_HAPPY_STATUS"]),
        "verdict": "Pass" if os.environ["V4_GO_R3_EXTERNAL_HAPPY_PASS"] == "true" else "Fail",
    },
    "falsification": {
        "go_build_exit": int(os.environ["V4_GO_R3_EXTERNAL_FALSIFICATION_STATUS"]),
        "expected_diagnostic": "not_callable_or_invalid_conversion",
        "diagnostic_observed": bool(re.search(r"cannot convert|invalid operation|not a type|too many arguments", stderr)),
        "verdict": "Pass" if os.environ["V4_GO_R3_EXTERNAL_FALSIFICATION_PASS"] == "true" else "Fail",
    },
    "proven": os.environ["V4_GO_R3_EXTERNAL_PROVEN"] == "true",
>>>>>>> 3b5433c445 (Go leaf-model: R1/R2a/R2b/R3-external claims, lens fixtures, verify scripts)
}, indent=2))
PY

if [[ "$proven" != true ]]; then
<<<<<<< HEAD
  echo "error: leaf-model go R3-external verification failed" >&2
  exit 1
fi

echo "leaf-model go R3-external verification PROVEN"
=======
  echo "error: leaf-model Go R3-external verification failed" >&2
  exit 1
fi

echo "leaf-model Go R3-external verification PROVEN"
>>>>>>> 3b5433c445 (Go leaf-model: R1/R2a/R2b/R3-external claims, lens fixtures, verify scripts)
