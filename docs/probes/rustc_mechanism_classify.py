#!/usr/bin/env python3
# PROBE INSTRUMENT (measurement only; never imported by production code).
# dissolve-on: the emitted self-host board reaches zero, or a modeled .dag carrier reads the
# diagnostic stream directly and owns the cross-code mechanism identity.
"""Rustc diagnostic manifestations -> cross-code mechanism rows.

The error code is an observation, not a root identity. E0308 blocks keep the canonical
manifestation expansion defined by e0308_classify_sites; every other coded rustc block contributes
one manifestation. A mechanism discriminator may therefore collect rows across codes without
silently changing the E0308 instrument's unit.

This first classifier deliberately recognizes one cross-code mechanism. Every other row is
UNCLASSIFIED rather than being guessed from its code. The E0308 candidate label is retained in a
separate projection column so the old partition remains reproducible but cannot masquerade as a
cross-code root partition.
"""
import collections
import csv
import re
import sys

import e0308_classify_sites as e0308


CLONE_E0277 = re.compile(r"^the trait bound `[^`]+: Clone` is not satisfied$")
CLONE_E0599 = re.compile(r"^no method named `clone` found for type parameter `[^`]+` in the current scope$")


def absent_clone_bound(code, message, lines, e0308_candidate):
    """Decide the one established cross-code mechanism from rustc's typed explanation."""
    body = "\n".join(lines)
    if code == "E0308":
        return e0308_candidate == "A-clone" and (
            "does not implement `Clone`, so `" in body and "was cloned instead" in body
        )
    if code == "E0277":
        return bool(CLONE_E0277.match(message))
    if code == "E0599":
        return bool(CLONE_E0599.match(message))
    return False


def primary_span(lines):
    for line in lines:
        match = e0308.SPAN.match(line)
        if match:
            return match.groups()
    return "", "", ""


def classify(path):
    rows = []
    e0308_seen = set()
    blocks = e0308.read_blocks(path, codes=None)
    for block_index, block in enumerate(blocks, 1):
        code = block["code"]
        if code == "E0308":
            manifestations = e0308.block_sites(block)
        else:
            file, line, col = primary_span(block["lines"])
            manifestations = [dict(file=file, line=line, col=col, expected="", found="")]
        for manifestation_index, site in enumerate(manifestations, 1):
            if code == "E0308":
                key = (site["file"], site["line"], site["col"],
                       frozenset((site["expected"], site["found"])))
                if key in e0308_seen:
                    continue
                e0308_seen.add(key)
            candidate = ""
            if code == "E0308":
                candidate, _ = e0308.classify(site)
            mechanism = (
                "ABSENT_CLONE_BOUND"
                if absent_clone_bound(code, block["msg"], block["lines"], candidate)
                else "UNCLASSIFIED"
            )
            rows.append({
                "code": code,
                "block": block_index,
                "manifestation": manifestation_index,
                "file": site.get("file") or "",
                "line": site.get("line") or "",
                "col": site.get("col") or "",
                "expected": site.get("expected", ""),
                "found": site.get("found", ""),
                "mechanism": mechanism,
                "e0308_candidate_projection": candidate,
                "message": block["msg"],
            })
    return rows


def main(path, out):
    rows = classify(path)
    columns = ["code", "block", "manifestation", "file", "line", "col", "expected", "found",
               "mechanism", "e0308_candidate_projection", "message"]
    with open(out, "w", newline="", encoding="utf-8") as output:
        writer = csv.DictWriter(output, fieldnames=columns, dialect="excel-tab", lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)

    by_code = collections.Counter(row["code"] for row in rows)
    clone_by_code = collections.Counter(
        row["code"] for row in rows if row["mechanism"] == "ABSENT_CLONE_BOUND"
    )
    clone_total = sum(clone_by_code.values())
    print("coded diagnostic manifestations: %d" % len(rows))
    print("ABSENT_CLONE_BOUND: %d (%.1f%%)" % (clone_total, 100.0 * clone_total / len(rows)))
    print("  by code: %s" % dict(sorted(clone_by_code.items())))
    print("  board by code: %s" % dict(sorted(by_code.items())))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
