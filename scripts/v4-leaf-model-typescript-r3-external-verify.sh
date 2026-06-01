#!/usr/bin/env bash
# scripts/v4-leaf-model-typescript-r3-external-verify.sh
#
# Phase 1 leaf-model R3-external — ECMA Symbol() factory vs `new Symbol` (not constructable).

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
if [[ ! -f "$fixture_dag" ]]; then
  echo "error: missing fixture authority at $fixture_dag" >&2
  exit 1
fi

if ! command -v npx >/dev/null 2>&1; then
  echo "error: npx not on PATH" >&2
  exit 1
fi

TSC_PKG="typescript@5.9.2"

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

happy = extract("typescript_r3_external_happy_fixture_source")
falsification = extract("typescript_r3_external_falsification_fixture_source")
print(f"happy_source={shlex.quote(happy)}")
print(f"falsification_source={shlex.quote(falsification)}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-typescript-r3-external-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

exercise_tsc_fixture() {
  local label="$1"
  local source="$2"
  local src_path="${scratch}/${label}.ts"
  local diag_path="${scratch}/${label}.diag"
  printf '%s' "$source" >"$src_path"
  set +e
  npx -p "$TSC_PKG" tsc --strict --noEmit --target ES2022 --module ES2022 "$src_path" >"$diag_path" 2>&1
  local status=$?
  set -e
  echo "$status" >"${scratch}/${label}.exit"
  cat "$diag_path"
}

happy_diag="$(exercise_tsc_fixture happy "$happy_source")"
happy_status="$(cat "${scratch}/happy.exit")"
falsification_diag="$(exercise_tsc_fixture falsification "$falsification_source")"
falsification_status="$(cat "${scratch}/falsification.exit")"

happy_pass=false
falsification_pass=false
[[ "$happy_status" -eq 0 ]] && happy_pass=true
[[ "$falsification_status" -ne 0 ]] && grep -qE 'TS7009|construct' <<<"$falsification_diag" && falsification_pass=true

proven=false
[[ "$happy_pass" == true && "$falsification_pass" == true ]] && proven=true

export V4_TS_R3_HAPPY_STATUS="$happy_status"
export V4_TS_R3_FALSIFICATION_STATUS="$falsification_status"
export V4_TS_R3_HAPPY_PASS="$happy_pass"
export V4_TS_R3_FALSIFICATION_PASS="$falsification_pass"
export V4_TS_R3_PROVEN="$proven"
export V4_TS_R3_FALSIFICATION_DIAG="$falsification_diag"

python3 - <<'PY'
import json
import os
import re

stderr = os.environ.get("V4_TS_R3_FALSIFICATION_DIAG", "")
print(
    json.dumps(
        {
            "schema": "scripts/v4-leaf-model-typescript-r3-external-verify.sh::host_receipt_v1",
            "claim_id": "TsR3ExternalSymbolProjection",
            "happy": {
                "tsc_exit": int(os.environ["V4_TS_R3_HAPPY_STATUS"]),
                "verdict": "Pass" if os.environ["V4_TS_R3_HAPPY_PASS"] == "true" else "Fail",
            },
            "falsification": {
                "tsc_exit": int(os.environ["V4_TS_R3_FALSIFICATION_STATUS"]),
                "expected_authority": "compile",
                "expected_diagnostic": "TS7009:new Symbol not constructable",
                "not_constructable_observed": bool(
                    re.search(r"TS7009|construct", stderr)
                ),
                "verdict": "Pass"
                if os.environ["V4_TS_R3_FALSIFICATION_PASS"] == "true"
                else "Fail",
            },
            "proven": os.environ["V4_TS_R3_PROVEN"] == "true",
        },
        indent=2,
    )
)
PY

if [[ "$proven" != true ]]; then
  echo "error: leaf-model typescript R3-external verification failed" >&2
  exit 1
fi

echo "leaf-model typescript R3-external verification PROVEN"
