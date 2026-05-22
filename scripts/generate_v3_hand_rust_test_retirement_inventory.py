#!/usr/bin/env python3
"""Generate a transient JSONL export of v3 hand-Rust test retirement classifications.

NOT a maintained authority (docs/modeling-discipline.md standing rule). Output is
a review-time worksheet only — regenerate from HEAD when needed. Durable facts:
  - path membership: EXPECTED_HAND_AUTHORED_TEST in sg0_census_test.rs
  - retirement execution: dashboard work-items + sg0 census shrinkage in the PR that deletes tests

Usage:
  python3 scripts/generate_v3_hand_rust_test_retirement_inventory.py > /tmp/inventory.jsonl
  python3 scripts/generate_v3_hand_rust_test_retirement_inventory.py --check  # assert unique (file, fn)

Audit prose (human review): docs/audit/v3-hand-rust-test-retirement.md
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
    text = CENSUS_FILE.read_text(encoding="utf-8")
    start = text.index("const EXPECTED_HAND_AUTHORED_TEST")
    end = text.index("const EXPECTED_HAND_AUTHORED_FRAGMENTS", start)
    return re.findall(r'"([^"]+\.rs)"', text[start:end])


def extract_tests(rel: str) -> list[str]:
    path = ROOT / rel
    if not path.is_file():
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


def classify_file(rel: str) -> tuple[str, str]:
    p = rel.lower()
    if "/boundary/" in p:
        return "KEEP-AS-RUST", "Class-5 boundary roundtrip"
    if "sg0_census" in p or "sg6_hand" in p or re.search(r"sg[1237]_", p):
        return "KEEP-AS-RUST", "SG-0/SG-* census ratchet until census zero"
    if "test_runner_test" in p or "t_pb_b_1_dag_runner" in p:
        return "KEEP-AS-RUST", "Host TestRunner for .dag TestClaims"
    if Path(rel).name == "integration.rs" or "determinism_test" in p:
        return "KEEP-AS-RUST", "Crate wiring / determinism infrastructure"
    if "/common/" in p:
        return "KEEP-AS-RUST", "Shared test helper (may have zero #[test])"
    if "v2_oracle" in p:
        return "KEEP-AS-RUST", "G-1 v2-consumer excision until src/v2/ removed"
    if re.search(r"v4_.*smoke", p) or "v4_test_bootstrap" in p:
        return "DELETE", "Inverted-dependency v3 parse smoke of v4 .dag"
    if "/cementing/" in p:
        return "DELETE", "Band-C cement; .dag gate-87 harness is authority"
    if "m1_substrate_test" in p:
        return "DELETE", "Imperative substrate walks"
    if "m0_acceptance" in p or "four_fixture" in p or "m2_feature_parity" in p:
        return "DELETE", "Obsolete milestone receipt"
    if "migration_test" in p or "canonical_lens" in p or "bridge_lower" in p:
        return "DELETE", "Transitional ratchet superseded"
    if "lens_behavioral_parity" in p:
        return "DELETE", "Temporary v2-oracle snapshot"
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
        return "DELETE", "Host driver for .dag TestClaims"
    if "e_i_lane" in p or "r1_release_acceptance" in p:
        return "DELETE", "One-shot preflight / release-wrapper"
    if any(x in p for x in ("idempotency_lens_instance", "prereq_x", "tc1_", "wiring_scanner")):
        return "DELETE", "Blocker/meta ratchet only"
    if "r3_gate_62" in p:
        return "KEEP-AS-RUST", "Filesystem tree audit"
    if any(x in p for x in ("pb1_bootstrap_full_snapshot", "r3_v3_self_host", "l5_cross_target")):
        return "KEEP-AS-RUST", "Host process / fixture bridge"
    if re.search(r"m1_[34]_emit_|m1_5_emit_omni|m2_emit_multi", p):
        return "KEEP-AS-RUST", "Emitted-target boundary"
    if "v4_" in p:
        return "DELETE", "v4-adjacent inverted dependency"
    if "anthropic" in p:
        return "REPLACE-VIA-TESTCLAIM", "Provider wire"
    return "REPLACE-VIA-TESTCLAIM", "Structural claim portable to TestClaim"


def t19_for(rel: str, bucket: str) -> str | None:
    if bucket != "REPLACE-VIA-TESTCLAIM":
        return None
    p = rel.lower()
    if "algebra" in p or "symbolic_cost" in p:
        return "AlgebraLaw"
    if "diagnostic" in p or "gate_106" in p or "reject" in p:
        return "DiagnosticExhaustiveness"
    if "lens" in p or "cost" in p or "parallelism" in p:
        return "LensApplicability"
    if "anthropic" in p or "emit" in p:
        return "LanguageBehaviorEquivalence"
    if "l5_" in p or "cross_target" in p:
        return "BidirectionalRoundtrip"
    if "gate_62" in p:
        return "T-19-CATEGORY-MISSING: RepoFileTreeNegativeBridgeAudit"
    if "ctrl_pr" in p:
        return "T-19-CATEGORY-MISSING: ModuleServiceParseSurface"
    return "TypeConstruction"


def generate_rows() -> list[dict]:
    seen: set[tuple[str, str]] = set()
    rows: list[dict] = []
    for rel in sorted(census_test_paths()):
        bucket, reason = classify_file(rel)
        tests = extract_tests(rel)
        if not tests:
            key = (rel, "(path-only)")
            if key in seen:
                raise ValueError(f"duplicate {key}")
            seen.add(key)
            rows.append(
                {
                    "file": rel,
                    "fn": "(path-only)",
                    "bucket": bucket,
                    "reason": reason + " [zero #[test] in file]",
                    "t19": t19_for(rel, bucket),
                }
            )
            continue
        for fn in tests:
            key = (rel, fn)
            if key in seen:
                raise ValueError(f"duplicate {key}")
            seen.add(key)
            rows.append(
                {
                    "file": rel,
                    "fn": fn,
                    "bucket": bucket,
                    "reason": reason,
                    "t19": t19_for(rel, bucket),
                }
            )
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="Validate uniqueness and print bucket counts to stderr; no stdout JSONL",
    )
    args = parser.parse_args()
    rows = generate_rows()
    test_rows = [r for r in rows if r["fn"] != "(path-only)"]
    counts = Counter(r["bucket"] for r in test_rows)
    if args.check:
        paths = {r["file"] for r in rows}
        print(
            f"OK paths={len(paths)} tests={len(test_rows)} "
            f"DELETE={counts['DELETE']} REPLACE={counts['REPLACE-VIA-TESTCLAIM']} "
            f"KEEP={counts['KEEP-AS-RUST']}",
            file=sys.stderr,
        )
        return 0
    for row in rows:
        print(json.dumps(row))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
