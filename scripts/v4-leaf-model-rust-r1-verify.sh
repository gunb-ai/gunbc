#!/usr/bin/env bash
# scripts/v4-leaf-model-rust-r1-verify.sh
#
# Phase 1 leaf-model verification host runner — rust.dag R1 (i32 surface spelling).
# Authority: src/v4/lens/leaf_model_verification.dag (fixture sources) +
#   src/v4/test/claim/language_model/rust_r1.dag (claim + receipt shape).
#
# Exercises rustc on happy + falsification fixtures; emits JSON receipt.
# Falsification probe expects E0308 (type mismatch) per
# docs/planning/v4-leaf-model-verification-2026-05-30.md §7.
#
# Dissolve-on-arrival: delete when T-22 modeled `run_target_verification` owns rustc
# invocation and structured TestClaimRun verdicts replace this host bridge (same
# trigger as src/v3/compiler/tests/boundary/v4_leaf_model_rust_r1_rustc_test.rs).

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
if [[ ! -f "$fixture_dag" ]]; then
  echo "error: missing fixture authority at $fixture_dag" >&2
  exit 1
fi

if ! command -v rustc >/dev/null 2>&1; then
  echo "error: rustc not on PATH — install a Rust toolchain to run leaf-model R1 verification" >&2
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

happy = extract("rust_r1_happy_fixture_source")
falsification = extract("rust_r1_falsification_fixture_source")
print(f"happy_source={shlex.quote(happy)}")
print(f"falsification_source={shlex.quote(falsification)}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
tmp_root="${RUNNER_TEMP:-/tmp}"
scratch="${tmp_root}/v4-leaf-model-rust-r1-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

compile_rust_lib() {
  local label="$1"
  local source="$2"
  local src_path="${scratch}/${label}.rs"
  local stderr_path="${scratch}/${label}.stderr"
  printf '%s' "$source" >"$src_path"
  set +e
  env -u RUSTC_BOOTSTRAP rustc \
    --edition=2021 \
    --crate-type lib \
    "$src_path" \
    -o "${scratch}/${label}.rlib" \
    2>"$stderr_path"
  local status=$?
  set -e
  echo "$status" >"${scratch}/${label}.exit"
  cat "$stderr_path"
}

echo "=== leaf-model R1: rustc happy fixture (rust.dag i32 surface spelling) ==="
happy_stderr="$(compile_rust_lib happy "$happy_source")"
happy_status="$(cat "${scratch}/happy.exit")"

echo "=== leaf-model R1: rustc falsification fixture (E0308 probe) ==="
falsification_stderr="$(compile_rust_lib falsification "$falsification_source")"
falsification_status="$(cat "${scratch}/falsification.exit")"

happy_pass=false
falsification_pass=false
if [[ "$happy_status" -eq 0 ]]; then
  happy_pass=true
fi
if [[ "$falsification_status" -ne 0 ]] && grep -qE 'E0308' <<<"$falsification_stderr"; then
  falsification_pass=true
fi

proven=false
if [[ "$happy_pass" == true && "$falsification_pass" == true ]]; then
  proven=true
fi

export V4_R1_HAPPY_STATUS="$happy_status"
export V4_R1_FALSIFICATION_STATUS="$falsification_status"
export V4_R1_HAPPY_PASS="$happy_pass"
export V4_R1_FALSIFICATION_PASS="$falsification_pass"
export V4_R1_PROVEN="$proven"
export V4_R1_FALSIFICATION_STDERR="$falsification_stderr"

python3 - <<'PY'
import json
import os
import re

stderr = os.environ.get("V4_R1_FALSIFICATION_STDERR", "")
print(
    json.dumps(
        {
            "schema": "scripts/v4-leaf-model-rust-r1-verify.sh::host_receipt_v1",
            "claim_id": "RustR1I32SurfaceSpelling",
            "happy": {
                "rustc_exit": int(os.environ["V4_R1_HAPPY_STATUS"]),
                "verdict": "Pass"
                if os.environ["V4_R1_HAPPY_PASS"] == "true"
                else "Fail",
            },
            "falsification": {
                "rustc_exit": int(os.environ["V4_R1_FALSIFICATION_STATUS"]),
                "expected_diagnostic": "E0308",
                "e0308_observed": bool(re.search(r"E0308", stderr)),
                "verdict": "Pass"
                if os.environ["V4_R1_FALSIFICATION_PASS"] == "true"
                else "Fail",
            },
            "proven": os.environ["V4_R1_PROVEN"] == "true",
        },
        indent=2,
    )
)
PY

if [[ "$proven" != true ]]; then
  echo "error: leaf-model R1 verification failed (happy_pass=${happy_pass} falsification_pass=${falsification_pass})" >&2
  if [[ "$happy_pass" != true ]]; then
    echo "--- happy rustc stderr ---" >&2
    printf '%s\n' "$happy_stderr" >&2
  fi
  if [[ "$falsification_pass" != true ]]; then
    echo "--- falsification rustc stderr ---" >&2
    printf '%s\n' "$falsification_stderr" >&2
  fi
  exit 1
fi

if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  {
    echo "### Leaf-model verification — rust.dag R1 (i32 surface spelling)"
    echo ""
    echo "| path | verdict |"
    echo "| --- | --- |"
    echo "| happy \`pub fn r1_test() -> i32 { 0i32 }\` | Pass (rustc clean) |"
    echo "| falsification \`... { \"string\" }\` | Pass (rustc E0308) |"
    echo ""
    echo "_Authority: \`${fixture_dag}\` + \`src/v4/test/claim/language_model/rust_r1.dag\`_"
  } >> "$GITHUB_STEP_SUMMARY"
fi

echo "leaf-model R1 verification PROVEN: happy compiles; falsification rejected with E0308"
