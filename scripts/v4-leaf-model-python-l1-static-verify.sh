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
# Tool policy: the runner CONSUMES the modeled pyright_profile_l1 (single authority,
# src/v4/extdeps/typecheckers/pyright.dag) — it pins exactly that pyright_version (PATH
# pyright is used only if its --version matches, else npx pins it) and feeds the modeled
# python_version + type_checking_mode to pyright via a generated pyrightconfig.json. So the
# F1 receipt proves the SAME profile that pyright_profile_l1_id references, not whatever
# pyright happens to be on PATH. The fixtures are modeled BlockingForRung
# (src/v4/std/leaf_model_verification.dag), so this gate is FAIL-CLOSED: the static
# authority must positively prove it caught the falsification. If pyright cannot be
# obtained (offline runner) the run still records an honest receipt that distinguishes
# tool-unavailable from a behavioral failure (F2), but it EXITS NON-ZERO — a blocking
# static check that was not proven is a MISS, never a deferred pass.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
pyright_profile_dag="src/v4/extdeps/typecheckers/pyright.dag"
# pyright_expected_rule is NOT hardcoded — it is resolved (below) from the modeled
# pyright_diagnostic_rules row for the fixture's expected diagnostic_code (single authority).

if [[ ! -f "$fixture_dag" ]]; then
  echo "error: missing fixture authority at $fixture_dag" >&2
  exit 1
fi
if [[ ! -f "$pyright_profile_dag" ]]; then
  echo "error: missing pyright profile authority at $pyright_profile_dag" >&2
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

# The fixture's modeled expected diagnostic_code (the falsification case's blocking
# rejection). Read it from the lens — the script must not re-author this fact.
m = re.search(r"diagnostic_code:\s*(pyright_\w+)", text)
if not m:
    raise SystemExit(f"error: {path}: missing falsification diagnostic_code symbol")
print(f"expected_diag_code={shlex.quote(m.group(1))}")
PY
)"

# Single authority: the static profile facts come from the modeled pyright_profile_l1
# (pyright.dag), NOT hardcoded. The runner pins exactly that version and feeds the modeled
# pythonVersion / typeCheckingMode to pyright via a generated pyrightconfig.json, so the F1
# receipt proves the SAME profile that pyright_profile_l1_id references.
eval "$(python3 - "$pyright_profile_dag" "$expected_diag_code" <<'PY'
from __future__ import annotations

import re
import shlex
import sys
from pathlib import Path

text = Path(sys.argv[1]).read_text()
expected_code = sys.argv[2]

def field(name: str) -> str:
    m = re.search(rf'{name}:\s*"([^"]*)"', text)
    if not m:
        raise SystemExit(f"error: pyright.dag: missing pyright_profile_l1 field {name}")
    return m.group(1)

# The typeCheckingMode keyword is NOT re-authored here — it is resolved from the modeled
# pyright_type_checking_mode_spellings projection (single authority), keyed by the profile's
# variant.
m = re.search(r"type_checking_mode:\s*(PyrightMode\w+)", text)
if not m:
    raise SystemExit("error: pyright.dag: missing pyright_profile_l1 type_checking_mode")
variant = m.group(1)
spell_m = re.search(
    rf'variant_name:\s*"{re.escape(variant)}"\s*,\s*keyword:\s*"([^"]*)"', text
)
if not spell_m:
    raise SystemExit(f"error: pyright.dag: no pyright_type_checking_mode_spellings row for {variant}")

# Resolve the expected rule STRING from the modeled pyright_diagnostic_rules row whose
# code_id matches the fixture's diagnostic_code — single authority, never a script literal.
rule_m = re.search(
    rf'code_id:\s*{re.escape(expected_code)}\s*,\s*rule_name:\s*"([^"]*)"', text
)
if not rule_m:
    raise SystemExit(f"error: pyright.dag: no pyright_diagnostic_rules row for code_id {expected_code}")

