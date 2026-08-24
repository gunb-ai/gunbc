#!/usr/bin/env python3
# PROBE INSTRUMENT (measurement only; never imported by production code).
# dissolve-on: the emitted self-host board reaches zero across the frontier roster, or a modeled
# .dag carrier owns cross-module mechanism identity and this script's join moves into it.
"""Per-module cargo logs -> cross-module mechanism-root weighting.

WHY THIS EXISTS. Every mechanism root the self-host repair queue is ranked on was traced from a
single entry (03_ingest). A per-module board answers "how big is this root HERE"; nothing answered
"is this root the same defect seen from fifteen doors, or fifteen defects". Those have opposite
staffing consequences and the same-looking histogram.

THE UNIT, AND WHY SUMS ARE REFUSED. Module closures overlap: `src/v2_std_algebra.rs` is emitted
into essentially every module's crate. Summing per-module manifestations therefore counts one
emitted defect once per door it is visible through, and the sum is an artifact of how many
entries were boarded -- it rises when you board more modules and nothing changed in the tree.
So this tool reports three quantities and never adds the first across modules:

  manifestations  per module, the existing board unit (rustc_mechanism_classify's grain)
  distinct sites  manifestations deduplicated by SITE IDENTITY across the whole cohort
  breadth         how many module boards a distinct site appears on

A root's weight is (distinct sites, breadth), never summed manifestations.

SITE IDENTITY, declared rather than emergent, and reported at two strictnesses because only one
of them can be wrong quietly:

  strict  (emitted_file, line, col, code, canonical expected/found pair)
  loose   (emitted_file, code, canonical expected/found pair, message)

Strict assumes an emitted file has the same bytes in every closure it is emitted into. That is
the natural assumption and it is exactly the assumption that is unsafe -- a closure-dependent
emit decision (the primitive-representation switch is one that provably exists) can shift line
numbers without changing the defect. Loose survives that shift and over-merges two genuinely
different sites in one file that share a message and a type pair. Publishing both makes the gap
between them a reading: a large strict/loose divergence means emitted line numbers are NOT
stable across closures and the strict figure is the one to discard.

SUBJECT ASSERTION IS PART OF THE JOIN, NOT PART OF THE CALLER'S DILIGENCE. `--rows` and
`--expect-sha` make the cohort refuse unless every board's own row agrees on the ref it was taken
at and that ref is the one the caller declared. A board taken at an unasserted ref is not a weak
measurement; it is a contaminant in a comparison, indistinguishable afterwards from a genuine
per-module difference. The same argument covers the population: the modules present as rows and
the modules present as logs must be the same set, so a board that line-stopped cannot be quietly
absent from a weighting that describes itself as covering the frontier.

ROOT VOCABULARY IS BORROWED, NOT MINTED (DESIGN.md section 3). Cross-code mechanism identity
comes from `rustc_mechanism_classify`; the E0308 15-root vocabulary comes from
`e0308_classify_sites` and is carried as a CODE-LOCAL PROJECTION under its own column name, never
promoted to a cross-code root here. Rows this repository has not established a cross-code
discriminator for stay UNCLASSIFIED. That is the honest majority and it is left visible rather
than filled in from the error code, because a code is a symptom.
"""
import argparse
import collections
import csv
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import e0308_classify_sites as e0308
import rustc_mechanism_classify as mech

TOOLCHAIN = re.compile(r"^/rustc/[0-9a-f]+/")


def canonical_file(path):
    """Toolchain sources carry a rustc hash in their path; two cohort members built by the same
    toolchain agree, but the hash is not a property of anything this repository emitted. It is
    replaced by a stable marker so a cross-cohort join does not silently key on the toolchain."""
    if TOOLCHAIN.match(path):
        return TOOLCHAIN.sub("<toolchain>/", path)
    return path


def module_rows(module, log_path):
    rows = []
    for row in mech.classify(log_path):
        row = dict(row)
        row["module"] = module
        row["file"] = canonical_file(row["file"])
        pair = "%s => %s" % (row["expected"], row["found"]) if row["expected"] or row["found"] else ""
        row["pair"] = pair
        row["strict_site"] = "\t".join((row["file"], row["line"], row["col"], row["code"], pair))
        row["loose_site"] = "\t".join((row["file"], row["code"], pair, row["message"]))
        rows.append(row)
    return rows


def root_of(row):
    """The reported root. UNCLASSIFIED stays UNCLASSIFIED: an error code is not a root, and the
    E0308 projection is a code-local view that this tool refuses to promote."""
    return row["mechanism"]


