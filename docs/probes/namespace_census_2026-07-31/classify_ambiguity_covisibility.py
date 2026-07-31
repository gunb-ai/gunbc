#!/usr/bin/env python3
"""Partition synthetic-root ambiguity diagnostics by textual co-visibility.

PROVENANCE LABEL: INFERRED INSTRUMENT ANALYSIS. Textual co-visibility is necessary,
not sufficient, for genuine co-resolution. CoVisible is therefore an upper bound;
this instrument establishes no lower bound on the real ambiguity population.
"""

import argparse
import json
import pathlib
import re
import subprocess

CORPUS_COMMIT = "0337fb27c039a800a1aff4b80140d6dbf027e595"
AMBIGUITY = re.compile(
    r"variant '([^']+)' appears in both '([^']+)' and '([^']+)' \(([^:]+):\d+-\d+\)"
)
BRACELESS_IMPORT = re.compile(r"^\s*import\s+[\w.]+\s*$", re.MULTILINE)
EXPECTED = {"CoVisible": 42, "BracelessUndecided": 1, "PoolReach": 281}


def corpus_file(repo: pathlib.Path, path: str) -> str:
    result = subprocess.run(
        ["git", "show", f"{CORPUS_COMMIT}:{path}"],
        cwd=repo,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise SystemExit(f"cannot read pinned corpus file {path}: {result.stderr.strip()}")
    return result.stdout


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("repo", type=pathlib.Path)
    parser.add_argument("log", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    args = parser.parse_args()

    occurrences = []
    for line in args.log.read_text(encoding="utf-8", errors="replace").splitlines():
        match = AMBIGUITY.fullmatch(line)
        if match:
            occurrences.append(match.groups())
    contents = {
        site: corpus_file(args.repo.resolve(), site)
        for site in sorted({occurrence[3] for occurrence in occurrences})
    }
    buckets = {name: [] for name in EXPECTED}
    for variant, first, second, site in occurrences:
        source = contents[site]
        if all(re.search(r"\b" + re.escape(owner) + r"\b", source) for owner in (first, second)):
            bucket = "CoVisible"
        elif BRACELESS_IMPORT.search(source):
            bucket = "BracelessUndecided"
        else:
            bucket = "PoolReach"
        buckets[bucket].append({
            "variant": variant, "owner_a": first, "owner_b": second, "site": site,
        })
    counts = {name: len(rows) for name, rows in buckets.items()}
    total = sum(counts.values())
    if total != len(occurrences):
        raise SystemExit(f"partition lost rows: {total} != {len(occurrences)}")
    if counts != EXPECTED or total != 324:
        raise SystemExit(f"co-visibility partition drift: {counts}, total={total}")
    result = {
        "authority": "inferred-instrument-analysis",
        "corpus_commit": CORPUS_COMMIT,
        "synthetic_root_ambiguity_diagnostics": total,
        "counts": counts,
        "real_ambiguity_lower_bound": None,
        "real_ambiguity_upper_bound": counts["CoVisible"],
        "upper_bound_note": "textual co-visibility is necessary, not sufficient",
        "buckets": buckets,
    }
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: value for key, value in result.items() if key != "buckets"},
                     indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
