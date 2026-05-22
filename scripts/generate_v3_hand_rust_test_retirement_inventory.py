#!/usr/bin/env python3
"""Audit helper: count and heuristically classify v3 hand-Rust *paths* under the census.

This is NOT a maintained `(file, fn) -> bucket` registry (docs/modeling-discipline.md).
It reads `EXPECTED_HAND_AUTHORED_TEST` from sg0_census_test.rs and prints ephemeral
summaries for PR review. Durable authority: census paths + inline sg0 comments +
operator work-items.

Usage:
  python3 scripts/generate_v3_hand_rust_test_retirement_inventory.py --check
  python3 scripts/generate_v3_hand_rust_test_retirement_inventory.py --summary
  python3 scripts/generate_v3_hand_rust_test_retirement_inventory.py --by-file > /tmp/by-file.jsonl

There is intentionally no --by-test mode in this checked-in script.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CENSUS_FILE = ROOT / "src/v3/compiler/tests/integration/sg0_census_test.rs"
# HEAD baseline for fail-closed --check. When execution PRs shrink the census,
# bump this constant to match len(EXPECTED_HAND_AUTHORED_TEST) — do not weaken
# --check by editing sg0_census_test.rs to satisfy a stale number.
EXPECTED_CENSUS_PATHS = 145


def strip_rust_comments(src: str) -> str:
    out: list[str] = []
    i, n = 0, len(src)
    while i < n:
        if src[i : i + 2] == "//":
            i += 2
            while i < n and src[i] != "\n":
                i += 1
        elif src[i : i + 2] == "/*":
            i += 2
            while i < n - 1 and src[i : i + 2] != "*/":
                i += 1
            i += 2
        else:
            out.append(src[i])
            i += 1
    return "".join(out)


def census_test_paths() -> list[str]:
    if not CENSUS_FILE.is_file():
        raise FileNotFoundError(f"census const missing: {CENSUS_FILE}")
    text = CENSUS_FILE.read_text(encoding="utf-8")
    start = text.index("const EXPECTED_HAND_AUTHORED_TEST")
    end = text.index("const EXPECTED_HAND_AUTHORED_FRAGMENTS", start)
    return re.findall(r'"([^"]+\.rs)"', text[start:end])


def extract_tests(rel: str, *, strict: bool) -> list[str]:
    path = ROOT / rel
    if not path.is_file():
        if strict:
            raise FileNotFoundError(f"census path not on disk: {rel}")
        return []
    src = strip_rust_comments(path.read_text(encoding="utf-8"))
    lines = src.splitlines()
    tests: list[str] = []
    for i, line in enumerate(lines):
        if re.search(r"#\[(?:ignore[^\]]*\)\s*)?test\]", line):
            for j in range(i + 1, min(i + 8, len(lines))):
                match = re.match(r"\s*(?:async\s+)?fn\s+(\w+)", lines[j])
                if match:
                    tests.append(match.group(1))
                    break
    return tests


def classify_path(rel: str) -> tuple[str, str]:
    p = rel.lower()
    if "/boundary/" in p:
        return "KEEP-AS-RUST", "Class-5 boundary"
    if "sg0_census" in p or "sg6_hand" in p or re.search(r"sg[1237]_", p):
        return "KEEP-AS-RUST", "SG-0/SG-* census ratchet"
    if "test_runner_test" in p or "t_pb_b_1_dag_runner" in p:
        return "KEEP-AS-RUST", "TestRunner harness"
    if Path(rel).name == "integration.rs" or "determinism_test" in p:
        return "KEEP-AS-RUST", "Crate wiring"
    if "/common/" in p:
        return "KEEP-AS-RUST", "Shared helper module"
    if "v2_oracle" in p:
        return "KEEP-AS-RUST", "G-1 v2 excision ratchet"
    if re.search(r"v4_.*smoke", p) or "v4_test_bootstrap" in p:
        return "DELETE", "v4 inverted-dependency smoke"
    if "/cementing/" in p:
        return "DELETE", "Band-C cementing (Rust leg)"
    if "m1_substrate_test" in p:
        return "DELETE", "Substrate walks bulk"
    if "m0_acceptance" in p or "four_fixture" in p or "m2_feature_parity" in p:
        return "DELETE", "Obsolete milestone"
    if "migration_test" in p or "canonical_lens" in p or "bridge_lower" in p:
        return "DELETE", "Transitional ratchet"
    if "lens_behavioral_parity" in p:
        return "DELETE", "v2-oracle snapshot"
    if any(
        x in p
        for x in (
            "r3_free_consequences",
            "r3_pb_runtime",
            "r3_verification_l4",
            "r3_sg0_non_test",
            "r3_lens_producer",
            "r3_substrate_gap",
        )
    ):
        return "DELETE", "Host .dag claim driver"
    if "e_i_lane" in p or "r1_release_acceptance" in p:
        return "DELETE", "One-shot / release wrapper"
    if any(x in p for x in ("idempotency_lens_instance", "prereq_x", "tc1_", "wiring_scanner")):
        return "DELETE", "Blocker/meta ratchet"
    if "r3_gate_62" in p:
        return "KEEP-AS-RUST", "Filesystem audit"
    if any(x in p for x in ("pb1_bootstrap_full_snapshot", "r3_v3_self_host", "l5_cross_target")):
        return "KEEP-AS-RUST", "Host / fixture bridge"
    if re.search(r"m1_[34]_emit_|m1_5_emit_omni|m2_emit_multi", p):
        return "KEEP-AS-RUST", "Emit boundary"
    if "v4_" in p:
        return "DELETE", "v4-adjacent"
    if "anthropic" in p:
        return "REPLACE-VIA-TESTCLAIM", "Provider wire"
    return "REPLACE-VIA-TESTCLAIM", "Portable to TestClaim"


def path_rows(*, strict: bool) -> list[dict]:
    rows: list[dict] = []
    for rel in sorted(census_test_paths()):
        n = len(extract_tests(rel, strict=strict))
        bucket, reason = classify_path(rel)
        rows.append(
            {
                "path": rel,
                "test_count": n,
                "bucket": bucket,
                "reason": reason,
            }
        )
    return rows


def aggregate(rows: list[dict]) -> dict:
    by_bucket: Counter[str] = Counter()
    test_total = 0
    for row in rows:
        by_bucket[row["bucket"]] += row["test_count"]
        test_total += row["test_count"]
    return {
        "census_paths": len(rows),
        "test_functions": test_total,
        "by_bucket": dict(by_bucket),
    }


def run_check() -> int:
    try:
        rows = path_rows(strict=True)
    except FileNotFoundError as exc:
        print(f"ERROR {exc}", file=sys.stderr)
        return 1
    summary = aggregate(rows)
    errors: list[str] = []
    if summary["census_paths"] != EXPECTED_CENSUS_PATHS:
        errors.append(
            f"census path count {summary['census_paths']} != expected {EXPECTED_CENSUS_PATHS}"
        )
    if errors:
        for msg in errors:
            print(f"ERROR {msg}", file=sys.stderr)
        return 1
    print(
        f"OK paths={summary['census_paths']} tests={summary['test_functions']} "
        f"DELETE={summary['by_bucket'].get('DELETE', 0)} "
        f"REPLACE={summary['by_bucket'].get('REPLACE-VIA-TESTCLAIM', 0)} "
        f"KEEP={summary['by_bucket'].get('KEEP-AS-RUST', 0)}",
        file=sys.stderr,
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="Fail closed: every census path must exist; path count must match HEAD baseline",
    )
    parser.add_argument("--summary", action="store_true", help="Print summary JSON to stdout")
    parser.add_argument(
        "--by-file",
        action="store_true",
        help="Print path-level rows to stdout (<=145 lines); missing paths are errors",
    )
    args = parser.parse_args()
    if args.check:
        return run_check()
    strict = bool(args.by_file or args.summary)
    try:
        rows = path_rows(strict=strict)
    except FileNotFoundError as exc:
        print(f"ERROR {exc}", file=sys.stderr)
        return 1
    summary = aggregate(rows)
    if args.by_file:
        for row in rows:
            print(json.dumps(row))
        return 0
    if args.summary:
        print(json.dumps(summary, indent=2))
        return 0
    parser.print_help()
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
