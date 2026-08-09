#!/usr/bin/env python3
"""Classify a stripped-tree diagnostic log into the residual ledger.

Input:  a `gunbc compile --target dag` log from a fully import-stripped tree,
        plus the tree it was produced from (for the declaration index).
Output: the residual ledger TSV, and the reconciliation totals on stdout.

Two deliberate properties, both of which exist because a ledger reads as
authoritative (DESIGN §5, no fabricated plausible output):

  * Nothing in this script observes the LOADER. It reads diagnostics and
    declarations. So `provider_in_loaded_closure`, `intended_provider` and
    `accepted_binding` are emitted as `unobserved` on every row — whether a
    provider entered the closure, and which declaration a reference actually
    accepted, are exactly the open E/F questions and are not derivable from
    diagnostic text. Filling them would fabricate the measurement.
  * A name that this script cannot find a declaration for is reported as an
    INDEX limitation with a typed sub-reason, never as "the name does not
    exist". The declaration index below is known-partial (see build_index).

Usage:
  classify_residual.py <stripped-diagnostics.log> <tree-root> <out.tsv> \
      [--control-count N]
"""
import collections
import csv
import os
import re
import sys

# Diagnostic-kind → occurrence category. Order matters: the method/field arms
# must be tried before the generic quoted-name fallback, or a method failure on
# an unresolved receiver is misread as an unresolved callee.
KINDS = [
    (r"^function '([^']+)' not found in scope", "callee"),
    (r"^unresolved type '([^']+)'", "type"),
    (r"^undefined variable '([^']+)'", "value"),
    (r"^variant '([^']+)' not found in type", "variant"),
    (r"^method '([^']+)' cannot be resolved", "method"),
    (r"^no field '([^']+)' on type", "field"),
    (r"^field '([^']+)' not found in type", "field"),
    (r"^missing required field '([^']+)' in literal", "record_shape"),
    (r"^type mismatch: expected '([^']+)'", "cascade"),
]
IDENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
LOC = re.compile(r"\(([^()]+?):(\d+)-\d+\)\s*$")


def build_index(root, source_dirs=("dag", "src/v2")):
    """name -> [(module, kind)] and variant tag -> [(module, owning type)].

    KNOWN-PARTIAL, and the ledger says so rather than treating a miss as
    absence. It reads line-anchored declarations and `|`-per-line coproduct
    variants. It does NOT see: inline `type X = A | B` variants, interpreter
    builtins, `std.primitives` PrimitiveContract rows, algebra-template
    operations, or per-target emit vocabulary. DESIGN's determinism open thread
    records why a complete primitive denominator is not assemblable today — the
    roster is forked across five authorities — so this script does not pretend
    to one.
    """
    decl = collections.defaultdict(list)
    variants = collections.defaultdict(list)
    for base in source_dirs:
        for dirpath, _, filenames in os.walk(os.path.join(root, base)):
            for filename in filenames:
                if not filename.endswith(".dag"):
                    continue
                path = os.path.join(dirpath, filename)
                try:
                    text = open(path).read()
                except OSError:
                    continue
                m = re.search(r"^module\s+([A-Za-z0-9_.]+)", text, re.M)
                module = m.group(1) if m else os.path.relpath(path, root)
                owner = None
                for line in text.split("\n"):
                    d = re.match(r"^(fn|func|type|data|service)\s+([A-Za-z_][A-Za-z0-9_]*)", line)
                    if d:
                        decl[d.group(2)].append((module, d.group(1)))
                        owner = d.group(2) if d.group(1) == "type" else None
                        continue
                    v = re.match(r"^\s*\|\s*([A-Z][A-Za-z0-9_]*)", line)
                    if v and owner:
                        variants[v.group(1)].append((module, owner))
    return decl, variants


def parse(line):
    """-> (consumer_file, occurrence_name, occurrence_category)"""
    loc = LOC.search(line)
    consumer = loc.group(1) if loc else "UNKNOWN"
    for pattern, category in KINDS:
        m = re.match(pattern, line)
        if m:
            return consumer, m.group(1), category
    m = re.search(r"'([^']+)'", line)
    return consumer, (m.group(1) if m else ""), "unnamed"


