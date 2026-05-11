#!/usr/bin/env python3
"""Aggregate N Criterion runs into ``tier3_baseline.json`` per ``docs/audit/c1-tier3-baseline-capture-procedure.md``.

For each budgeted bench name, per run: invoke ``src/v3/compiler/benches/tier3_extract_perf_stats.py``
on that run's ``sample.json``. Then ``median_ns`` = median of N per-run medians; ``p99_ns`` = max
of N per-run p99 values (R-7 conservative pin).

``--host-id`` is stamped into ``captured_on.host_id`` exactly as supplied—there is no runner/OS probe.
Callers MUST pass the truthful capture label (paired workflow step uses ``ubicloud-standard-2`` beside
the matching ``runs-on`` VM); spoofing canonical hosts is procedural fraud, not something this CLI
automates away.
"""

from __future__ import annotations

import argparse
import json
import re
import statistics
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
EXTRACT = REPO_ROOT / "src/v3/compiler/benches/tier3_extract_perf_stats.py"
# Written by each capture job next to ``criterion/`` (same runner as the bench binary).
RUSTC_VERSION_FILE = "rustc_version.txt"

TIER3_BENCH_ORDER: list[str] = [
    "tier3_termination_merge_evidence",
    "tier3_computation_positive_descent_count",
    "tier3_computation_lower_same_argument_call",
    "tier3_induction_type_iteration_dimension_miss",
    "tier3_effects_lane2_linear_read_chain",
]


def criterion_version_from_lock(repo: Path) -> str:
    """Return major.minor for the workspace `criterion` crate (default 0.5)."""
    cargo_lock = repo / "Cargo.lock"
    text = cargo_lock.read_text()
    m = re.search(
        r'\[\[package\]\]\nname = "criterion"\nversion = "([0-9]+)\.([0-9]+)',
        text,
    )
    if m:
        return f"{m.group(1)}.{m.group(2)}"
    return "0.5"


def read_consistent_rustc_version(run_bundles: list[Path]) -> str:
    """Return the single ``rustc --version`` line recorded by every capture bundle.

    Provenance must come from the benchmark-producing jobs (P2: no fresh ``rustc`` read
    in the aggregate environment, which may differ from the matrix runners).
    """
    versions: list[str] = []
    for bundle in run_bundles:
        path = bundle / RUSTC_VERSION_FILE
        if not path.is_file():
            raise SystemExit(
                f"missing {RUSTC_VERSION_FILE} next to criterion/ in {bundle} "
                "(capture job must record rustc --version alongside Criterion output)"
            )
        text = path.read_text().strip()
        if not text:
            raise SystemExit(f"empty {RUSTC_VERSION_FILE} in {bundle}")
        versions.append(text)
    distinct = sorted(set(versions))
    if len(distinct) != 1:
        raise SystemExit(
            f"{RUSTC_VERSION_FILE} mismatch across capture bundles: {distinct!r}"
        )
    return versions[0]


def extract_row(extract_tool: Path, sample: Path, bench: str) -> dict:
    out = subprocess.check_output(
        [sys.executable, str(extract_tool), str(sample), "--name", bench],
        text=True,
    )
    return json.loads(out)


def mirror_groups_from_runs(criterion_run_dirs: list[Path]) -> dict:
    benches_out: dict[str, dict] = {}
    for bench in TIER3_BENCH_ORDER:
        medians: list[int] = []
        p99s: list[int] = []
        for run_root in criterion_run_dirs:
            sample = run_root / bench / "new" / "sample.json"
            if not sample.is_file():
                raise SystemExit(f"missing sample.json for {bench}: {sample}")
            row = extract_row(EXTRACT, sample, bench)
            medians.append(int(row["median_ns"]))
            p99s.append(int(row["p99_ns"]))
        agg_median = int(round(statistics.median(medians)))
        agg_p99 = max(p99s)
        if agg_p99 < agg_median:
            raise SystemExit(
                f"aggregated p99 < median for {bench}: median={agg_median} p99={agg_p99}"
            )
        if agg_median <= 0 or agg_p99 <= 0:
            raise SystemExit(f"non-positive aggregate for {bench}: median={agg_median} p99={agg_p99}")
        benches_out[bench] = {
            "name": bench,
            "median_ns": agg_median,
            "p99_ns": agg_p99,
        }

    return {
        "termination": {
            "claim": "tier3_termination_mirror_perf_within_budget",
            "benches": [benches_out["tier3_termination_merge_evidence"]],
        },
        "computation": {
            "claim": "tier3_computation_mirror_perf_within_budget",
            "benches": [
                benches_out["tier3_computation_positive_descent_count"],
                benches_out["tier3_computation_lower_same_argument_call"],
            ],
        },
        "induction": {
            "claim": "tier3_induction_mirror_perf_within_budget",
            "benches": [benches_out["tier3_induction_type_iteration_dimension_miss"]],
        },
        "effect_carrier": {
            "claim": "tier3_effect_carrier_mirror_perf_within_budget",
            "benches": [benches_out["tier3_effects_lane2_linear_read_chain"]],
        },
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--host-id", required=True, help="R-3 canonical host string")
    ap.add_argument(
        "--git-sha",
        required=True,
        help="40-char git SHA at capture (e.g. $GITHUB_SHA)",
    )
    ap.add_argument(
        "--output",
        type=Path,
        default=Path("tier3_baseline.json"),
        help="output path",
    )
    ap.add_argument(
        "run_bundle_dirs",
        nargs="+",
        type=Path,
        help="one capture bundle per independent run: must contain subdirectory "
        "`criterion/` (Criterion tree) and `rustc_version.txt` from that run's host",
    )
    args = ap.parse_args()

    sha = args.git_sha.strip()
    if len(sha) != 40 or any(c not in "0123456789abcdef" for c in sha.lower()):
        raise SystemExit(f"--git-sha must be a full 40-char hex SHA, got {sha!r}")

    if not EXTRACT.is_file():
        raise SystemExit(f"missing extraction helper: {EXTRACT}")

    run_bundles = [p.resolve() for p in args.run_bundle_dirs]
    if len(run_bundles) < 3:
        raise SystemExit("need at least 3 run directories (procedure N≥3)")
    rustc_line = read_consistent_rustc_version(run_bundles)
    criterion_roots = []
    for bundle in run_bundles:
        root = bundle / "criterion"
        if not root.is_dir():
            raise SystemExit(f"missing criterion/ under capture bundle {bundle}")
        criterion_roots.append(root)
    mg = mirror_groups_from_runs(criterion_roots)

    doc = {
        "$schema": "C1 Phase 1 baseline format v1",
        "captured_on": {
            "host_id": args.host_id,
            "git_sha": sha.lower(),
            "criterion_version": criterion_version_from_lock(REPO_ROOT),
            "rustc_version": rustc_line,
            "captured_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        },
        "mirror_groups": mg,
    }
    args.output.write_text(json.dumps(doc, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
