#!/usr/bin/env python3
"""Path-(a) extraction helper for Tier-3 perf-budget capture (C1 Phase 1 / Phase 2).

Reads a Criterion 0.5 ``sample.json`` file (e.g.
``target/criterion/<bench>/new/sample.json``) and reports ``median_ns`` and
``p99_ns`` per the procedure at ``docs/audit/c1-tier3-baseline-capture-procedure.md``
§2.1 path (a).

Criterion's ``sample.json`` shape stores ``times`` (total ns per sample slot)
and ``iters`` (iteration counts per sample slot). Per-iteration ns is
``times[i] / iters[i]``; the helper sorts the resulting per-iteration ns values
and reports ``median_ns`` (rank ``n/2``, linear-interpolation between the two
middle order statistics for even ``n``) and ``p99_ns`` (rank ``0.99 * (n-1)``,
linear interpolation between adjacent order statistics).

Output is one JSON object per invocation, matching the ``benches[*]`` row shape
in ``tier3_baseline.json``'s schema (procedure §4):

    { "name": "<bench_name>", "median_ns": <int>, "p99_ns": <int> }

``median_ns`` and ``p99_ns`` are rounded to integer nanoseconds before emit
(procedure §5 rule 4: no decimal/float timings; rejects platform float drift).

Single-authority for the path-(a) extraction; called once per bench by the
N=5 multi-run capture orchestration (procedure §2.2). The Rust runtime
evaluator (``compute_perf_budget_bounds``) and ``.dag data`` ``PerfBaselineMeasurement``
declarations consume the aggregated output downstream.

Steady-state per ROADMAP §"P2 — structural compression" row "Tier-3 perf-budget
consumer-slice steady-state hand-Rust surfaces" (Python extraction helper is
owned by the capture procedure's authority document, not this lane).
"""

import argparse
import json
import sys
from pathlib import Path


def per_iter_ns(sample_json: dict) -> list[float]:
    """Normalize Criterion 0.5 sample.json into per-iteration ns values."""
    times = sample_json["times"]
    iters = sample_json["iters"]
    if len(times) != len(iters):
        raise ValueError(
            f"sample.json corruption: len(times)={len(times)} != len(iters)={len(iters)}"
        )
    if not times:
        raise ValueError("sample.json contains no measurement slots")
    return [float(t) / float(it) for t, it in zip(times, iters)]


def quantile(sorted_values: list[float], q: float) -> float:
    """Linear-interpolation quantile (rank = q * (n - 1)).

    Matches the algorithm referenced by procedure §2.1 path (a):
    "linear interpolation between order-statistic ranks; sample size 100 → p99
    ≈ value at rank 99 / 100".
    """
    n = len(sorted_values)
    if n == 0:
        raise ValueError("cannot compute quantile of empty sample")
    if n == 1:
        return sorted_values[0]
    rank = q * (n - 1)
    lower = int(rank)
    upper = min(lower + 1, n - 1)
    fraction = rank - lower
    return sorted_values[lower] + fraction * (sorted_values[upper] - sorted_values[lower])


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "sample_json", type=Path, help="path to target/criterion/<bench>/new/sample.json"
    )
    parser.add_argument(
        "--name",
        required=True,
        help="bench name (matches `bench_function` arg, e.g. `tier3_termination_merge_evidence`)",
    )
    args = parser.parse_args()

    if not args.sample_json.is_file():
        print(f"sample.json not found: {args.sample_json}", file=sys.stderr)
        return 2

    sample_json = json.loads(args.sample_json.read_text())
    per_iter = per_iter_ns(sample_json)
    per_iter.sort()
    median = quantile(per_iter, 0.5)
    p99 = quantile(per_iter, 0.99)

    if median <= 0 or p99 <= 0:
        print(
            f"non-positive measurement (procedure §5 rule 4): median={median} p99={p99}",
            file=sys.stderr,
        )
        return 3
    if p99 < median:
        print(
            f"p99 < median (procedure §5 rule 3 sanity band violation): "
            f"median={median} p99={p99}",
            file=sys.stderr,
        )
        return 4

    output = {
        "name": args.name,
        "median_ns": round(median),
        "p99_ns": round(p99),
    }
    print(json.dumps(output))
    return 0


if __name__ == "__main__":
    sys.exit(main())
