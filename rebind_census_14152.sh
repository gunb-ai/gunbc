#!/usr/bin/env bash
# The rebind census instrument (PR #14152, stern-bat-401 msg_a5591a7a: "make the census TOTAL ...
# commit the parser and its output ... so the partition can be re-derived, §6 name the
# instrument"). Committed as .sh because .gitignore keeps *.py outside the committed
# substrate; the classifier kernel is embedded below verbatim.
#
# USAGE: tools/rebind_census.sh <artifact.log> [<artifact2.log> ...]   # prints the TOTAL
#          partition (every row exactly one class, UNCLASSIFIED explicit) and the rows NO
#          planned change removes, by identity.
#
# See the header inside the kernel for the class definitions and the predicted change.
set -uo pipefail
[[ $# -ge 1 ]] || { echo "usage: $0 <artifact.log> ..." >&2; exit 2; }
python3 - "$@" <<'PY'
"""The rebind census instrument (PR #14152, stern-bat-401 msg_a5591a7a).

NAME THE INSTRUMENT (DESIGN.md §6): this file is the ONLY authority for how the v1
unify_generics rebind refusals are partitioned. Every claim in the PR body and in the
RFM row cites a partition produced by this file against a named artifact, so any reader
can re-derive it:

    tools/rebind_census.sh <artifact.log> [<more.log>] [--json out.json]

INPUT is a CI artifact (or any log) containing the fold's refusal lines, emitted by
v1.compiler.infer unify_generics (src/v1/04_infer.dag) and its seed mirror
src/v1/stage0/src/v1_compiler_infer.rs:

    type variable <NAME> bound as '<FORMAL>' (first argument), got '<ACTUAL>' (later argument)' (<FILE>:<START>-<END>)

The '(first|1st|2nd...)' spelling varies with call position, so both are accepted.

TOTAL PARTITION (§6, parent ruling): each row gets EXACTLY ONE class, decided by the
ordered DECISION list below; the last class is UNCLASSIFIED, and a non-empty UNCLASSIFIED
bucket is reported by identity rather than hidden. Classes are decided from a parsed
shape tree (head + children), never from a substring heuristic, so 'String' as a head is
not confused with 'String' inside a name.

Each class carries the ONE planned change predicted to remove it, or NONE:
  RELATION  -- v1.compiler.infer generic_types_unify (the fold's own unification
               relation: alias/decl unfold, the String=FreeMonoid<Char> text pair, and
               an unapplied head against its application).
  FORMAL(c) -- v1.compiler.infer binding_is_unsettled_formal_annotation: the stored
               first binding still mentions one of the callee's own generics, so it is
               the callee's written formal, not a settled binding.
  X1        -- v1.compiler.types type_heads_agree Absent arms: two structurally
               identical containers whose head is not a declared carrier alias and whose
               DeclarationRef is absent on either side print identically and refuse.
               (Separate PR, parent-approved.)
  X2        -- v1.compiler.infer structured_application_site_type_mismatch_with_peeled
               is name-only and defers to the resolved type (separate PR, approved).
  NONE      -- no planned change removes it. These are the rows where the next surprise
               is, and they are reported explicitly.
"""

import argparse
import collections
import json
import re
import sys

REFUSAL = re.compile(
    r"expected 'type variable (\S+) bound as (.*?) \((?:first|\d+(?:st|nd|rd|th)) argument\)',"
    r" got '(.*?) \((?:later|\d+(?:st|nd|rd|th)) argument\)' \(([^)]*?)\)"
)

# A type parameter is spelled as a single uppercase letter in this corpus (K, V, T, A,
# U, P, Q ...). This is a fact about the PRINTED shape, tested on a leaf's own name, not
# a substring of the whole shape.
GENERIC_LEAF = re.compile(r"^[A-Z]$")

TEXT_HEADS = {"String"}
FREE_MONOID_HEAD = "FreeMonoid"
CHAR_LEAF = "Char"


class Shape:
    """A printed type shape: a head plus ordered child shapes."""

    def __init__(self, head, children):
        self.head = head
        self.children = tuple(children)

    def leaves(self):
        if not self.children:
            yield self
        else:
            for c in self.children:
                yield from c.leaves()

    def is_unapplied_head(self):
        """A head with no arguments at all: 'Coproduct(FreeMonoid)' has children, so an
        applied form is NOT unapplied; 'Unit<>' parses to zero children and counts."""
        return len(self.children) == 0

    def __repr__(self):
        if not self.children:
            return self.head
        return "%s(%s)" % (self.head, ",".join(repr(c) for c in self.children))


def parse_shape(text):
    """Parse a printed shape. Tolerant: anything unparseable raises, and the caller puts
    the row in MALFORMED_SHAPE rather than guessing a class."""
    text = text.strip()
    shape, pos = _parse(text, 0)
    pos = _skip_ws(text, pos)
    if pos != len(text):
        raise ValueError("trailing text at %d in %r" % (pos, text))
    return shape


def _skip_ws(t, i):
    while i < len(t) and t[i] == " ":
        i += 1
    return i


def _parse(t, i):
    i = _skip_ws(t, i)
    start = i
    while i < len(t) and (t[i].isalnum() or t[i] in "._-"):
        i += 1
    head = t[start:i]
    children = []
    i = _skip_ws(t, i)
    if i < len(t) and t[i] in "(<":
        closer = ")" if t[i] == "(" else ">"
        i += 1
        children = []
        i = _skip_ws(t, i)
        if i < len(t) and t[i] == closer:  # Unit<> / Map() : zero arguments
            return Shape(head, children), i + 1
        while True:
            child, i = _parse(t, i)
            children.append(child)
            i = _skip_ws(t, i)
            if i < len(t) and t[i] == ",":
                i += 1
                continue
            if i < len(t) and t[i] == closer:
                return Shape(head, children), i + 1
            raise ValueError("expected %r at %d in %r" % (closer, i, t))
    return Shape(head, children), i


def has_empty_leaf(shape):
    """A leaf with NO name: node_type_shape prints a leaf as concat("Primitive(", name,
    ")") (v1.compiler.types), so a printed 'Primitive()' is a Primitive node whose name is
    EMPTY -- that is the kernel Map template, Map(Primitive(),Primitive()). Encoding:
    'Primitive' with zero children (a named one, Primitive(K), parses to a child)."""
    return any(
        l.head == "Primitive" and len(l.children) == 0 for l in shape.leaves()
    )


def has_generic_leaf(shape):
    return any(GENERIC_LEAF.match(l.head) for l in shape.leaves())


def is_text_pair(f, a):
    """One side is the text carrier, the other the free monoid over Char. DESIGN.md §4
    grounds the text reading; the element leaf must be Char on the monoid side."""
    pairs = [(f, a), (a, f)]
    for text_side, monoid_side in pairs:
        if text_side.head in TEXT_HEADS and monoid_side.head == FREE_MONOID_HEAD:
            if len(monoid_side.children) == 1 and monoid_side.children[0].head == CHAR_LEAF:
                return True
        # also under a common wrapper: Node(String<...>) vs Node(FreeMonoid<Char-ish>)
        if (
            len(text_side.children) == 1
            and len(monoid_side.children) == 1
            and is_text_pair(text_side.children[0], monoid_side.children[0])
        ):
            return True
    return False


DECISION = [
    (
        "SAME_TEXT",
        lambda f, a, s: f is not None and a is not None and repr(f) == repr(a),
        "X1",
        "identical printed shapes refuse: the Absent arms of v1.compiler.types type_heads_agree",
    ),
    (
        "TEXT_PAIR",
        lambda f, a, s: f is not None
        and a is not None
        and is_text_pair(f, a)
        and repr(f) != repr(a),
        "RELATION",
        "String = FreeMonoid<Char> (DESIGN.md §4 text ruling) via generic_types_are_the_text_pair",
    ),
    (
        "KERNEL_EMPTY_LEAF",
        lambda f, a, s: (f is not None and has_empty_leaf(f))
        or (a is not None and has_empty_leaf(a)),
        "NONE",
        "kernel template with empty leaf names; no planned change touches it (parent msg_a5591a7a)",
    ),
    (
        "FORMAL_VS_CONCRETE",
        lambda f, a, s: (f is not None and a is not None and (
            (has_generic_leaf(f) and not has_generic_leaf(a))
            or (has_generic_leaf(a) and not has_generic_leaf(f))
        )),
        "FORMAL(c)",
        "the stored first binding still mentions a callee generic (binding_is_unsettled_formal_annotation)",
    ),
    (
        "SAME_HEAD_ONE_UNAPPLIED",
        lambda f, a, s: f is not None
        and a is not None
        and f.head == a.head
        and repr(f) != repr(a)
        and (f.is_unapplied_head() or a.is_unapplied_head()),
        "RELATION",
        "unapplied head against its application (generic_head_matches_application)",
    ),
    (
        "SAME_HEAD_BOTH_APPLIED",
        lambda f, a, s: f is not None
        and a is not None
        and f.head == a.head
        and repr(f) != repr(a)
        and not f.is_unapplied_head()
        and not a.is_unapplied_head(),
        "NONE",
        "same carrier, different arguments: needs per-row reading (true positives live here)",
    ),
    (
        "DIFF_HEAD_UNFOLD_CANDIDATE",
        lambda f, a, s: f is not None
        and a is not None
        and f.head != a.head
        and len(f.children) == len(a.children)
        and all(
            repr(x) == repr(y) or x.head == y.head
            for x, y in zip(f.children, a.children)
        ),
        "RELATION",
        "heads differ but children agree: alias/declaration unfold (generic_type_unify_unfold)",
    ),
    (
        "DIFF_HEAD_NESTED_TEXT",
        lambda f, a, s: f is not None and a is not None and f.head != a.head and is_text_pair(f, a),
        "RELATION",
        "text carrier against free monoid under a wrapper",
    ),
    (
        "DIFF_HEAD_DISJOINT",
        lambda f, a, s: f is not None and a is not None and f.head != a.head,
        "NONE",
        "different carriers, no planned change removes it; per-row reading required",
    ),
    (
        "MALFORMED_SHAPE",
        lambda f, a, s: f is None or a is None,
        "NONE",
        "the printed shape did not parse; classified by identity, never guessed",
    ),
]


def classify(row):
    f, a = row["fshape"], row["ashape"]
    for name, pred, predicted_by, why in DECISION:
        try:
            if pred(f, a, row):
                return name, predicted_by, why
        except Exception:  # a predicate that cannot decide must not guess a class
            continue
    return "UNCLASSIFIED", "NONE", "no predicate in the decision list decided this row"


def census(paths):
    rows = []
    for p in paths:
        with open(p, errors="replace") as fh:
            for line in fh:
                for m in REFUSAL.finditer(line):
                    var, formal, actual, span = m.groups()
                    r = {
                        "variable": var,
                        "formal": formal.strip(),
                        "actual": actual.strip(),
                        "span": span.strip(),
                        "artifact": p,
                    }
                    try:
                        r["fshape"] = parse_shape(r["formal"])
                    except Exception:
                        r["fshape"] = None
                    try:
                        r["ashape"] = parse_shape(r["actual"])
                    except Exception:
                        r["ashape"] = None
                    rows.append(r)
    for r in rows:
        r["class"], r["predicted_by"], r["why"] = classify(r)
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("artifacts", nargs="+")
    ap.add_argument("--json")
    args = ap.parse_args()
    rows = census(args.artifacts)
    print("instrument: tools/rebind_census.sh")
    print("artifacts: %s" % ", ".join(args.artifacts))
    print("rows parsed: %d" % len(rows))
    print("distinct (formal,actual,span): %d" % len(set((r["formal"], r["actual"], r["span"]) for r in rows)))
    by_class = collections.Counter(r["class"] for r in rows)
    total = sum(by_class.values())
    print("\nTOTAL PARTITION (every row exactly one class; sum must equal rows): %d/%d" % (total, len(rows)))
    for cls, n in by_class.most_common():
        pred = next(p for c, _, p, _ in DECISION if c == cls) if cls in dict(
            (c, p) for c, _, p, _ in DECISION) else "NONE"
        print("  %-26s %4d   predicted removed by %s" % (cls, n, pred))
    assert total == len(rows), "partition is not total"
    print("\nPREDICTION SUMMARY (rows per predicted change):")
    pred_counts = collections.Counter(r["predicted_by"] for r in rows)
    for p, n in pred_counts.most_common():
        print("  %-10s %4d" % (p, n))
    print("\nROWS NO PLANNED CHANGE REMOVES, by identity:")
    for (c, f, a, s), n in collections.Counter(
        (r["class"], r["formal"][:70], r["actual"][:70], r["span"])
        for r in rows
        if r["predicted_by"] == "NONE"
    ).most_common():
        print("  x%d %s\n      F=%s\n      L=%s\n      @%s" % (n, c, f, a, s))
    if args.json:
        with open(args.json, "w") as fh:
            json.dump(
                [
                    {k: (repr(r[k]) if k in ("fshape", "ashape") else r[k]) for k in
                     ("variable", "formal", "actual", "span", "artifact", "class", "predicted_by", "why")}
                    for r in rows
                ],
                fh,
                indent=1,
            )
        print("\nwrote %s" % args.json)
    return 0


if __name__ == "__main__":
    sys.exit(main())
PY