# NAMED, CHECKED DISPOSITIONS — not thresholds.
#
# An earlier revision classified these two buckets by heuristic: "10 or more
# declarations is a convention" and "every candidate module has .fixture. in its
# path is an intentional collision". Both fail OPEN in the direction that
# matters. A genuine accidental fork that happened to reach ten sites would have
# been absorbed into the convention bucket and disappeared from the hygiene
# count; an unrelated duplicate between two fixture modules would have been
# excused by its path. Since the hygiene bucket is being asserted at ZERO, a
# rule whose failure mode is "quietly removes rows from the number under test"
# is the absorbing fallback DESIGN §5 forbids, aimed at this measurement's one
# load-bearing figure.
#
# So each subject is NAMED, and its defining property is CHECKED against the
# tree. A subject that stops satisfying its property does not keep its
# disposition — it lands in `duplicate_unclassified`, which is loud, counted,
# and not zero. Nothing is excused by shape.

CONVENTION_POPULATIONS = {
    # Every extdeps module declares its own authority anchor and model scope.
    # The property is not "there are many" — it is one per module, all under
    # one prefix, which is what makes it a convention rather than a fork.
    "extdeps_external_authority_anchor": {"module_prefix": "extdeps."},
    "extdeps_model_scope": {"module_prefix": "extdeps."},
}

INTENTIONAL_AMBIGUITIES = {
    # Fixtures that exist SO THAT two declarations collide; renaming them
    # destroys the test subject. Pinned to the EXACT module pair, so a third
    # declaration of the same name anywhere else is not excused by these rows.
    "CensusProbeSubject": frozenset({
        "test.fixture.record_construction_census.homonym",
        "test.fixture.record_construction_census.specimens",
    }),
    "SharedBareArm": frozenset({
        "test.fixture.decl_facts_reflection.ambiguous_shared_a",
        "test.fixture.decl_facts_reflection.ambiguous_shared_b",
    }),
}


def convention_holds(name, candidates):
    """One declaration per module, every module under the declared prefix."""
    spec = CONVENTION_POPULATIONS.get(name)
    if not spec or not candidates:
        return False
    mods = list(candidates)
    if len(mods) != len(set(mods)):
        return False                      # two in one module is a fork, not a convention
    return all(m.startswith(spec["module_prefix"]) for m in mods)


def intentional_ambiguity_holds(name, candidates):
    """The candidate set is EXACTLY the pinned pair — no more, no fewer."""
    pinned = INTENTIONAL_AMBIGUITIES.get(name)
    return bool(pinned) and frozenset(candidates) == pinned


def disposition(name, category, decl_count, is_variant, line, candidates=()):
    """The typed disposition. Nothing here claims a loader observation."""
    # A duplicate-named FIELD or method is not a declaration-naming defect: the
    # first failure is on an unresolved or wrong receiver, and renaming the
    # declaration would not touch it. Category wins over multiplicity here.
    if category in ("field", "method", "record_shape"):
        return {"field": "field_on_unresolved_or_wrong_type",
                "method": "method_on_unresolved_receiver",
                "record_shape": "record_shape_cascade"}[category]
    if decl_count > 1 and name in CONVENTION_POPULATIONS:
        return ("per_module_convention_population" if convention_holds(name, candidates)
                else "duplicate_unclassified")
    if decl_count > 1 and name in INTENTIONAL_AMBIGUITIES:
        return ("intentional_ambiguity_fixture" if intentional_ambiguity_holds(name, candidates)
                else "duplicate_unclassified")
    if decl_count > 1:
        return "corpus_hygiene"
    if is_variant:
        # The nullary-variant mechanism was REFUTED by execution (§5 of the
        # cascade diagnosis). These rows carry a re-proof obligation, not a
        # known cause.
        return "variant_mechanism_unobserved"
    if category == "cascade" or not IDENT.match(name or ""):
        return "cascade"
    if decl_count == 1:
        # One indexed declaration + an unresolved reference does NOT establish
        # that the provider was absent from the closure. It is consistent with
        # provider-absent, provider-present-but-unadmitted, wrong occurrence
        # category on a legacy path, a fabrication arm rewriting the error, or
        # the index having matched a declaration that is not the intended
        # binding. Promotion to clause E requires loader instrumentation.
        return "unique_decl_unresolved_mechanism_unobserved"
    # decl_count == 0 — split the mixed bucket by first failing shape.
    if category == "method":
        return "method_on_unresolved_receiver"
    if category == "field":
        return "field_on_unresolved_or_wrong_type"
    if category == "record_shape":
        return "record_shape_cascade"
    if category == "callee":
        return "ordinary_callee_unindexed"
    if name and name[0].isupper():
        return "variant_owner_unindexed"
    return "unindexed_symbol_candidate"


