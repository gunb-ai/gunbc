#!/usr/bin/env bash
# scripts/v4-leaf-model-python-cross-runtime-drift-verify.sh
#
# PY-L2-CROSS-RUNTIME-DRIFT — positive cross-runtime DIVERGENCE receipt. This is the
# complement of the L2 cross-target PARITY lane: where parity proves runtime-value EQUALITY
# on the common domain, this runner proves that the SAME modeled program (exact integer add
# at the fixed-width boundary, MAX + 1) DIVERGES across runtimes. Python realizes Int as
# arbitrary precision (exact sum); Rust i64 / Go int64 realize it as fixed-width
# two's-complement (defined wraparound). The receipt asserts the drift POSITIVELY so the
# verification system demonstrably DETECTS drift rather than treating wraparound as parity.
#
# Authority: src/v4/lens/leaf_model_verification.dag (python_cross_runtime_drift_*); the
# expected divergence is the modeled ValueDiff<String> (expected = arbitrary-precision value,
# actual = fixed-width wrapped value) consumed from the lens, never re-authored here.
#
# Tool policy: python3 + rustc are MANDATORY (they form the arbitrary-precision-vs-fixed-width
# drift pair the claim is about). Go is a CORROBORATING fixed-width witness (rust == go) and is
# asserted only when the `go` toolchain is present; an offline runner without Go still produces
# an honest receipt that proves the Python-vs-Rust drift.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
if [[ ! -f "$fixture_dag" ]]; then
  echo "error: missing fixture authority at $fixture_dag" >&2
  exit 1
fi
for tool in python3 rustc; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: $tool not on PATH (mandatory for the cross-runtime drift pair)" >&2
    exit 1
  fi
done

go_available=false
command -v go >/dev/null 2>&1 && go_available=true

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
    raise SystemExit(f"error: {sys.argv[1]}: missing data {name}: String = ...")

