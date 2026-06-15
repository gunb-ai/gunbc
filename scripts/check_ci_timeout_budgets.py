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
        "fmt": 15,
        "doc_refs": 5,
        # Was python-only (instant); now builds ci_claim_gate + v2-compiler +
        # layering_imports_scan before scanner-execution gate. 15m matches fmt job
        # shape; warm sccache from parallel floor jobs keeps uncontended wall low.
        "layering_imports": 15,
        "ci_floor": 60,
        "ci_floor_parity": 60,
        "ci_floor_emit": 60,
        # INTERIM 35m (2026-06-13): uncontended base structurally outgrew 20m
        # (witness-row + perturb-closure growth: ~2min/resolve over 73-source
        # closures mid-perturb; #4792/#4785/#4786 all green-then-killed at
        # 20m22s; reruns cannot fix a structural overrun). Budget rule:
        # documented uncontended wall (~16m) x2 = 32, rounded to 35. The x2
        # multiplier covers typical merge-wave DRAM-bandwidth contention
        # (measured 1.5-2.6x, 2.5x observed twice); the 2.0-2.6x tail is
        # accepted as rare manual-rerun territory. Dissolve-on: perturb-phase
        # split into a parallel job and/or #4783 multi-entry claim_batch
        # parse-cache adoption land -- both shrink the uncontended wall back
        # step under load -> 45m ceiling (13.7m x3.5 rounded).
        "v4_lens_gate": 45,
        "v4_lens_ci": 35,
        # Per-row PERTURB fan-out split out of v4_lens_ci into a capped parallel
        # matrix (4 legs, max-parallel default 4 — the "perturb-phase split into a
        # parallel job" dissolve-on named in the v4_lens_ci comment above). Budget
        # rule (uncontended wall x2): the heaviest shard owns 4 of 15 rows; ~108s
        # cold resolve/row + per-row src/v4 copy + checkout/setup/cache-restore
        # ~= 8m uncontended; x2 -> 16, rounded to 20 (matches the corpus shards).
        # Re-derive from the first on-wave per-shard receipt; ratchet against raises.
        "v4_lens_ci_perturb": 20,
        # Non-gating latency timing ledger: checkout + rust + the shell-free .dag github.Actions
        # fetch (collect-affected-set-timings, which embeds v2-compiler and writes the timed receipt
        # directly). Budget rule (documented uncontended wall x2): v2-compiler is sccache-warm from
        # the floor jobs; warm wall ~1-2m, cold-build tail ~3-4m; 4m x2 = 8. Re-derive from first green.
        "affected_timings": 8,
        "v4_claim_witness_corpus_a": 20,  # ~8.4m uncontended/shard (7.4m Phase A + ~1m spot); ×2 → 20m
        "v4_claim_witness_corpus_b": 20,
        "timeout_budgets": 5,
        "ci_lens_lane_class": 5,
        "ci_fleet_shard": 2,
        "ci": 5,
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
