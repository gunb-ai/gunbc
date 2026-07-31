#!/usr/bin/env python3
"""Partition synthetic-root ambiguity diagnostics by textual owner-name presence.

PROVENANCE LABEL: REPRODUCIBLE TEXTUAL CLASSIFICATION. This predicate does not
constrain semantic visibility in either direction and establishes no bound.
"""

import argparse
import json
import pathlib
import re

from receipt_common import load_summary, require_pinned_repo

AMBIGUITY = re.compile(
    r"variant '([^']+)' appears in both '([^']+)' and '([^']+)' \(([^:]+):\d+-\d+\)"
)
BRACELESS_IMPORT = re.compile(r"^\s*import\s+[\w.]+\s*$", re.MULTILINE)
BUCKET_NAMES = (
    "BothOwnerNamesTextual",
    "BracelessImportTextPresent",
    "BothOwnerNamesNotTextual",
)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("repo", type=pathlib.Path)
    parser.add_argument("log", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    parser.add_argument("--summary-json", type=pathlib.Path, required=True)
    args = parser.parse_args()
    summary = load_summary(args.summary_json)
    repo = require_pinned_repo(args.repo, summary)

    occurrences = []
    for line in args.log.read_text(encoding="utf-8", errors="replace").splitlines():
        match = AMBIGUITY.fullmatch(line)
        if match:
            occurrences.append(match.groups())
    contents = {site: (repo / site).read_text(encoding="utf-8", errors="replace")
                for site in sorted({occurrence[3] for occurrence in occurrences})}
    buckets = {name: [] for name in BUCKET_NAMES}
    for variant, first, second, site in occurrences:
        source = contents[site]
        if all(re.search(r"\b" + re.escape(owner) + r"\b", source) for owner in (first, second)):
            bucket = "BothOwnerNamesTextual"
        elif BRACELESS_IMPORT.search(source):
            bucket = "BracelessImportTextPresent"
        else:
            bucket = "BothOwnerNamesNotTextual"
        buckets[bucket].append({
            "variant": variant, "owner_a": first, "owner_b": second, "site": site,
        })
    counts = {name: len(rows) for name, rows in buckets.items()}
    total = sum(counts.values())
    if total != len(occurrences):
        raise SystemExit(f"partition lost rows: {total} != {len(occurrences)}")
    result = {
        "authority": "reproducible-textual-classification",
        "corpus_commit": summary["inputs"]["corpus_commit"],
        "synthetic_root_ambiguity_diagnostics": total,
        "counts": counts,
        "semantic_visibility_note": "textual owner-name presence constrains semantic visibility in neither direction",
        "buckets": buckets,
    }
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: value for key, value in result.items() if key != "buckets"},
                     indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