for name, var in [
    ("python_cross_runtime_drift_python_source", "python_source"),
    ("python_cross_runtime_drift_rust_source", "rust_source"),
    ("python_cross_runtime_drift_go_source", "go_source"),
    ("python_cross_runtime_drift_arbitrary_precision_value", "expected_python_value"),
    ("python_cross_runtime_drift_fixed_width_value", "expected_fixed_width_value"),
]:
    print(f"{var}={shlex.quote(extract(name))}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-python-cross-runtime-drift-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

printf '%s' "$python_source" >"${scratch}/fixture.py"
printf '%s' "$rust_source" >"${scratch}/fixture.rs"
printf '%s' "$go_source" >"${scratch}/fixture.go"

# --- Python: arbitrary precision ---
python3 -m py_compile "${scratch}/fixture.py"
python_value="$(python3 "${scratch}/fixture.py")"

# --- Rust: fixed-width wrapping_add ---
rustc -O "${scratch}/fixture.rs" -o "${scratch}/fixture_rs" >/dev/null
rust_value="$("${scratch}/fixture_rs")"

# --- Go: corroborating fixed-width witness (optional) ---
go_value=""
if [[ "$go_available" == true ]]; then
  ( cd "$scratch" && GOFLAGS= GOCACHE="${scratch}/.gocache" go build -o fixture_go fixture.go ) >/dev/null 2>&1
  go_value="$("${scratch}/fixture_go")"
fi

# --- drift relation ---
python_matches=false
rust_matches=false
go_matches=true
drift_python_vs_rust=false
drift_python_vs_go=true
fixed_width_agree=true

[[ "$python_value" == "$expected_python_value" ]] && python_matches=true
[[ "$rust_value" == "$expected_fixed_width_value" ]] && rust_matches=true
[[ "$python_value" != "$rust_value" ]] && drift_python_vs_rust=true

if [[ "$go_available" == true ]]; then
  go_matches=false
  fixed_width_agree=false
  drift_python_vs_go=false
  [[ "$go_value" == "$expected_fixed_width_value" ]] && go_matches=true
  [[ "$python_value" != "$go_value" ]] && drift_python_vs_go=true
  [[ "$rust_value" == "$go_value" ]] && fixed_width_agree=true
fi

drift_detected=false
if [[ "$python_matches" == true && "$rust_matches" == true \
      && "$drift_python_vs_rust" == true \
      && "$go_matches" == true && "$drift_python_vs_go" == true \
      && "$fixed_width_agree" == true ]]; then
  drift_detected=true
fi

export V4_DRIFT_PYTHON_VALUE="$python_value"
export V4_DRIFT_RUST_VALUE="$rust_value"
export V4_DRIFT_GO_VALUE="$go_value"
export V4_DRIFT_GO_AVAILABLE="$go_available"
export V4_DRIFT_EXPECTED_PYTHON="$expected_python_value"
export V4_DRIFT_EXPECTED_FIXED="$expected_fixed_width_value"
export V4_DRIFT_PYTHON_MATCHES="$python_matches"
export V4_DRIFT_RUST_MATCHES="$rust_matches"
export V4_DRIFT_GO_MATCHES="$go_matches"
export V4_DRIFT_PVR="$drift_python_vs_rust"
export V4_DRIFT_PVG="$drift_python_vs_go"
export V4_DRIFT_FIXED_AGREE="$fixed_width_agree"
export V4_DRIFT_DETECTED="$drift_detected"

python3 - <<'PY'
import json, os

def b(name): return os.environ[name] == "true"

go_available = b("V4_DRIFT_GO_AVAILABLE")
print(json.dumps({
    "schema": "scripts/v4-leaf-model-python-cross-runtime-drift-verify.sh::drift_receipt_v1",
    "claim_id": "PythonCrossRuntimeDriftArbitraryPrecision",
    "kind": "positive-divergence (complement of L2 parity)",
    "modeled_divergence": {
        "expected_arbitrary_precision_value": os.environ["V4_DRIFT_EXPECTED_PYTHON"],
        "expected_fixed_width_value": os.environ["V4_DRIFT_EXPECTED_FIXED"],
        "carrier": "ValueDiff<String> (src/v4/std/host_run.dag) via python_cross_runtime_drift_probe",
    },
    "observed": {
        "python": {"value": os.environ["V4_DRIFT_PYTHON_VALUE"], "matches_expected": b("V4_DRIFT_PYTHON_MATCHES")},
        "rust": {"value": os.environ["V4_DRIFT_RUST_VALUE"], "matches_expected": b("V4_DRIFT_RUST_MATCHES")},
        "go": ({"value": os.environ["V4_DRIFT_GO_VALUE"], "matches_expected": b("V4_DRIFT_GO_MATCHES")}
               if go_available else {"value": None, "matches_expected": "deferred(go-unavailable)"}),
    },
    "drift_relation": {
        "python_vs_rust_diverges": b("V4_DRIFT_PVR"),
        "python_vs_go_diverges": (b("V4_DRIFT_PVG") if go_available else "deferred(go-unavailable)"),
        "rust_vs_go_agree": (b("V4_DRIFT_FIXED_AGREE") if go_available else "deferred(go-unavailable)"),
    },
    "drift_detected": b("V4_DRIFT_DETECTED"),
}, indent=2))
PY

if [[ "$drift_detected" != true ]]; then
  echo "error: leaf-model python cross-runtime drift NOT positively detected (python=${python_value}, rust=${rust_value}, go=${go_value:-<unavailable>})" >&2
  exit 1
fi

if [[ "$go_available" == true ]]; then
  echo "leaf-model python cross-runtime drift DETECTED (python ${python_value} != fixed-width ${rust_value}; rust == go)"
else
  echo "leaf-model python cross-runtime drift DETECTED (python ${python_value} != rust ${rust_value}; go corroboration deferred — toolchain unavailable)"
fi
