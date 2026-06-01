#!/usr/bin/env bash
# scripts/v4-leaf-model-python-l1-static-verify.sh
#
# PY-L1-STATIC-STRUCTURAL — pyright static structural analysis as a THIRD, distinct
# verification authority alongside CPython `py_compile` and `python3` runtime exec.
# Authority: Modeling DFS Arbiter ruling proud-fox-405 (msg_41813c03);
# worksheet docs/planning/v4-python-rca-manager-worksheets-2026-06-01.md Worksheet A.
# Fixture authority: src/v4/lens/leaf_model_verification.dag (python_l1_static_*).
#
# F1 (worksheet): the falsification fixture returns a `str` from a `-> int` function.
#   - py_compile ACCEPTS it (syntactically valid)            → compile authority misses.
#   - python3 runs it with exit 0 (the value is never used)  → runtime authority misses.
#   - pyright reports `reportReturnType`                     → static authority CATCHES it.
# This is exactly why static analysis is modeled as TargetStaticAnalysisVerdict, a
# carrier distinct from TargetPythonCompileRejected and TargetPythonExecRejected.
#
# Tool policy: pyright is pinned via npx. If pyright cannot be obtained (offline
# runner), the static step degrades to a recorded "deferred" advisory and the gate
# still proves the compile+runtime authorities miss the defect (F2: tool-unavailable
# is recorded honestly, never reported as a behavioral/runtime failure).

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
pyright_version="1.1.410"
pyright_expected_rule="reportReturnType"

if [[ ! -f "$fixture_dag" ]]; then
  echo "error: missing fixture authority at $fixture_dag" >&2
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "error: python3 not on PATH" >&2
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

happy = extract("python_l1_static_happy_fixture_source")
falsification = extract("python_l1_static_falsification_fixture_source")
print(f"happy_source={shlex.quote(happy)}")
print(f"falsification_source={shlex.quote(falsification)}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-python-l1-static-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

printf '%s' "$happy_source" >"${scratch}/happy.py"
printf '%s' "$falsification_source" >"${scratch}/falsification.py"

# --- compile + runtime authorities: both must MISS the falsification defect ---
compile_authority_misses=true
runtime_authority_misses=true

for label in happy falsification; do
  if ! python3 -m py_compile "${scratch}/${label}.py" 2>"${scratch}/${label}.compile.err"; then
    compile_authority_misses=false
    echo "note: py_compile rejected ${label} (unexpected for L1 static fixture)" >&2
  fi
done
# Falsification must RUN clean — the bad return value is never consumed.
if ! python3 "${scratch}/falsification.py" >/dev/null 2>"${scratch}/falsification.run.err"; then
  runtime_authority_misses=false
fi

# --- static authority: pyright (pinned via npx; degrade gracefully if offline) ---
static_available=false
happy_static_clean=false
falsification_static_rule=""
falsification_static_rejected=false

pyright_run() {
  local target="$1" out="$2"
  if command -v pyright >/dev/null 2>&1; then
    pyright --outputjson "$target" >"$out" 2>/dev/null
  elif command -v npx >/dev/null 2>&1; then
    npx --yes "pyright@${pyright_version}" --outputjson "$target" >"$out" 2>/dev/null
  else
    return 2
  fi
  return 0
}

if pyright_run "${scratch}/happy.py" "${scratch}/happy.pyright.json"; then
  if pyright_run "${scratch}/falsification.py" "${scratch}/falsification.pyright.json"; then
    static_available=true
  fi
fi

if [[ "$static_available" == true ]]; then
  happy_errors="$(python3 - "${scratch}/happy.pyright.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
print(sum(1 for x in d.get("generalDiagnostics", []) if x.get("severity") == "error"))
PY
)"
  [[ "$happy_errors" == "0" ]] && happy_static_clean=true

  falsification_static_rule="$(python3 - "${scratch}/falsification.pyright.json" "$pyright_expected_rule" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
