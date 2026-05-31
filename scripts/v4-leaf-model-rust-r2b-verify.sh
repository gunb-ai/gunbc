#!/usr/bin/env bash
# scripts/v4-leaf-model-rust-r2b-verify.sh
#
# Phase 1 leaf-model R2b — debug overflow panic + release wrap (live sub-claims 1–2).
# Authority: src/v4/lens/leaf_model_verification.dag + rust_r2b.dag.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
if [[ ! -f "$fixture_dag" ]]; then
  echo "error: missing fixture authority at $fixture_dag" >&2
  exit 1
fi

if ! command -v rustc >/dev/null 2>&1; then
  echo "error: rustc not on PATH" >&2
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

print(f"source={shlex.quote(extract('rust_r2b_runtime_fixture_source'))}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-rust-r2b-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

src_path="${scratch}/r2b.rs"
printf '%s' "$source" >"$src_path"

build_and_run() {
  local label="$1"
  shift
  local bin="${scratch}/${label}"
  local stderr_path="${scratch}/${label}.stderr"
  set +e
  env -u RUSTC_BOOTSTRAP rustc --edition=2021 --crate-type bin "$@" "$src_path" -o "$bin" 2>"$stderr_path"
  local compile_status=$?
  local run_status=127
  if [[ "$compile_status" -eq 0 ]]; then
    "$bin" 2>>"$stderr_path"
    run_status=$?
  fi
  set -e
  echo "$compile_status" >"${scratch}/${label}.compile_exit"
  echo "$run_status" >"${scratch}/${label}.run_exit"
  cat "$stderr_path"
}

debug_stderr="$(build_and_run debug)"
debug_compile="$(cat "${scratch}/debug.compile_exit")"
debug_run="$(cat "${scratch}/debug.run_exit")"

release_stderr="$(build_and_run release -C opt-level=2)"
release_compile="$(cat "${scratch}/release.compile_exit")"
release_run="$(cat "${scratch}/release.run_exit")"

debug_happy=false
release_happy=false
[[ "$debug_compile" -eq 0 && "$debug_run" -ne 0 ]] && debug_happy=true
[[ "$release_compile" -eq 0 && "$release_run" -eq 0 ]] && release_happy=true

proven=false
[[ "$debug_happy" == true && "$release_happy" == true ]] && proven=true

export V4_R2B_DEBUG_COMPILE="$debug_compile" V4_R2B_DEBUG_RUN="$debug_run"
export V4_R2B_RELEASE_COMPILE="$release_compile" V4_R2B_RELEASE_RUN="$release_run"
export V4_R2B_DEBUG_HAPPY="$debug_happy" V4_R2B_RELEASE_HAPPY="$release_happy"
export V4_R2B_PROVEN="$proven"

python3 - <<'PY'
import json
import os

print(
    json.dumps(
        {
            "schema": "scripts/v4-leaf-model-rust-r2b-verify.sh::host_receipt_v1",
            "debug_default": {
                "compile_exit": int(os.environ["V4_R2B_DEBUG_COMPILE"]),
                "run_exit": int(os.environ["V4_R2B_DEBUG_RUN"]),
                "expected_outcome": "Panic",
                "verdict": "Pass" if os.environ["V4_R2B_DEBUG_HAPPY"] == "true" else "Fail",
            },
            "release_default": {
                "compile_exit": int(os.environ["V4_R2B_RELEASE_COMPILE"]),
                "run_exit": int(os.environ["V4_R2B_RELEASE_RUN"]),
                "expected_outcome": "Wrap",
                "verdict": "Pass" if os.environ["V4_R2B_RELEASE_HAPPY"] == "true" else "Fail",
            },
            "proven": os.environ["V4_R2B_PROVEN"] == "true",
        },
        indent=2,
    )
)
PY

if [[ "$proven" != true ]]; then
  echo "error: leaf-model R2b verification failed (debug_happy=${debug_happy} release_happy=${release_happy})" >&2
  exit 1
fi

echo "leaf-model R2b verification PROVEN (debug panic + release wrap)"
