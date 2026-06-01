#!/usr/bin/env bash
# scripts/v4-leaf-model-python-l1-static-verify.sh
#
# PY-L1-STATIC-STRUCTURAL — pyright static structural analysis as a THIRD, distinct
# verification authority alongside CPython `py_compile` and `python3` runtime exec.
# Authority: Modeling DFS Arbiter ruling proud-fox-405 (msg_41813c03);
# worksheet docs/planning/v4-python-rca-manager-worksheets-2026-06-01.md Worksheet A.
# Fixture authority: src/v4/lens/leaf_model_verification.dag (python_l1_static_* roster).
#
# FIXTURE-SCALE: this runner exercises the WHOLE modeled roster
# (python_l1_static_fixture_roster), not a single pair. Each roster fixture holds the same
# hard third-authority property:
#   - py_compile ACCEPTS happy + falsification (syntactically valid)  → compile authority misses.
#   - python3 runs the falsification with exit 0 (the defect lives in
#     a function that is never called)                                → runtime authority misses.
#   - pyright reports the modeled diagnostic rule for the fixture      → static authority CATCHES it.
# Covered diagnostic classes (Jun-1 widening): reportReturnType, reportUndefinedVariable,
# reportMissingImports. This is exactly why static analysis is modeled as
# TargetStaticAnalysisVerdict, a carrier distinct from TargetPythonCompileRejected and
# TargetPythonExecRejected.
#
# Tool policy: the runner CONSUMES the modeled pyright_profile_l1 (single authority,
# src/v4/extdeps/typecheckers/pyright.dag) — it pins exactly that pyright_version (PATH
# pyright is used only if its --version matches, else npx pins it) and feeds the modeled
# python_version + type_checking_mode to pyright via a generated pyrightconfig.json. So each
# receipt proves the SAME profile that pyright_profile_l1_id references, not whatever pyright
# happens to be on PATH. The roster membership AND every per-fixture fact (sources +
# diagnostic_code + expected rule) are read from the model, never re-authored here. The
# fixtures are modeled BlockingForRung (src/v4/std/leaf_model_verification.dag), so this gate
# is FAIL-CLOSED: the static authority must positively prove it caught EVERY falsification.
# If pyright cannot be obtained (offline runner) the run still records an honest receipt that
# distinguishes tool-unavailable from a behavioral failure, but it EXITS NON-ZERO — a
# blocking static check that was not proven is a MISS, never a deferred pass.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_dag="src/v4/lens/leaf_model_verification.dag"
pyright_profile_dag="src/v4/extdeps/typecheckers/pyright.dag"

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

# The entire fixture-scale gate runs in one Python driver: it parses the modeled roster +
# pyright profile (single authority), then exercises the three authorities per fixture and
# emits a combined receipt. The pyright invocation policy (PATH version-match else npx pin)
# matches the modeled pyright_version exactly. Exit code is fail-closed over the whole roster.
scratch_root="${RUNNER_TEMP:-/tmp}/v4-leaf-model-python-l1-static-${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
rm -rf "$scratch_root"
mkdir -p "$scratch_root"

V4_PYL1_FIXTURE_DAG="$fixture_dag" \
V4_PYL1_PROFILE_DAG="$pyright_profile_dag" \
V4_PYL1_SCRATCH_ROOT="$scratch_root" \
python3 - <<'PY'
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

fixture_dag = Path(os.environ["V4_PYL1_FIXTURE_DAG"])
profile_dag = Path(os.environ["V4_PYL1_PROFILE_DAG"])
scratch_root = Path(os.environ["V4_PYL1_SCRATCH_ROOT"])

lens = fixture_dag.read_text()
profile = profile_dag.read_text()


def fail(msg: str) -> None:
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


# --- modeled pyright profile (single authority: pyright.dag) ---
def profile_field(name: str) -> str:
    m = re.search(rf'{name}:\s*"([^"]*)"', profile)
    if not m:
        fail(f"pyright.dag: missing pyright_profile_l1 field {name}")
    return m.group(1)


pyright_version = profile_field("pyright_version")
python_version = profile_field("python_version")

mode_m = re.search(r"type_checking_mode:\s*(PyrightMode\w+)", profile)
if not mode_m:
    fail("pyright.dag: missing pyright_profile_l1 type_checking_mode")
