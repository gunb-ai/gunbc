#!/usr/bin/env bash
# scripts/v4-leaf-model-python-l1-mypy-static-verify.sh
#
# PY-L1-STATIC-STRUCTURAL - mypy companion receipt for the same return-type fixture
# exercised by the pyright lane (Worksheet B in
# docs/planning/v4-python-rca-manager-worksheets-2026-06-01.md). Static evidence only,
# distinct from CPython compile/runtime and distinct from L2 cross-target behavioral parity.
# Fixture authority: src/v4/lens/leaf_model_verification.dag (python_l1_static_mypy_fixture).

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
mypy_profile_dag="src/v4/extdeps/typecheckers/mypy.dag"

if [[ ! -f "$fixture_dag" || ! -f "$mypy_profile_dag" ]]; then
  echo "error: missing fixture/profile authority" >&2
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

print(f"happy_source={shlex.quote(extract('python_l1_static_happy_fixture_source'))}")
print(f"falsification_source={shlex.quote(extract('python_l1_static_falsification_fixture_source'))}")

# The mypy fixture's modeled expected diagnostic_code (falsification case). Read from the
# python_l1_static_mypy_fixture_pair block — the script must not re-author this fact.
m = re.search(
    r"python_l1_static_mypy_fixture_pair[\s\S]*?diagnostic_code:\s*(mypy_\w+)",
    text,
)
if not m:
    raise SystemExit(f"error: {path}: missing mypy falsification diagnostic_code symbol")
print(f"expected_diag_code={shlex.quote(m.group(1))}")
PY
)"

# Single authority: profile facts from mypy_profile_l1 (mypy.dag), including strict and
# show_error_codes; expected bracket code resolved via mypy_diagnostic_codes keyed by the
# fixture's diagnostic_code symbol (same pattern as the pyright lane).
eval "$(python3 - "$mypy_profile_dag" "$expected_diag_code" <<'PY'
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
        raise SystemExit(f"error: mypy.dag: missing mypy_profile_l1 field {name}")
    return m.group(1)

def bool_field(name: str) -> str:
    m = re.search(rf'{name}:\s*(true|false)', text)
    if not m:
        raise SystemExit(f"error: mypy.dag: missing mypy_profile_l1 field {name}")
    return m.group(1)

code_m = re.search(
    rf'code_id:\s*{re.escape(expected_code)}\s*,\s*code_name:\s*"([^"]*)"',
    text,
)
if not code_m:
    raise SystemExit(
        f"error: mypy.dag: no mypy_diagnostic_codes row for code_id {expected_code}"
    )

print(f"mypy_version={shlex.quote(field('mypy_version'))}")
print(f"mypy_python_version={shlex.quote(field('python_version'))}")
print(f"mypy_strict={shlex.quote(bool_field('strict'))}")
print(f"mypy_show_error_codes={shlex.quote(bool_field('show_error_codes'))}")
print(f"mypy_expected_code={shlex.quote(code_m.group(1))}")
PY
)"

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
scratch="${RUNNER_TEMP:-/tmp}/v4-leaf-model-python-l1-mypy-static-${run_suffix}"
rm -rf "$scratch"
mkdir -p "$scratch"

printf '%s' "$happy_source" >"${scratch}/happy.py"
printf '%s' "$falsification_source" >"${scratch}/falsification.py"

compile_authority_misses=true
runtime_authority_misses=true
for label in happy falsification; do
  if ! python3 -m py_compile "${scratch}/${label}.py" 2>"${scratch}/${label}.compile.err"; then
    compile_authority_misses=false
  fi
done
if ! python3 "${scratch}/falsification.py" >/dev/null 2>"${scratch}/falsification.run.err"; then
  runtime_authority_misses=false
fi

mypy_available=false
happy_static_clean=false
falsification_static_rejected=false

mypy_profile_args=()
[[ "$mypy_strict" == true ]] && mypy_profile_args+=(--strict)
[[ "$mypy_show_error_codes" == true ]] && mypy_profile_args+=(--show-error-codes)

