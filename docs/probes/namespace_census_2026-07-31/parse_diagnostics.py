#!/usr/bin/env python3
"""Fail closed while normalizing the pinned whole-corpus compiler log."""

import argparse
import collections
import hashlib
import json
import pathlib
import re

from receipt_common import load_summary

HEADER = re.compile(r"^v2 self-compile produced (\d+) hard diagnostic\(s\):$")
LOCATION = r" \([^()]+:\d+-\d+\)$"
PATTERNS = {
    "unresolved_name": (
        re.compile(r"^function '[^']+' not found in scope" + LOCATION),
        re.compile(r"^undefined variable '[^']+'" + LOCATION),
        re.compile(r"^variant '[^']+' not found in type '[^']+'" + LOCATION),
        re.compile(r"^unresolved type '?[^'\s]+'?" + LOCATION),
        re.compile(r"^type '[^']+' not found[^()]*" + LOCATION),
    ),
    "ambiguous_variant_synthetic_root_diagnostic": (re.compile(r"^variant '[^']+' appears in both '[^']+' and '[^']+'" + LOCATION),),
    "no_field": (re.compile(r"^no field '[^']+' on type '[^']+'" + LOCATION),),
    "type_mismatch": (re.compile(r"^type mismatch: expected '[^']+', got '[^']+'" + LOCATION),),
}
SINGLETONS = (
    re.compile(r"^name '[^']+' not found in module '[^']+' \(imported by '[^']+'\)" + LOCATION),
    re.compile(r"^unresolved import: module '[^']+' not found \(imported by '[^']+'\)" + LOCATION),
    re.compile(r"^empty_map\(\): expected type is not a keyed collection" + LOCATION),
    re.compile(r"^set_contains receiver must be a Set" + LOCATION),
    re.compile(r"^if branches resolve to incompatible types: .+" + LOCATION),
    re.compile(r"^non-exhaustive match: missing variant\(s\) .+" + LOCATION),
)
NOISE = re.compile(r"^(indexed |resolved |\[census\]|◐|✓|✗|◷|\s*$)")
POPULATION = (
    ("function", re.compile(r"^function '([^']+)' not found in scope \(([^:]+):\d+-\d+\)$")),
    ("variable", re.compile(r"^undefined variable '([^']+)' \(([^:]+):\d+-\d+\)$")),
    ("variant", re.compile(r"^variant '([^']+)' not found in type '([^']+)' \(([^:]+):\d+-\d+\)$")),
    ("type", re.compile(r"^unresolved type '?([^'\s]+)'? \(([^:]+):\d+-\d+\)$")),
    ("type", re.compile(r"^type '([^']+)' not found[^()]*\(([^:]+):\d+-\d+\)$")),
)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("log", type=pathlib.Path)
    parser.add_argument("result_json", type=pathlib.Path)
    parser.add_argument("--summary-json", type=pathlib.Path, required=True)
    parser.add_argument("--population-json", type=pathlib.Path)
    parser.add_argument("--ambiguity-json", type=pathlib.Path)
    args = parser.parse_args()
    summary = load_summary(args.summary_json)
    raw = args.log.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    expected_digest = summary["inputs"]["raw_log_sha256"]
    if digest != expected_digest:
        raise SystemExit(f"raw log digest mismatch: {digest}")

    counts: collections.Counter[str] = collections.Counter()
    headers: list[int] = []
    unparsed: list[str] = []
    population: list[dict[str, str | None]] = []
    ambiguities: list[dict[str, str]] = []
    for line in raw.decode("utf-8", errors="replace").splitlines():
        header = HEADER.fullmatch(line)
        if header:
            headers.append(int(header.group(1)))
            continue
        if NOISE.match(line):
            continue
        matched = False
        for category, patterns in PATTERNS.items():
            if any(pattern.fullmatch(line) for pattern in patterns):
                counts[category] += 1
                matched = True
                break
        if not matched and any(pattern.fullmatch(line) for pattern in SINGLETONS):
            counts["singleton"] += 1
            matched = True
        if not matched:
            unparsed.append(line)
        elif category == "unresolved_name":
            for population_category, pattern in POPULATION:
                match = pattern.fullmatch(line)
                if match is None:
                    continue
                groups = match.groups()
                if population_category == "variant":
                    symbol, owner, file = groups
                else:
                    symbol, file = groups
                    owner = None
                population.append({"category": population_category, "symbol": symbol,
                                   "owner_type": owner, "file": file})
                break
        elif category == "ambiguous_variant_synthetic_root_diagnostic":
            match = re.fullmatch(
                r"variant '([^']+)' appears in both '([^']+)' and '([^']+)' \(([^:]+):\d+-\d+\)",
                line,
            )
            if match is None:
                raise SystemExit(f"ambiguity extraction drift: {line}")
            variant, first, second, file = match.groups()
            ambiguities.append({"variant": variant, "a": first, "b": second, "file": file})

    if unparsed:
        raise SystemExit("unparsed diagnostics:\n" + "\n".join(unparsed[:20]))
    if len(headers) != 1:
        raise SystemExit(f"expected exactly one compiler diagnostic header, got {headers}")
    normalized = {key: counts[key] for key in PATTERNS}
    normalized["singleton"] = counts["singleton"]
    classified = sum(normalized.values())
    if classified != headers[0]:
        raise SystemExit(f"classification sum {classified} != compiler total {headers[0]}")
    if len(population) != normalized["unresolved_name"]:
        raise SystemExit(f"population extraction drift: {len(population)}")
    if args.population_json:
        args.population_json.write_text(json.dumps({"rows": population}, indent=2) + "\n")
    if len(ambiguities) != normalized["ambiguous_variant_synthetic_root_diagnostic"]:
        raise SystemExit(f"ambiguity extraction drift: {len(ambiguities)}")
    if args.ambiguity_json:
        args.ambiguity_json.write_text(json.dumps({"occurrences": ambiguities}, indent=2) + "\n")
    result = {
        "authority": "compiler-authoritative",
        "compiler_reported_hard_diagnostics": headers[0],
        "classification": normalized,
        "classification_sum": classified,
        "header_lines": 1,
        "raw_log_sha256": digest,
    }
    args.result_json.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
