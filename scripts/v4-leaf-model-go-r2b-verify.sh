#!/usr/bin/env bash
# scripts/v4-leaf-model-go-r2b-verify.sh
#
# Phase 1 leaf-model R2b — typed int64 overflow truncates silently at runtime.
# Authority: src/v4/lens/leaf_model_verification.dag + go_r2b.dag.
#
# Dissolve-on-arrival: delete when T-22 modeled `run_target_verification` owns
# go run invocation and structured TestClaimRun verdicts replace this host bridge.

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

print(f"happy_source={shlex.quote(extract('go_r2b_happy_fixture_source'))}")
print(f"falsification_source={shlex.quote(extract('go_r2b_falsification_fixture_source'))}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-go-r2b-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

exercise_go_run() {
  local label="$1"
  local source="$2"
  local src_path="${scratch}/${label}.go"
  local stderr_path="${scratch}/${label}.stderr"
  printf '%s' "$source" >"$src_path"
  set +e
  GO111MODULE=off go run "$src_path" 2>"$stderr_path"
  local status=$?
  set -e
  echo "$status" >"${scratch}/${label}.exit"
  cat "$stderr_path"
}

happy_stderr="$(exercise_go_run happy "$happy_source")"
happy_status="$(cat "${scratch}/happy.exit")"
falsification_stderr="$(exercise_go_run falsification "$falsification_source")"
falsification_status="$(cat "${scratch}/falsification.exit")"

happy_pass=false
falsification_pass=false
[[ "$happy_status" -eq 0 ]] && happy_pass=true
[[ "$falsification_status" -ne 0 ]] && grep -qE 'panic: deliberately wrong expected wrap value' <<<"$falsification_stderr" && falsification_pass=true

proven=false
[[ "$happy_pass" == true && "$falsification_pass" == true ]] && proven=true

export V4_GO_R2B_HAPPY_STATUS="$happy_status"
export V4_GO_R2B_FALSIFICATION_STATUS="$falsification_status"
export V4_GO_R2B_HAPPY_PASS="$happy_pass"
export V4_GO_R2B_FALSIFICATION_PASS="$falsification_pass"
export V4_GO_R2B_PROVEN="$proven"
export V4_GO_R2B_FALSIFICATION_STDERR="$falsification_stderr"

python3 - <<'PY'
import json
import os
import re

stderr = os.environ.get("V4_GO_R2B_FALSIFICATION_STDERR", "")
print(json.dumps({
    "schema": "scripts/v4-leaf-model-go-r2b-verify.sh::host_receipt_v1",
    "claim_id": "GoR2bInt64SilentOverflowTruncates",
    "happy": {
        "go_run_exit": int(os.environ["V4_GO_R2B_HAPPY_STATUS"]),
        "expected_outcome": "TypedInt64OverflowWrapsToMinInt64",
        "verdict": "Pass" if os.environ["V4_GO_R2B_HAPPY_PASS"] == "true" else "Fail",
    },
    "falsification": {
        "go_run_exit": int(os.environ["V4_GO_R2B_FALSIFICATION_STATUS"]),
        "expected_diagnostic": "panic:wrong_expected_wrap_value",
        "panic_observed": bool(re.search(r"panic: deliberately wrong expected wrap value", stderr)),
        "verdict": "Pass" if os.environ["V4_GO_R2B_FALSIFICATION_PASS"] == "true" else "Fail",
    },
    "proven": os.environ["V4_GO_R2B_PROVEN"] == "true",
}, indent=2))
PY

if [[ "$proven" != true ]]; then
  echo "error: leaf-model Go R2b verification failed" >&2
  exit 1
fi

echo "leaf-model Go R2b verification PROVEN (typed int64 overflow wraps)"
