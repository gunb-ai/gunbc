#!/usr/bin/env python3
"""Group ambiguity occurrences using an explicitly non-semantic heuristic.

PROVENANCE LABEL: INFERRED GROUPING. Neither grouping occurrences into decisions
nor assigning A/B/C is compiler output. A decision key is the variant name plus an
unordered pair of candidate type names. The parallel-tower rule is a suffix
heuristic chosen by inspection, not a semantic compiler judgment.
"""

import argparse
import collections
import json
import pathlib

TOWER_SUFFIXES = (
    "IntWidth", "FloatWidth", "IntKind", "Scalar", "ScalarKind",
    "FloatBits", "Signedness", "NonIntegerScalar",
)


def classify(first: str, second: str) -> str:
    if first == second:
        return "A_SELF"
    if (any(first.endswith(suffix) for suffix in TOWER_SUFFIXES)
            and any(second.endswith(suffix) for suffix in TOWER_SUFFIXES)):
        return "B_PARALLEL_TOWER"
    return "C_TRUE_HOMONYM"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("ambiguity_json", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    args = parser.parse_args()
    occurrences = json.loads(args.ambiguity_json.read_text())["occurrences"]
    decisions = collections.OrderedDict()
    for occurrence in occurrences:
        key = (occurrence["variant"], *sorted((occurrence["a"], occurrence["b"])))
        decision = decisions.setdefault(key, {
            "variant": key[0], "candidates": [key[1], key[2]],
            "class": classify(key[1], key[2]), "occurrences": 0, "files": [],
        })
        decision["occurrences"] += 1
        decision["files"].append(occurrence["file"])
    rows = list(decisions.values())
    decision_counts = collections.Counter(row["class"] for row in rows)
    occurrence_counts = collections.Counter()
    for row in rows:
        occurrence_counts[row["class"]] += row["occurrences"]
    if sum(decision_counts.values()) != len(rows) or sum(occurrence_counts.values()) != len(occurrences):
        raise SystemExit("inferred grouping sum invariant failed")
    result = {"authority": "inferred-grouping", "decision_key": "variant + unordered candidate pair",
              "decisions": rows, "decision_counts": dict(decision_counts),
              "occurrence_counts": dict(occurrence_counts)}
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({key: value for key, value in result.items() if key != "decisions"},
                     indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
