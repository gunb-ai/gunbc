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

THE ROSTER READ LIVES HERE, because the runner that used to hold it is gone. `--roster` points at
`dag/tools/self_host_module_behavioral_transport_roster.dag` and the join refuses unless the
cohort's modules are exactly the modules that authority declares -- no missing member, no
non-member. This is the defence against the failure that motivated the whole lane: enumerating the
frontier by filename glob, by grep on a field name, or by hand each produced a population close
enough to pass a glance and wrong in the denominator (17 where the roster says 16; a glob reported
as roster membership; a zero where the true answer was five). A wrong denominator does its damage
in a COMPARISON, where it is afterwards indistinguishable from a real per-module difference.
Holding the check here rather than in a sweep runner also puts it where it cannot be skipped: a
runner is only consulted by whoever chooses to run it, while every published weighting passes
through this join.

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

# ---- cross-code mechanisms established by this cohort ----------------------
# TWO ADDITIONS TO THE CROSS-CODE VOCABULARY, AND WHY EACH IS A ROOT RATHER THAN A CODE.
# Both are visible only from a cohort: each spans codes whose per-code partitions would rank it
# twice at half size, and each is concentrated in emitted std files that sit in every closure.
#
# PRIMITIVE_REPR_FORK. DESIGN.md's declared open thread and the mechanism eager-deer-389
# root-caused: `v1.compiler.04_infer` `rust_corpus_repr` picks HostNative vs FaithfulFreeMonoid
# from a path-substring test over the closure's source keys, so a pure-v2 closure renders the
# numeric tower as the modeled carrier while its neighbours are native. rustc then reports the
# same emitted decision as a type mismatch (E0308) where a value crosses the seam and as an
# unsupported operator (E0369) where an operator is applied to the modeled carrier. The
# discriminator requires a modeled numeric carrier on one side and a native integer on the
# other -- never the mere presence of the word Nat, which also appears in unrelated mismatches.
#
# MAP_CARRIER_FORK. One map concept realized two ways: a modeled `PartialFunction`/`PointwisePower`
# in type position against `HashMap`/`BTreeMap` in the seed's own surface. It reaches rustc as a
# mismatch (E0308) where a map value crosses, and as a missing struct field (E0560) where the
# emitter writes a record literal whose field names belong to the OTHER realization. The E0560
# half is the load-bearing part of the join: read per code it looks like an unrelated
# "struct has no field" family, and no E0308 repair would close it.
#
# WHAT IS DELIBERATELY NOT ADDED. E0004 non-exhaustive patterns, E0425 unresolved names and
# E0432 unresolved imports are each large here and each plausibly one root, and none of them has
# a discriminator this cohort establishes. They stay UNCLASSIFIED. Filling them in from their
# code is exactly the move this file exists to refuse.
MODELED_NUMERIC = {"Nat", "Int", "CommutativeSemiring", "GroupCompletion", "Magnitude"}
MAP_MODELED = {"PartialFunction", "PointwisePower"}
MAP_NATIVE = {"HashMap", "BTreeMap", "OrdMap"}

E0369_MODELED = re.compile(r"`(?:&)?Rc<(?:[a-z0-9_]+::)*(Nat|Int|CommutativeSemiring<[^`]*|GroupCompletion<[^`]*)>`")
E0560_MAP = re.compile(r"^struct `(?:&)?(?:Rc<)?(PartialFunction|PointwisePower)<")


def _heads(text):
    """Nominal heads of a type spelling, Rc wrappers erased, module paths dropped."""
    bare = e0308.strip_rc(e0308.normalize(text))[0]
    return e0308.head(bare)


def primitive_repr_fork(code, message, expected, found):
    if code == "E0308":
        if not (expected and found):
            return False
        he, hf = _heads(expected), _heads(found)
        modeled = {h for h in (he, hf) if h in MODELED_NUMERIC}
        native = {h for h in (he, hf) if h in e0308.NUMERIC_NATIVE}
        return bool(modeled) and bool(native)
    if code == "E0369":
        # An operator refused ON the modeled carrier. The native side is not required to appear:
        # `binary operation `<` cannot be applied to type `Rc<Nat>`` names only the receiver, and
        # excluding it would drop the operator half of the very fork being counted.
        return bool(E0369_MODELED.search(message))
    return False


def map_carrier_fork(code, message, expected, found):
    if code == "E0560":
        return bool(E0560_MAP.match(message))
    if code == "E0308":
        if not (expected and found):
            return False
        he, hf = _heads(expected), _heads(found)
        return ({he, hf} & MAP_MODELED) and ({he, hf} & MAP_NATIVE)
    return False


# ---- instrument attribution, which is NOT a root -----------------------------
# A board taken with a lane shim installs that lane's hand-written lib.rs OVER the assembled one,
# and that lib.rs declares only the modules its own lane needed. Every module of the closure it
# does not declare then refuses as an unresolved crate-root path. curated_cargo_probe_one.sh names
# this the FALSE-RED hazard; measured across this cohort it is not a hazard in principle but the
# single largest population difference between shim-bearing and shim-free boards -- all 60 E0432
# and all 16 E0608 manifestations here sit on the four shim boards, and 41 of 57 E0433 do.
#
# It is attributed rather than left UNCLASSIFIED because the two are different claims: UNCLASSIFIED
# says no root is established, this says the population is a property of how the board was taken.
# Ranking it as a repair root would staff a defect that does not exist outside the probe; dropping
# it silently would make four boards look smaller than they are. It is therefore its own labelled
# row, excluded from root ranking by name and never by a filter someone has to remember.
SHIM_UNRESOLVED = re.compile(
    r"^(?:unresolved import `crate::[a-z0-9_]+"
    r"|cannot find (?:[a-z0-9_]+|`[a-z0-9_]+`) in `crate`"
    r"|cannot find `[a-z0-9_]+` in `crate`)")


