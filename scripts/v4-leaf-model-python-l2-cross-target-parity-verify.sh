#!/usr/bin/env bash
# scripts/v4-leaf-model-python-l2-cross-target-parity-verify.sh
#
# PY-L2-CROSS-TARGET-BEHAVIORAL-PARITY - common-domain fixture receipts for
# Python vs Rust vs Go. This is behavioral parity, not static analysis.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
if [[ ! -f "$fixture_dag" ]]; then
  echo "error: missing fixture authority at $fixture_dag" >&2
  exit 1
fi
for tool in python3 rustc go; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: $tool not on PATH" >&2
    exit 1
  fi
done

eval "$(python3 - "$fixture_dag" <<'PY'
from __future__ import annotations

import re, shlex, sys
from pathlib import Path

text = Path(sys.argv[1]).read_text()
def extract(name: str) -> str:
    pattern = rf'^data {name}: String = "(.*)"\s*$'
    for line in text.splitlines():
        m = re.match(pattern, line)
        if m:
            return bytes(m.group(1), "utf-8").decode("unicode_escape")
    raise SystemExit(f"missing {name}")

for name, var in [
    ("python_l2_parity_python_source", "python_source"),
    ("python_l2_parity_rust_source", "rust_source"),
    ("python_l2_parity_go_source", "go_source"),
    ("python_l2_parity_expected_stdout", "expected_stdout"),
]:
    print(f"{var}={shlex.quote(extract(name))}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-python-l2-cross-target-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

printf '%s' "$python_source" >"${scratch}/fixture.py"
printf '%s' "$rust_source" >"${scratch}/fixture.rs"
printf '%s' "$go_source" >"${scratch}/fixture.go"

python3 -m py_compile "${scratch}/fixture.py"
python_stdout="$(python3 "${scratch}/fixture.py")"

env -u RUSTC_BOOTSTRAP rustc --edition=2021 "${scratch}/fixture.rs" -o "${scratch}/fixture-rust"
rust_stdout="$("${scratch}/fixture-rust")"

go_stdout="$(cd "$scratch" && go run fixture.go)"

trimmed_expected="${expected_stdout%$'\n'}"
python_pass=false
rust_pass=false
go_pass=false
[[ "$python_stdout" == "$trimmed_expected" ]] && python_pass=true
[[ "$rust_stdout" == "$trimmed_expected" ]] && rust_pass=true
[[ "$go_stdout" == "$trimmed_expected" ]] && go_pass=true

proven=false
[[ "$python_pass" == true && "$rust_pass" == true && "$go_pass" == true ]] && proven=true

export V4_L2_EXPECTED="$trimmed_expected"
export V4_L2_PYTHON_STDOUT="$python_stdout"
export V4_L2_RUST_STDOUT="$rust_stdout"
export V4_L2_GO_STDOUT="$go_stdout"
export V4_L2_PYTHON_PASS="$python_pass"
export V4_L2_RUST_PASS="$rust_pass"
export V4_L2_GO_PASS="$go_pass"
export V4_L2_PROVEN="$proven"

python3 - <<'PY'
import json, os

def b(name): return os.environ[name] == "true"
print(json.dumps({
    "schema": "scripts/v4-leaf-model-python-l2-cross-target-parity-verify.sh::parity_receipt_v1",
    "claim_id": "PythonL2CrossTargetBehavioralParityCommonDomain",
    "targets": ["python3", "rustc", "go"],
    "expected_stdout": os.environ["V4_L2_EXPECTED"],
    "observed_stdout": {
        "python": os.environ["V4_L2_PYTHON_STDOUT"],
        "rust": os.environ["V4_L2_RUST_STDOUT"],
        "go": os.environ["V4_L2_GO_STDOUT"],
    },
    "target_pass": {
        "python": b("V4_L2_PYTHON_PASS"),
        "rust": b("V4_L2_RUST_PASS"),
        "go": b("V4_L2_GO_PASS"),
    },
    "proven": b("V4_L2_PROVEN"),
}, indent=2))
PY

if [[ "$proven" != true ]]; then
  echo "error: python L2 cross-target parity failed" >&2
  exit 1
fi

echo "leaf-model python L2 cross-target behavioral parity PROVEN (python/rust/go common domain)"