mode_variant = mode_m.group(1)
# typeCheckingMode keyword resolved from the modeled spelling row keyed by the enum variant
# (single authority — never re-spelled here).
spell_m = re.search(
    rf'mode:\s*{re.escape(mode_variant)}\s*,\s*keyword:\s*"([^"]*)"', profile
)
if not spell_m:
    fail(f"pyright.dag: no pyright_type_checking_mode_spellings row for {mode_variant}")
pyright_mode = spell_m.group(1)


def data_string(name: str) -> str:
    m = re.search(rf'^data {re.escape(name)}: String = "(.*)"\s*$', lens, re.MULTILINE)
    if not m:
        fail(f"{fixture_dag}: missing data {name}: String = ...")
    return bytes(m.group(1), "utf-8").decode("unicode_escape")


def rule_for_code(code_id: str) -> str:
    m = re.search(
        rf'code_id:\s*{re.escape(code_id)}\s*,\s*rule_name:\s*"([^"]*)"', profile
    )
    if not m:
        fail(f"pyright.dag: no pyright_diagnostic_rules row for code_id {code_id}")
    return m.group(1)


# --- roster membership (single authority: python_l1_static_fixture_roster) ---
roster_m = re.search(
    r"fn python_l1_static_fixture_roster\(\)\s*->\s*List<[^>]+>\s*\{(.*?)\n\}",
    lens,
    re.DOTALL,
)
if not roster_m:
    fail(f"{fixture_dag}: missing fn python_l1_static_fixture_roster")
pair_fn_names = re.findall(r"([a-z0-9_]+)\(\)", roster_m.group(1))
if not pair_fn_names:
    fail(f"{fixture_dag}: python_l1_static_fixture_roster lists no pair fns")


def parse_pair_fn(fn_name: str) -> dict:
    body_m = re.search(
        rf"fn {re.escape(fn_name)}\(\)\s*->\s*LeafModelPythonStaticFixturePair\s*\{{(.*?)\n\}}",
        lens,
        re.DOTALL,
    )
    if not body_m:
        fail(f"{fixture_dag}: missing fixture-pair fn {fn_name}")
    body = body_m.group(1)
    srcs = re.findall(r"source_text:\s*([a-z0-9_]+)", body)
    if len(srcs) != 2:
        fail(f"{fixture_dag}: {fn_name} must reference exactly 2 source_text data (happy, falsification)")
    code_m = re.search(r"diagnostic_code:\s*([a-z0-9_]+)", body)
    if not code_m:
        fail(f"{fixture_dag}: {fn_name} missing falsification diagnostic_code")
    code_id = code_m.group(1)
    return {
        "fixture": fn_name,
        "happy_source": data_string(srcs[0]),
        "falsification_source": data_string(srcs[1]),
        "diagnostic_code": code_id,
        "expected_rule": rule_for_code(code_id),
    }


fixtures = [parse_pair_fn(n) for n in pair_fn_names]


# --- pyright invocation (pins the modeled version; judges availability by parseable json) ---
def pyright_cmd(scratch: Path, target: Path) -> list[str] | None:
    try:
        path_ver = subprocess.run(
            ["pyright", "--version"], capture_output=True, text=True
        ).stdout
        if re.search(rf"\b{re.escape(pyright_version)}\b", path_ver):
            return ["pyright", "--project", str(scratch), "--outputjson", str(target)]
    except FileNotFoundError:
        pass
    if subprocess.run(["sh", "-c", "command -v npx"], capture_output=True).returncode == 0:
        return ["npx", "--yes", f"pyright@{pyright_version}", "--project", str(scratch), "--outputjson", str(target)]
    return None


def pyright_error_rules(scratch: Path, target: Path) -> list[str] | None:
    """Returns the list of error-severity rule names, or None if the tool is unavailable."""
    cmd = pyright_cmd(scratch, target)
    if cmd is None:
        return None
    proc = subprocess.run(cmd, capture_output=True, text=True)
    # pyright EXITS NON-ZERO when it reports diagnostics — that is the expected success path
    # for a falsification, NOT a tool-availability failure. Judge availability purely by
    # whether parseable --outputjson was produced (empty/invalid => offline npx => unavailable).
    try:
        doc = json.loads(proc.stdout)
    except (json.JSONDecodeError, ValueError):
        return None
    return [d.get("rule") for d in doc.get("generalDiagnostics", []) if d.get("severity") == "error"]