def main(argv):
    log_path, root, out_path = argv[1], argv[2], argv[3]
    control = 0
    if "--control-count" in argv:
        control = int(argv[argv.index("--control-count") + 1])

    decl, variants = build_index(root)
    lines = [l.rstrip("\n") for l in open(log_path) if l.strip()]

    # The header line and the control's own pre-existing diagnostics are not
    # strip-attributable; the caller states the control count so the ledger
    # identity is checkable rather than assumed.
    header = [i for i, l in enumerate(lines) if "hard diagnostic" in l]
    total = None
    if header:
        m = re.search(r"produced (\d+) hard diagnostic", lines[header[0]])
        total = int(m.group(1)) if m else None
        lines = lines[header[0] + 1:]
    lines = [l for l in lines if not l.startswith("source annotation sits inside")]

    rows, seen = [], collections.Counter()
    for line in lines:
        consumer, name, category = parse(line)
        sites = decl.get(name, [])
        vsites = variants.get(name, [])
        n = len(sites)
        candidates = sorted({m for m, _ in sites}) or sorted({m for m, _ in vsites})
        rows.append(dict(
            consumer_file=consumer,
            occurrence_name=name,
            occurrence_category=category,
            candidate_provider_modules="|".join(candidates),
            candidate_count=len(candidates),
            intended_provider="unobserved",
            accepted_binding="unobserved",
            provider_in_loaded_closure="unobserved",
            binding_outcome=("suspected_fabricated" if "<anon>" in line
                             else "ambiguous_candidates" if n > 1
                             else "unresolved" if category in ("type", "value", "callee", "variant")
                             else "n/a"),
            first_root_diagnostic=re.sub(r"\s*\([^()]*:\d+-\d+\)\s*$", "", line)[:200],
            disposition=disposition(name, category, n, bool(vsites) and n == 0, line,
                                    candidates=candidates),
        ))
        seen[(consumer, name)] += 1
    for r in rows:
        r["downstream_diagnostic_count"] = seen[(r["consumer_file"], r["occurrence_name"])] - 1

    cols = ["consumer_file", "occurrence_name", "occurrence_category",
            "candidate_provider_modules", "candidate_count", "intended_provider",
            "accepted_binding", "provider_in_loaded_closure", "binding_outcome",
            "first_root_diagnostic", "downstream_diagnostic_count", "disposition"]
    with open(out_path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=cols, delimiter="\t",
                           extrasaction="ignore", lineterminator="\n")
        w.writeheader()
        for r in sorted(rows, key=lambda r: (r["disposition"], r["candidate_provider_modules"],
                                             r["consumer_file"], r["occurrence_name"])):
            w.writerow(r)

    counts = collections.Counter(r["disposition"] for r in rows)
    print(f"ledger rows: {len(rows)}")
    for k, v in counts.most_common():
        print(f"  {k:42s} {v}")
    print(f"  {'SUM':42s} {sum(counts.values())}")
    if total is not None:
        print(f"\nreconciliation: stripped_hard={total} = control_hard={control} "
              f"+ attributable={total - control}")
        print(f"ledger rows {len(rows)} vs attributable {total - control}: "
              f"{'OK' if len(rows) == total - control else 'MISMATCH'}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