mypy_run() {
  local target="$1" out="$2"
  local cmd
  if command -v mypy >/dev/null 2>&1 && mypy --version 2>/dev/null | grep -qw "$mypy_version"; then
    cmd=(mypy --python-version "$mypy_python_version" "${mypy_profile_args[@]}" "$target")
  elif command -v python3 >/dev/null 2>&1; then
    if [[ ! -x "${scratch}/mypy-venv/bin/mypy" ]]; then
      if python3 -m venv "${scratch}/mypy-venv" >/dev/null 2>&1; then
        "${scratch}/mypy-venv/bin/python" -m pip install --disable-pip-version-check -q "mypy==${mypy_version}" >/dev/null 2>&1 || return 2
      else
        mkdir -p "${scratch}/pip-bootstrap" "${scratch}/mypy-site" "${scratch}/mypy-venv/bin"
        python3 - "${scratch}/pip-bootstrap/get-pip.py" <<'PY' || return 2
import sys
import urllib.request

urllib.request.urlretrieve("https://bootstrap.pypa.io/get-pip.py", sys.argv[1])
PY
        python3 "${scratch}/pip-bootstrap/get-pip.py" --target "${scratch}/pip-bootstrap/site" --no-warn-script-location >/dev/null 2>&1 || return 2
        PYTHONPATH="${scratch}/pip-bootstrap/site" python3 -m pip install --disable-pip-version-check -q --target "${scratch}/mypy-site" "mypy==${mypy_version}" >/dev/null 2>&1 || return 2
        cat >"${scratch}/mypy-venv/bin/mypy" <<SH
#!/usr/bin/env bash
PYTHONPATH="${scratch}/mypy-site" python3 -m mypy "\$@"
SH
        chmod +x "${scratch}/mypy-venv/bin/mypy"
      fi
    fi
    cmd=("${scratch}/mypy-venv/bin/mypy" --python-version "$mypy_python_version" "${mypy_profile_args[@]}" "$target")
  else
    return 2
  fi
  "${cmd[@]}" >"$out" 2>&1 || true
  grep -qE 'Success: no issues found|error: ' "$out" || return 2
  return 0
}

if mypy_run "${scratch}/happy.py" "${scratch}/happy.mypy.txt" \
  && mypy_run "${scratch}/falsification.py" "${scratch}/falsification.mypy.txt"; then
  mypy_available=true
fi

if [[ "$mypy_available" == true ]]; then
  grep -q 'Success: no issues found' "${scratch}/happy.mypy.txt" && happy_static_clean=true
  grep -q "\\[${mypy_expected_code}\\]" "${scratch}/falsification.mypy.txt" && falsification_static_rejected=true
fi

distinct_authority_proven=false
[[ "$compile_authority_misses" == true && "$runtime_authority_misses" == true ]] && distinct_authority_proven=true
static_proven=false
[[ "$mypy_available" == true && "$happy_static_clean" == true && "$falsification_static_rejected" == true ]] && static_proven=true

export V4_MYPY_VERSION="$mypy_version"
export V4_MYPY_PYTHON_VERSION="$mypy_python_version"
export V4_MYPY_STRICT="$mypy_strict"
export V4_MYPY_SHOW_ERROR_CODES="$mypy_show_error_codes"
export V4_MYPY_EXPECTED_CODE="$mypy_expected_code"
export V4_MYPY_COMPILE_MISSES="$compile_authority_misses"
export V4_MYPY_RUNTIME_MISSES="$runtime_authority_misses"
export V4_MYPY_AVAILABLE="$mypy_available"
export V4_MYPY_HAPPY_CLEAN="$happy_static_clean"
export V4_MYPY_FALS_REJECTED="$falsification_static_rejected"
export V4_MYPY_DISTINCT_PROVEN="$distinct_authority_proven"
export V4_MYPY_STATIC_PROVEN="$static_proven"

python3 - <<'PY'
import json, os

def b(name): return os.environ[name] == "true"
print(json.dumps({
    "schema": "scripts/v4-leaf-model-python-l1-mypy-static-verify.sh::static_receipt_v1",
    "claim_id": "PythonL1StaticReturnType",
    "tool": {
        "id": "mypy",
        "version": os.environ["V4_MYPY_VERSION"],
        "python_version": os.environ["V4_MYPY_PYTHON_VERSION"],
        "strict": b("V4_MYPY_STRICT"),
        "show_error_codes": b("V4_MYPY_SHOW_ERROR_CODES"),
        "profile": "mypy_profile_l1 (single authority: src/v4/extdeps/typecheckers/mypy.dag)",
    },
    "authority_separation": {
        "py_compile_accepts_both": b("V4_MYPY_COMPILE_MISSES"),
        "runtime_exec_clean_on_falsification": b("V4_MYPY_RUNTIME_MISSES"),
        "distinct_authority_proven": b("V4_MYPY_DISTINCT_PROVEN"),
    },
    "static_analysis": {
        "available": b("V4_MYPY_AVAILABLE"),
        "role": "BlockingForRung",
        "happy_clean": b("V4_MYPY_HAPPY_CLEAN"),
        "falsification_rejected": b("V4_MYPY_FALS_REJECTED"),
        "expected_diagnostic_code": os.environ["V4_MYPY_EXPECTED_CODE"],
        "proven": b("V4_MYPY_STATIC_PROVEN"),
    },
}, indent=2))
PY

if [[ "$distinct_authority_proven" != true ]]; then
  echo "error: mypy L1 distinct-authority proof failed" >&2
  exit 1
fi
if [[ "$static_proven" != true ]]; then
  echo "error: mypy unavailable or did not match expected return-value verdict" >&2
  exit 1
fi

echo "leaf-model python L1 mypy static-structural verification PROVEN"
