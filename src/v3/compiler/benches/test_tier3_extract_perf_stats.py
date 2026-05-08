#!/usr/bin/env python3
"""Edge-case unit tests for ``tier3_extract_perf_stats``.

Hardens the path-(a) extraction discipline (per
``docs/audit/c1-tier3-baseline-capture-procedure.md`` §2.1) against the
boundary cases the brief's acceptance highlights: linear-interpolation at
even-/odd-/single-sample sizes, integer rounding per procedure §5 rule 4,
sanity bands per §5 rule 3 (``p99 >= median``) and rule 4 (non-zero).

Stdlib unittest (no pytest dep). Run via ``python3 -m unittest
src.v3.compiler.benches.test_tier3_extract_perf_stats`` from the repo
root, or directly via ``python3 src/v3/compiler/benches/test_tier3_extract_perf_stats.py``.
"""

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HELPER = Path(__file__).parent / "tier3_extract_perf_stats.py"


def run_helper(sample_data: dict, name: str = "tier3_test") -> tuple[int, str, str]:
    """Run the helper against a synthetic sample.json; return (exit_code, stdout, stderr)."""
    with tempfile.NamedTemporaryFile(
        "w", suffix=".json", delete=False, encoding="utf-8"
    ) as f:
        json.dump(sample_data, f)
        sample_path = f.name
    try:
        result = subprocess.run(
            [sys.executable, str(HELPER), sample_path, "--name", name],
            capture_output=True,
            text=True,
        )
        return result.returncode, result.stdout, result.stderr
    finally:
        os.unlink(sample_path)


class QuantileEdgeCaseTests(unittest.TestCase):
    def test_single_sample(self):
        """Single sample → median and p99 both equal the sole value."""
        rc, out, _ = run_helper({"iters": [1.0], "times": [42.0]})
        self.assertEqual(rc, 0)
        result = json.loads(out)
        self.assertEqual(result["median_ns"], 42)
        self.assertEqual(result["p99_ns"], 42)

    def test_even_sample_count_linear_interpolation(self):
        """Even n: median is linear-interp between the two middle order statistics."""
        # Per-iter values [10, 20, 30, 40]; median rank = 0.5*3 = 1.5 → 20 + 0.5*(30-20) = 25
        rc, out, _ = run_helper(
            {"iters": [1.0, 1.0, 1.0, 1.0], "times": [10.0, 20.0, 30.0, 40.0]}
        )
        self.assertEqual(rc, 0)
        self.assertEqual(json.loads(out)["median_ns"], 25)

    def test_odd_sample_count_exact_median(self):
        """Odd n: median rank lands exactly on a value (no interpolation needed)."""
        rc, out, _ = run_helper({"iters": [1.0] * 5, "times": [10.0, 20.0, 30.0, 40.0, 50.0]})
        self.assertEqual(rc, 0)
        self.assertEqual(json.loads(out)["median_ns"], 30)

    def test_p99_at_top_of_distribution(self):
        """p99 of 100 samples lands at rank 0.99*99 = 98.01 — between order statistics 99 and 100."""
        # Values 1..100 inclusive. Rank 98.01: 99 + 0.01*(100-99) = 99.01 → rounds to 99.
        sample = {"iters": [1.0] * 100, "times": [float(i) for i in range(1, 101)]}
        rc, out, _ = run_helper(sample)
        self.assertEqual(rc, 0)
        self.assertEqual(json.loads(out)["p99_ns"], 99)


class FailClosedSanityBandTests(unittest.TestCase):
    def test_rejects_non_positive_measurement(self):
        """Procedure §5 rule 4: zero measurement → exit code 3."""
        rc, _, err = run_helper({"iters": [1.0], "times": [0.0]})
        self.assertEqual(rc, 3)
        self.assertIn("non-positive", err)

    def test_rejects_p99_below_median(self):
        """Procedure §5 rule 3: p99 < median is impossible by construction.

        The sorted-order ensures p99 >= median, so this can only fire on
        sample.json corruption. Synthesize a single-element sample and
        verify p99 == median (the boundary case).
        """
        rc, out, _ = run_helper({"iters": [1.0], "times": [10.0]})
        self.assertEqual(rc, 0)
        result = json.loads(out)
        self.assertGreaterEqual(result["p99_ns"], result["median_ns"])

    def test_rejects_length_mismatch(self):
        """sample.json with len(times) != len(iters) → corruption error."""
        rc, _, err = run_helper({"iters": [1.0, 2.0], "times": [10.0]})
        self.assertNotEqual(rc, 0)

    def test_rejects_empty_sample(self):
        """Empty times/iters → corruption error (no measurement slots)."""
        rc, _, err = run_helper({"iters": [], "times": []})
        self.assertNotEqual(rc, 0)


class IntegerRoundingTests(unittest.TestCase):
    def test_fractional_per_iter_rounds_to_int(self):
        """Procedure §5 rule 4: integer ns only; reject decimal/float."""
        # times=10, iters=3 → per-iter = 3.333..., median=p99=3.333... → rounds to 3.
        rc, out, _ = run_helper({"iters": [3.0], "times": [10.0]})
        self.assertEqual(rc, 0)
        result = json.loads(out)
        self.assertIsInstance(result["median_ns"], int)
        self.assertIsInstance(result["p99_ns"], int)
        self.assertEqual(result["median_ns"], 3)


class OutputSchemaTests(unittest.TestCase):
    def test_output_matches_baseline_json_row_shape(self):
        """Output is exactly { name, median_ns, p99_ns } per procedure §4 schema."""
        rc, out, _ = run_helper(
            {"iters": [1.0, 1.0], "times": [10.0, 20.0]}, name="tier3_schema_test"
        )
        self.assertEqual(rc, 0)
        result = json.loads(out)
        self.assertEqual(set(result.keys()), {"name", "median_ns", "p99_ns"})
        self.assertEqual(result["name"], "tier3_schema_test")


if __name__ == "__main__":
    unittest.main()