def weight(rows, key):
    """(distinct sites, breadth histogram) for one site-identity strictness."""
    sites = collections.defaultdict(set)
    for row in rows:
        sites[row[key]].add(row["module"])
    return sites


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("log_dir", help="directory of <module>.cargo.log files, one per board")
    ap.add_argument("out_prefix", help="output path prefix for the TSVs this writes")
    ap.add_argument("--rows", help="the cohort's probe rows TSV (curated_cargo_probe_one 13-column "
                                   "rows); required together with --expect-sha")
    ap.add_argument("--expect-sha", help="the ref every board must declare in its own row")
    args = ap.parse_args()

    if bool(args.rows) != bool(args.expect_sha):
        sys.exit("cross_module_weighting: REFUSED — --rows and --expect-sha are one assertion and "
                 "must be given together; half of it asserts nothing")

    logs = sorted(f for f in os.listdir(args.log_dir) if f.endswith(".cargo.log"))
    if not logs:
        sys.exit("cross_module_weighting: REFUSED — no <module>.cargo.log under %s" % args.log_dir)

    declared = {}
    if args.rows:
        with open(args.rows, encoding="utf-8") as fh:
            for raw in fh:
                fields = raw.rstrip("\n").split("\t")
                if len(fields) < 9 or not fields[0].endswith(".dag"):
                    continue
                declared[os.path.basename(fields[0])[: -len(".dag")]] = fields[8]
        if not declared:
            sys.exit("cross_module_weighting: REFUSED — %s carried no probe rows" % args.rows)
        wrong = {m: sha for m, sha in declared.items() if sha != args.expect_sha}
        if wrong:
            sys.exit("cross_module_weighting: REFUSED — %d board(s) declare a ref other than %s: %s"
                     % (len(wrong), args.expect_sha, wrong))
        logged = {name[: -len(".cargo.log")] for name in logs}
        if logged != set(declared):
            sys.exit("cross_module_weighting: REFUSED — rows and logs describe different cohorts; "
                     "rows-without-log %s, log-without-row %s"
                     % (sorted(set(declared) - logged), sorted(logged - set(declared))))
        print("subject asserted: %d boards, all at %s" % (len(declared), args.expect_sha))

    rows = []
    per_module = collections.OrderedDict()
    for name in logs:
        module = name[: -len(".cargo.log")]
        module_result = module_rows(module, os.path.join(args.log_dir, name))
        per_module[module] = module_result
        rows.extend(module_result)

    columns = ["module", "code", "block", "manifestation", "file", "line", "col", "expected",
               "found", "mechanism", "e0308_candidate_projection", "message"]
    with open(args.out_prefix + "_manifestations.tsv", "w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(fh, fieldnames=columns, extrasaction="ignore",
                                dialect="excel-tab", lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)

    strict = weight(rows, "strict_site")
    loose = weight(rows, "loose_site")

    # Per-root weighting. Manifestations are reported per module and the cohort column is the
    # DISTINCT-SITE count, never their sum.
    root_rows = []
    for root in sorted({root_of(r) for r in rows}):
        sel = [r for r in rows if root_of(r) == root]
        root_strict = {s for s in {r["strict_site"] for r in sel}}
        root_loose = {s for s in {r["loose_site"] for r in sel}}
        breadth = [len(loose[s]) for s in root_loose]
        shared = sum(1 for b in breadth if b > 1)
        root_rows.append({
            "root": root,
            "modules_present": len({r["module"] for r in sel}),
            "manifestations_summed_DO_NOT_RANK": len(sel),
            "distinct_sites_strict": len(root_strict),
            "distinct_sites_loose": len(root_loose),
            "shared_sites_loose": shared,
            "unique_sites_loose": len(root_loose) - shared,
            "max_breadth": max(breadth) if breadth else 0,
            "codes": ",".join(sorted({r["code"] for r in sel})),
        })
    with open(args.out_prefix + "_roots.tsv", "w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(fh, fieldnames=list(root_rows[0].keys()),
                                dialect="excel-tab", lineterminator="\n")
        writer.writeheader()
        writer.writerows(root_rows)

    # Per-module board, so a per-module figure is never recovered by subtraction.
    board_rows = []
    for module, sel in per_module.items():
        by_code = collections.Counter(r["code"] for r in sel)
        board_rows.append({
            "module": module,
            "manifestations": len(sel),
            "distinct_codes": len(by_code),
            "classified": sum(1 for r in sel if r["mechanism"] != "UNCLASSIFIED"),
            "histogram": " ".join("%s:%d" % kv for kv in by_code.most_common()),
        })
    with open(args.out_prefix + "_boards.tsv", "w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(fh, fieldnames=list(board_rows[0].keys()),
                                dialect="excel-tab", lineterminator="\n")
        writer.writeheader()
        writer.writerows(board_rows)

    total = len(rows)
    shared_strict = sum(1 for s, m in strict.items() if len(m) > 1)
    shared_loose = sum(1 for s, m in loose.items() if len(m) > 1)
    print("cohort: %d module boards" % len(per_module))
    print("manifestations (summed across boards, NOT a defect count): %d" % total)
    print("distinct sites: strict %d, loose %d" % (len(strict), len(loose)))
    print("shared sites (breadth > 1): strict %d, loose %d" % (shared_strict, shared_loose))
    if len(per_module) > 1:
        print("shared-floor share of loose distinct sites: %.1f%%"
              % (100.0 * shared_loose / len(loose)))
    print("breadth histogram (loose): %s"
          % dict(sorted(collections.Counter(len(m) for m in loose.values()).items())))
    for row in sorted(root_rows, key=lambda r: -r["distinct_sites_loose"]):
        print("  %-20s modules=%-3d sites_loose=%-5d shared=%-5d max_breadth=%d"
              % (row["root"], row["modules_present"], row["distinct_sites_loose"],
                 row["shared_sites_loose"], row["max_breadth"]))


if __name__ == "__main__":
    main()