print(f"pyright_version={shlex.quote(field('pyright_version'))}")
print(f"pyright_python_version={shlex.quote(field('python_version'))}")
print(f"pyright_mode={shlex.quote(spell_m.group(1))}")
print(f"pyright_expected_rule={shlex.quote(rule_m.group(1))}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-python-l1-static-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

printf '%s' "$happy_source" >"${scratch}/happy.py"
printf '%s' "$falsification_source" >"${scratch}/falsification.py"

# Modeled profile → pyright config (auto-discovered by pyright from the target's directory).
cat >"${scratch}/pyrightconfig.json" <<JSON
{ "pythonVersion": "${pyright_python_version}", "typeCheckingMode": "${pyright_mode}" }
JSON

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
  local cmd
  # Pin EXACTLY the modeled pyright_version. A pyright already on PATH is used only if its
  # version matches the modeled profile; otherwise pin via npx. This prevents the receipt
  # from passing under a different tool profile than pyright_profile_l1 declares.
  if command -v pyright >/dev/null 2>&1 \
     && pyright --version 2>/dev/null | grep -qw "$pyright_version"; then
    cmd=(pyright --outputjson "$target")
  elif command -v npx >/dev/null 2>&1; then
    cmd=(npx --yes "pyright@${pyright_version}" --outputjson "$target")
  else
    return 2  # modeled tool/version unavailable
  fi
  # pyright EXITS NON-ZERO when it reports diagnostics — for the falsification fixture
  # that is the EXPECTED success path (reportReturnType), NOT a tool-availability failure.
  # So ignore pyright's exit code and judge availability purely by whether it produced
  # parseable --outputjson. An empty/invalid file (e.g. npx could not fetch pyright,
  # offline runner) => tool unavailable (return 2), which the caller treats as a MISS.
  "${cmd[@]}" >"$out" 2>/dev/null || true
  python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$out" >/dev/null 2>&1 || return 2
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
export V4_PYL1_PYTHON_VERSION="$pyright_python_version"
export V4_PYL1_MODE="$pyright_mode"
export V4_PYL1_EXPECTED_RULE="$pyright_expected_rule"

python3 - <<'PY'
import json, os

def b(name): return os.environ[name] == "true"

static_available = b("V4_PYL1_STATIC_AVAILABLE")
print(json.dumps({
    "schema": "scripts/v4-leaf-model-python-l1-static-verify.sh::static_receipt_v1",
    "claim_id": "PythonL1StaticReturnType",
    "tool": {
        "id": "pyright",
        "version": os.environ["V4_PYL1_PYRIGHT_VERSION"],
        "python_version": os.environ["V4_PYL1_PYTHON_VERSION"],
        "type_checking_mode": os.environ["V4_PYL1_MODE"],
        "profile": "pyright_profile_l1 (single authority: src/v4/extdeps/typecheckers/pyright.dag)",
    },
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

# FAIL-CLOSED for the modeled BlockingForRung role (src/v4/std/leaf_model_verification.dag
# TargetStaticAnalysisRole / python_l1_static_fixture_pair, both cases BlockingForRung):
# the static authority must POSITIVELY prove it caught the falsification. Anything short of
# that — pyright unavailable (deferred) OR pyright ran but did not match — is a MISS and
# exits non-zero. A deferred/advisory pass would let an unproven blocking check succeed,
# which contradicts the model. (To run this fixture as non-blocking, the modeled role must
# be Advisory, not BlockingForRung — the role drives the gate, not a script-local escape.)
if [[ "$static_proven" == true ]]; then
  echo "leaf-model python L1 static-structural verification PROVEN (pyright ${pyright_version}: happy clean, falsification ${pyright_expected_rule})"
elif [[ "$static_available" == true ]]; then
  echo "error: pyright ran but did not match expected verdicts (happy_clean=${happy_static_clean}, falsification_rule=${falsification_static_rule})" >&2
  exit 1
else
  echo "error: pyright unavailable — BlockingForRung static authority UNPROVEN (fail-closed). The compile+runtime distinct-authority proof passed, but a blocking static check cannot be deferred to a pass." >&2
  exit 1
fi