def shim_closure_gap(code, message, shim_board):
    if not shim_board or code not in ("E0432", "E0433", "E0608"):
        return False
    return bool(SHIM_UNRESOLVED.match(message))


CROSS_CODE = (
    ("PRIMITIVE_REPR_FORK", primitive_repr_fork),
    ("MAP_CARRIER_FORK", map_carrier_fork),
)


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


def root_of(row, shim_boards=frozenset()):
    """The reported root. UNCLASSIFIED stays UNCLASSIFIED: an error code is not a root, and the
    E0308 projection is a code-local view that this tool refuses to promote.

    rustc_mechanism_classify's ABSENT_CLONE_BOUND keeps precedence: it is the established
    mechanism and its discriminator is the strictest here (it requires rustc's own explanation
    text, not a type shape), so a row it claims is never re-labelled by a shape test."""
    if row["mechanism"] != "UNCLASSIFIED":
        return row["mechanism"]
    if shim_closure_gap(row["code"], row["message"], row["module"] in shim_boards):
        return "SHIM_CLOSURE_GAP_instrument"
    for name, decide in CROSS_CODE:
        if decide(row["code"], row["message"], row["expected"], row["found"]):
            return name
    return "UNCLASSIFIED"


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
    ap.add_argument("--roster", help="path to the frontier roster authority; when given, the "
                                     "cohort must be exactly the modules it declares")
    ap.add_argument("--shim-board", action="append", default=[], metavar="MODULE",
                    help="a board taken with a lane shim_lib_rel installed. Unresolved crate-root "
                         "paths on such a board are attributed to the shim's undeclared closure, "
                         "not counted as an emitter root.")
    ap.add_argument("--declared-absent", action="append", default=[], metavar="MODULE=REASON",
                    help="a roster module that produced a row but no log, with the reason it did "
                         "not board. Exclusion must be declared and reasoned; the population "
                         "check refuses an undeclared gap.")
    args = ap.parse_args()

    if bool(args.rows) != bool(args.expect_sha):
        sys.exit("cross_module_weighting: REFUSED — --rows and --expect-sha are one assertion and "
                 "must be given together; half of it asserts nothing")

    def roster_membership(path):
        """Module paths declared by the roster, read from the authority rather than from a glob."""
        declared_rows, current = [], None
        with open(path, encoding="utf-8") as fh:
            for raw in fh:
                literal = re.search(r'"([^"]*)"', raw)
                if "module_path:" in raw and literal:
                    current = literal.group(1)
                elif "shim_lib_rel:" in raw and literal and current is not None:
                    declared_rows.append(current)
                    current = None
        return declared_rows

    logs = sorted(f for f in os.listdir(args.log_dir) if f.endswith(".cargo.log"))
    if not logs:
        sys.exit("cross_module_weighting: REFUSED — no <module>.cargo.log under %s" % args.log_dir)

    declared, absent = {}, {}
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
        for spec in args.declared_absent:
            module, sep, reason = spec.partition("=")
            if not sep or not reason.strip():
                sys.exit("cross_module_weighting: REFUSED — --declared-absent needs MODULE=REASON; "
                         "an exclusion with no reason is a silent gap wearing a flag: %r" % spec)
            if module not in declared:
                sys.exit("cross_module_weighting: REFUSED — %s is declared absent but produced no "
                         "probe row either, so nothing establishes it was attempted" % module)
            absent[module] = reason.strip()
        for module, reason in sorted(absent.items()):
            print("declared absent: %s — %s" % (module, reason))
            declared.pop(module)
        logged = {name[: -len(".cargo.log")] for name in logs}
        if logged != set(declared):
            sys.exit("cross_module_weighting: REFUSED — rows and logs describe different cohorts; "
                     "rows-without-log %s, log-without-row %s"
                     % (sorted(set(declared) - logged), sorted(logged - set(declared))))
        print("subject asserted: %d boards, all at %s" % (len(declared), args.expect_sha))

    if args.roster:
        if not args.rows:
            sys.exit("cross_module_weighting: REFUSED — --roster checks the cohort against the "
                     "authority's membership, which needs --rows to know what was attempted")
        membership = roster_membership(args.roster)
        if not membership:
            sys.exit("cross_module_weighting: REFUSED — roster parse produced zero rows; the "
                     "authority's shape changed and this check would pass vacuously")
        expected = {os.path.basename(m)[: -len(".dag")] for m in membership}
        attempted = set(declared) | set(absent)
        if attempted != expected:
            sys.exit("cross_module_weighting: REFUSED — cohort is not the roster's membership; "
                     "declared-but-not-boarded %s, boarded-but-not-declared %s"
                     % (sorted(expected - attempted), sorted(attempted - expected)))
        print("roster membership asserted: %d modules from %s" % (len(expected), args.roster))

    rows = []
    per_module = collections.OrderedDict()
    for name in logs:
        module = name[: -len(".cargo.log")]
        module_result = module_rows(module, os.path.join(args.log_dir, name))
        per_module[module] = module_result
        rows.extend(module_result)

    shim_boards = frozenset(args.shim_board)
    for row in rows:
        row["root"] = root_of(row, shim_boards)

    columns = ["module", "root", "code", "block", "manifestation", "file", "line", "col", "expected",
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
    for root in sorted({r["root"] for r in rows}):
        sel = [r for r in rows if r["root"] == root]
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
            "shim_board": module in shim_boards,
            "classified": sum(1 for r in sel if r["root"] != "UNCLASSIFIED"),
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