results = []
all_proven = True
for fx in fixtures:
    scratch = scratch_root / fx["fixture"]
    scratch.mkdir(parents=True, exist_ok=True)
    happy_py = scratch / "happy.py"
    fals_py = scratch / "falsification.py"
    happy_py.write_text(fx["happy_source"])
    fals_py.write_text(fx["falsification_source"])
    (scratch / "pyrightconfig.json").write_text(
        json.dumps({"pythonVersion": python_version, "typeCheckingMode": pyright_mode})
    )

    # compile authority: both files must be accepted by py_compile.
    compile_misses = True
    for p in (happy_py, fals_py):
        if subprocess.run([sys.executable, "-m", "py_compile", str(p)], capture_output=True).returncode != 0:
            compile_misses = False
    # runtime authority: the falsification must RUN clean (defect never reached at runtime).
    runtime_misses = subprocess.run([sys.executable, str(fals_py)], capture_output=True).returncode == 0

    distinct_authority_proven = compile_misses and runtime_misses

    happy_rules = pyright_error_rules(scratch, happy_py)
    fals_rules = pyright_error_rules(scratch, fals_py)
    static_available = happy_rules is not None and fals_rules is not None
    happy_clean = static_available and len(happy_rules) == 0
    fals_rejected = static_available and fx["expected_rule"] in (fals_rules or [])
    static_proven = static_available and happy_clean and fals_rejected

    proven = distinct_authority_proven and static_proven
    all_proven = all_proven and proven

    results.append({
        "fixture": fx["fixture"],
        "diagnostic_code": fx["diagnostic_code"],
        "expected_diagnostic_rule": fx["expected_rule"],
        "authority_separation": {
            "py_compile_accepts_both": compile_misses,
            "runtime_exec_clean_on_falsification": runtime_misses,
            "distinct_authority_proven": distinct_authority_proven,
        },
        "static_analysis": {
            "available": static_available,
            "role": "BlockingForRung",
            "happy_verdict": (
                "StaticAnalysisAccepted" if happy_clean
                else ("Deferred(tool-unavailable)" if not static_available else "StaticAnalysisRejected")
            ),
            "falsification_verdict": (
                f"StaticAnalysisRejected:{fx['expected_rule']}" if fals_rejected
                else ("Deferred(tool-unavailable)" if not static_available else "StaticAnalysisAccepted")
            ),
            "observed_falsification_rules": fals_rules,
            "proven": static_proven,
        },
        "proven": proven,
    })

receipt = {
    "schema": "scripts/v4-leaf-model-python-l1-static-verify.sh::static_receipt_v2",
    "claim_id": "PythonL1StaticRoster",
    "tool": {
        "id": "pyright",
        "version": pyright_version,
        "python_version": python_version,
        "type_checking_mode": pyright_mode,
        "profile": "pyright_profile_l1 (single authority: src/v4/extdeps/typecheckers/pyright.dag)",
    },
    "roster_size": len(fixtures),
    "roster_all_proven": all_proven,
    "fixtures": results,
}
print(json.dumps(receipt, indent=2))

# Fail-closed over the whole roster. The distinct-authority proof (compile+runtime miss) is
# env-independent and must hold for every fixture; the BlockingForRung static authority must
# POSITIVELY catch every falsification (anything short — pyright unavailable, or ran but did
# not match — is a MISS). A deferred/advisory pass would let an unproven blocking check
# succeed, contradicting the model.
distinct_failures = [r["fixture"] for r in results if not r["authority_separation"]["distinct_authority_proven"]]
if distinct_failures:
    print(
        f"error: PY-L1 static distinct-authority proof failed for {distinct_failures} "
        "(compile/runtime did not both miss the defect)",
        file=sys.stderr,
    )
    sys.exit(1)

unproven = [r["fixture"] for r in results if not r["static_analysis"]["proven"]]
if unproven:
    any_unavailable = any(not r["static_analysis"]["available"] for r in results)
    if any_unavailable:
        print(
            f"error: pyright unavailable for {unproven} — BlockingForRung static authority "
            "UNPROVEN (fail-closed). The compile+runtime distinct-authority proof passed, but a "
            "blocking static check cannot be deferred to a pass.",
            file=sys.stderr,
        )
    else:
        print(
            f"error: pyright ran but did not match expected verdicts for {unproven}",
            file=sys.stderr,
        )
    sys.exit(1)

print(
    f"leaf-model python L1 static-structural FIXTURE-SCALE verification PROVEN "
    f"(pyright {pyright_version}: {len(fixtures)} fixtures, all happy clean + falsification caught)"
)
PY
