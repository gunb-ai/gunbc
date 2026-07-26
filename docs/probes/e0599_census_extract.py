#!/usr/bin/env python3
"""Extract E0599 census from curated-cargo probe build logs.

Usage:
  e0599_census_extract.py <module.dag> <cargo.log> [--tsv]
  e0599_census_extract.py --aggregate <logdir>/*.cargo.log

Keys each E0599 by (method, receiver_carrier, failure_shape).
failure_shape:
  missing_method — no method named `m` found for ...
  bounds_unsatisfied — the method `m` exists for ..., but its trait bounds were not satisfied
  no_variant — no variant or associated item named ...
  no_assoc_fn — no function or associated item named ...
"""
from __future__ import annotations

import argparse
import collections
import pathlib
import re
import sys

RE_MISSING = re.compile(
    r"error\[E0599\]: no method named `([^`]+)` found for (.+?) in the current scope"
)
RE_BOUNDS = re.compile(
    r"error\[E0599\]: the method `([^`]+)` exists for (.+?), but its trait bounds were not satisfied"
)
RE_VARIANT = re.compile(
    r"error\[E0599\]: no variant(?:, associated function, or constant)? named `([^`]+)` found for (.+?)(?: in the current scope)?$"
)
RE_ASSOC = re.compile(
    r"error\[E0599\]: no function or associated item named `([^`]+)` found for (.+?)(?: in the current scope)?$"
)
RE_OTHER = re.compile(r"error\[E0599\]: (.+)")


def normalize_receiver(raw: str) -> str:
    raw = raw.strip()
    # Collapse whitespace; keep rustc's carrier spelling.
    return " ".join(raw.split())


def classify_line(line: str) -> tuple[str, str, str, str] | None:
    for rx, shape in (
        (RE_MISSING, "missing_method"),
        (RE_BOUNDS, "bounds_unsatisfied"),
        (RE_VARIANT, "no_variant"),
        (RE_ASSOC, "no_assoc_fn"),
    ):
        m = rx.search(line)
        if m:
            return shape, m.group(1), normalize_receiver(m.group(2)), line.strip()
    m = RE_OTHER.search(line)
    if m:
        return "other", "?", normalize_receiver(m.group(1)), line.strip()
    return None


def parse_log(path: pathlib.Path) -> list[tuple[str, str, str, str]]:
    rows: list[tuple[str, str, str, str]] = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        hit = classify_line(line)
        if hit:
            rows.append(hit)
    return rows


def module_from_log(path: pathlib.Path) -> str:
    stem = path.name.removesuffix(".cargo.log")
    return stem


def emit_module_tsv(module: str, rows: list[tuple[str, str, str, str]]) -> None:
    counts: collections.Counter[tuple[str, str, str]] = collections.Counter()
    for shape, method, receiver, _ in rows:
        counts[(shape, method, receiver)] += 1
    print(f"# module={module} total_E0599={len(rows)}")
    print("module\tfailure_shape\tmethod\treceiver_carrier\tcount")
    for (shape, method, receiver), n in sorted(
        counts.items(), key=lambda kv: (-kv[1], kv[0][0], kv[0][1], kv[0][2])
    ):
        print(f"{module}\t{shape}\t{method}\t{receiver}\t{n}")


def emit_aggregate(log_paths: list[pathlib.Path]) -> None:
    per_module: dict[str, list[tuple[str, str, str, str]]] = {}
    global_counts: collections.Counter[tuple[str, str, str]] = collections.Counter()
    module_totals: dict[str, int] = {}

    for path in sorted(log_paths):
        mod = module_from_log(path)
        rows = parse_log(path)
        per_module[mod] = rows
        module_totals[mod] = len(rows)
        for shape, method, receiver, _ in rows:
            global_counts[(shape, method, receiver)] += 1

    print("# e0599_canonical_seven_census aggregate")
    print("module\ttotal_E0599")
    for mod in sorted(module_totals):
        print(f"{mod}\t{module_totals[mod]}")
    print(f"TOTAL\t{sum(module_totals.values())}")
    print()
    print("failure_shape\tmethod\treceiver_carrier\ttotal_count\tmodules_hit")
    mod_sets: dict[tuple[str, str, str], set[str]] = collections.defaultdict(set)
    for mod, rows in per_module.items():
        seen: set[tuple[str, str, str]] = set()
        for shape, method, receiver, _ in rows:
            key = (shape, method, receiver)
            seen.add(key)
        for key in seen:
            mod_sets[key].add(mod)
    for key, n in sorted(global_counts.items(), key=lambda kv: (-kv[1], kv[0])):
        shape, method, receiver = key
        print(
            f"{shape}\t{method}\t{receiver}\t{n}\t{len(mod_sets[key])}"
        )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("paths", nargs="+", type=pathlib.Path)
    ap.add_argument("--aggregate", action="store_true")
    ap.add_argument("--module", default="")
    args = ap.parse_args()

    if args.aggregate or len(args.paths) > 1:
        emit_aggregate(args.paths)
        return 0

    path = args.paths[0]
    module = args.module or module_from_log(path)
    rows = parse_log(path)
    emit_module_tsv(module, rows)
    return 0


if __name__ == "__main__":
    sys.exit(main())
