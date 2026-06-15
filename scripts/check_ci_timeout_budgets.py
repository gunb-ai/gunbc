#!/usr/bin/env python3
"""CI wall-time budget gate -- declared `timeout-minutes` ceilings are gate errors.

History this gate exists to stop: a PR lands a slow step and bumps the job's
`timeout-minutes` in the same diff to absorb it (#4324 bumped the M1 emit probe
20->35; #4633 bumped v4_lens_gate 20->35 to absorb a per-entry resolve fan-out
that #4719 then root-caused back under 20). A timeout raise is a latency
regression wearing a config diff. This gate makes every `timeout-minutes` in
.github/workflows/ subject to a ceiling declared HERE, so raising one requires
a separate, loud edit to this table that review can interrogate ("what did you
root-cause?") instead of a one-line change buried in a feature diff.

Per the S3 §3.1 amendment idiom: the declared budget is the gate; exceeding it
is an ERROR, never a sanctioned placeholder. Lower is always allowed -- this is
a ratchet against raises, not a pin.

Pure stdlib; no yaml dependency. The parser is indentation-based and
fail-closed: every `timeout-minutes:` line in every workflow file must be
attributable to a job and covered by a ceiling, or the gate errors.

Usage:
  python3 scripts/check_ci_timeout_budgets.py                  # gate
  python3 scripts/check_ci_timeout_budgets.py --perturb-check  # self-test
"""

import re
import sys
import tempfile
from pathlib import Path

# Ceiling (minutes) per workflow job. Covers BOTH job-level and step-level
# `timeout-minutes` under the job. Raising a ceiling requires editing this
# table -- state the root-cause analysis in the PR, not just the new number.
CEILINGS = {
    "ci.yml": {
        "infra_isolation": 5,
        # One-binary bankruptcy floor (operator 2026-06-15): serializes the former
        # parallel Wave-1 jobs (ci_floor, parity, emit, lens, corpus, doc_refs, …)
        # into a single `run_ci_pipeline` invocation. Budget rule (uncontended wall ×2):
        # cold pre-warm + 3 gates + witness teeth ~90–120m observed; ×2 → 240 for merge-wave
        # contention. Re-derive from first green one-binary receipt; ratchet against raises.
        "ci": 240,
    },
    "ci-spot-rerun.yml": {
        "rerun-once": 5,
    },
    "release.yml": {
        "build": 60,
        "release": 15,
    },
}

JOB_RE = re.compile(r"^  ([A-Za-z_][A-Za-z0-9_-]*):\s*(#.*)?$")
TIMEOUT_RE = re.compile(r"^\s+timeout-minutes:\s*(\S+)")


def collect_timeouts(path: Path):
    """Yield (job_id, line_no, minutes) for every timeout-minutes line."""
    in_jobs = False
    job = None
    for line_no, line in enumerate(path.read_text().splitlines(), start=1):
        if re.match(r"^jobs:\s*$", line):
            in_jobs = True
            continue
        if in_jobs and re.match(r"^[A-Za-z_]", line):
            in_jobs = False  # left the jobs: block (new top-level key)
        if in_jobs:
            m = JOB_RE.match(line)
            if m:
                job = m.group(1)
        m = TIMEOUT_RE.match(line)
        if m:
            raw = m.group(1)
            if not raw.isdigit():
                yield (job, line_no, None, raw)  # unparseable (e.g. expression)
            else:
                yield (job, line_no, int(raw), raw)


def run_gate(workflows_dir: Path) -> list:
    errors = []
    seen_jobs = set()
    files = sorted(workflows_dir.glob("*.yml")) + sorted(workflows_dir.glob("*.yaml"))
    if not files:
        return [f"no workflow files found under {workflows_dir}"]
    for path in files:
        name = path.name
        ceilings = CEILINGS.get(name)
        for job, line_no, minutes, raw in collect_timeouts(path):
            if job is None:
                errors.append(
                    f"{name}:{line_no}: timeout-minutes outside any job -- parser "
                    f"cannot attribute it; restructure or extend the gate"
                )
                continue
            seen_jobs.add((name, job))
            if minutes is None:
                errors.append(
                    f"{name}:{line_no}: non-literal timeout-minutes '{raw}' in job "
                    f"'{job}' -- budgets must be literal integers"
                )
                continue
            if ceilings is None or job not in ceilings:
                errors.append(
                    f"{name}:{line_no}: job '{job}' has timeout-minutes {minutes} but "
                    f"no declared ceiling in scripts/check_ci_timeout_budgets.py -- "
                    f"declare one (a new job must declare its wall-time budget)"
                )
                continue
            if minutes > ceilings[job]:
                errors.append(
                    f"{name}:{line_no}: job '{job}' timeout-minutes {minutes} exceeds "
                    f"declared ceiling {ceilings[job]} -- root-cause the slow step "
                    f"instead of absorbing it; a justified raise edits BOTH this line "
                    f"and the ceiling table with the analysis in the PR"
                )
    for name, jobs in CEILINGS.items():
        for job in jobs:
            if (name, job) not in seen_jobs:
                errors.append(
                    f"stale ceiling: {name} job '{job}' is declared in "
                    f"scripts/check_ci_timeout_budgets.py but has no timeout-minutes "
                    f"in the workflow -- remove the stale entry"
                )
    return errors


def perturb_check(workflows_dir: Path) -> int:
    """Plant a raised timeout in a temp copy and require detection."""
    import shutil

    clean = run_gate(workflows_dir)
    if clean:
        print("perturb-check requires a clean tree first; gate errors present:")
        for e in clean:
            print(f"  {e}")
        return 1
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp) / "workflows"
        shutil.copytree(workflows_dir, tmp_dir)
        target = tmp_dir / "ci.yml"
        text = target.read_text()
        # Derive the plant target from the declared ceilings rather than
        # hardcoding a value: pick the first ci.yml job whose declared
        # timeout-minutes literal appears in the file, and raise it by 1
        # past its ceiling. A hardcoded value rots the moment budgets
        # change (the previous 'timeout-minutes: 20' plant broke when the
        # last 20m budgets were raised).
        planted, n = "", 0
        for job, ceiling in CEILINGS["ci.yml"].items():
            planted, n = re.subn(
                rf"timeout-minutes: {ceiling}\b",
                f"timeout-minutes: {ceiling + 1}",
                text,
                count=1,
            )
            if n == 1:
                break
        if n != 1:
            print(
                "perturb-check could not plant a raise (no declared ceiling "
                "value found verbatim in ci.yml)"
            )
            return 1
        target.write_text(planted)
        errors = run_gate(tmp_dir)
        if not any("exceeds declared ceiling" in e for e in errors):
            print("perturb-check FAILED: planted timeout raise was not detected")
            return 1
    print("perturb-check OK: planted timeout raise detected")
    return 0


def main(argv: list) -> int:
    workflows_dir = Path(".github/workflows")
    if "--perturb-check" in argv:
        return perturb_check(workflows_dir)
    errors = run_gate(workflows_dir)
    if errors:
        for e in errors:
            print(f"::error::timeout budget gate: {e}")
        return 1
    print("timeout budget gate OK: all timeout-minutes within declared ceilings")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