want = sys.argv[2]
rules = [x.get("rule") for x in d.get("generalDiagnostics", []) if x.get("severity") == "error"]
print(want if want in rules else (rules[0] if rules else ""))
PY
)"
  [[ "$falsification_static_rule" == "$pyright_expected_rule" ]] && falsification_static_rejected=true
fi

# --- verdict ---
distinct_authority_proven=false
[[ "$compile_authority_misses" == true && "$runtime_authority_misses" == true ]] && distinct_authority_proven=true

static_proven=false
[[ "$static_available" == true && "$happy_static_clean" == true && "$falsification_static_rejected" == true ]] && static_proven=true

export V4_PYL1_COMPILE_MISSES="$compile_authority_misses"
export V4_PYL1_RUNTIME_MISSES="$runtime_authority_misses"
export V4_PYL1_STATIC_AVAILABLE="$static_available"
export V4_PYL1_HAPPY_CLEAN="$happy_static_clean"
export V4_PYL1_FALS_REJECTED="$falsification_static_rejected"
export V4_PYL1_FALS_RULE="$falsification_static_rule"
export V4_PYL1_DISTINCT_PROVEN="$distinct_authority_proven"
export V4_PYL1_STATIC_PROVEN="$static_proven"
export V4_PYL1_PYRIGHT_VERSION="$pyright_version"
export V4_PYL1_EXPECTED_RULE="$pyright_expected_rule"

python3 - <<'PY'
import json, os

def b(name): return os.environ[name] == "true"

static_available = b("V4_PYL1_STATIC_AVAILABLE")
print(json.dumps({
    "schema": "scripts/v4-leaf-model-python-l1-static-verify.sh::static_receipt_v1",
    "claim_id": "PythonL1StaticReturnType",
    "tool": {"id": "pyright", "version": os.environ["V4_PYL1_PYRIGHT_VERSION"]},
    "authority_separation": {
        "py_compile_accepts_both": b("V4_PYL1_COMPILE_MISSES"),
        "runtime_exec_clean_on_falsification": b("V4_PYL1_RUNTIME_MISSES"),
        "distinct_authority_proven": b("V4_PYL1_DISTINCT_PROVEN"),
    },
    "static_analysis": {
        "available": static_available,
        "role": "BlockingForRung",
        "happy_verdict": ("StaticAnalysisAccepted" if b("V4_PYL1_HAPPY_CLEAN")
                          else ("Deferred(tool-unavailable)" if not static_available else "StaticAnalysisRejected")),
        "falsification_verdict": ("StaticAnalysisRejected:" + os.environ["V4_PYL1_FALS_RULE"] if b("V4_PYL1_FALS_REJECTED")
                                  else ("Deferred(tool-unavailable)" if not static_available else "StaticAnalysisAccepted")),
        "expected_diagnostic_rule": os.environ["V4_PYL1_EXPECTED_RULE"],
        "proven": b("V4_PYL1_STATIC_PROVEN"),
    },
}, indent=2))
PY

# The distinct-authority proof (compile+runtime miss) is the hard, env-independent
# requirement and must hold everywhere python3 is present.
if [[ "$distinct_authority_proven" != true ]]; then
  echo "error: PY-L1 static distinct-authority proof failed (compile/runtime did not both miss the defect)" >&2
  exit 1
fi

if [[ "$static_proven" == true ]]; then
  echo "leaf-model python L1 static-structural verification PROVEN (pyright ${pyright_version}: happy clean, falsification ${pyright_expected_rule})"
elif [[ "$static_available" == true ]]; then
  echo "error: pyright ran but did not match expected verdicts (happy_clean=${happy_static_clean}, falsification_rule=${falsification_static_rule})" >&2
  exit 1
else
  echo "leaf-model python L1 static-structural verification DEFERRED: pyright unavailable; compile+runtime distinct-authority proof PASSED" >&2
fi
