#!/usr/bin/env python3
"""Generate the "what the 43 refusals actually are" section from the triage JSON.

Why generated. stern-bat-401's push gate demands a disposition per refusal, and a table typed by
hand is exactly where this lane has been caught before: a number transcribed from a log that was
itself a summary line. This reads only the JSON that triage_conflicts_14152.py wrote beside the
logs, and refuses to print a section if a side is missing, so an unmeasured closure cannot appear
as a zero.

It also refuses to over-claim: the classes are defined by the RENDERING of the stored binding, and
the section says so, because "the rebound side prints Product(Unit<>)" is evidence about what the
reader sees and about who is being blamed, not a proof of the producer. The producer of those
placeholders is recorded as unidentified in the RFM row, and this section points there rather than
asserting a cause it has not established.
"""
import collections
import json
import os
import sys

CLASSES = (
    ("genuine-disagreement",
     "genuine disagreement",
     "two different types were bound to one formal; main's first-wins accepted the call on the "
     "first one alone. This is the class the PR exists to surface, and the class the new RFM row "
     "files."),
    ("uninformative-stored-binding",
     "the binding being rebound renders as a fabricated placeholder",
     "`Product(Unit<>)` or an empty `Primitive()`. Reporting a conflict here blames the call site "
     "for an upstream fabrication: the slot was never a type anyone wrote. Producer unidentified "
     "(recorded as UNATTRIBUTED in "
     "`gunbc.recurring_failure_mode.error_type_fabricated_as_a_type_by_total_accessors`), and "
     "these rows are what the pre-registered refine-don't-fix predicate now covers -- as a "
     "SEPARATE change, so the count cannot fall for the wrong reason and leave the report untested."),
    ("identical-rendering",
     "expected and got render IDENTICALLY",
     "the reader sees `expected X, got X`, which is indistinguishable at the citing end from a "
     "broken comparator even when the refusal is deliberate (hidden identity: an alias against its "
     "peeled rhs, the peel_alias_rhs_once / text-judgment class). Shipped as a named class, not as "
     "an unexplained refusal."),
)


def load(d, closure):
    p = os.path.join(d, "triage_%s.json" % closure.replace("-", "_"))
    if not os.path.exists(p):
        return None
    return json.load(open(p))


def main():
    d = sys.argv[1] if len(sys.argv) > 1 else "/tmp/gate_mergedout"
    out = sys.argv[2] if len(sys.argv) > 2 else None
    L = []
    L.append("### What the refusals actually are, disposed by class")
    L.append("")
    L.append("A count is not a disposition, and 43 rows are too many to read twenty of and eyeball "
             "the rest. `triage_conflicts_14152.py` (on the instruments ref) classifies every row "
             "by a mechanical predicate over the **rendering of the stored binding**, which is what "
             "the diagnostic itself shows a reader, and writes JSON beside each log; this table is "
             "generated from that JSON.")
    L.append("")
    missing = []
    for closure in ("self-host", "v2-native-cli"):
        j = load(d, closure)
        if j is None:
            missing.append(closure)
            continue
        n = len(j["rows"])
        c = j["counts"]
        L.append(f"**{closure}** — {n} conflict rows")
        L.append("")
        L.append("| class | n | what it means |")
        L.append("|---|---|---|")
        for key, label, meaning in CLASSES:
            L.append(f"| {label} | {c.get(key, 0)} | {meaning} |")
        L.append("")
        mods = {}
        for r in j["rows"]:
            mods.setdefault(r["module"], []).append(r["class"])
        L.append("| module | rows | genuine | placeholder-bound | identical-render |")
        L.append("|---|---|---|---|---|")
        for mod in sorted(mods, key=lambda m: (-len(mods[m]), m)):
            cls = mods[mod]
            g = sum(1 for x in cls if x == "genuine-disagreement")
            u = sum(1 for x in cls if x == "uninformative-stored-binding")
            i = sum(1 for x in cls if x == "identical-rendering")
            L.append(f"| `{mod}` | {len(cls)} | {g} | {u} | {i} |")
        L.append("")
    if missing:
        L.append(f"**MEASUREMENT MISSING for: {', '.join(missing)}.** No triage JSON for that "
                 "closure, so no claim about it is made here; the counts above cover only what ran.")
    L.append("")
    # Even the summary sentence takes its numbers from the artifact: a count typed into prose is how
    # a stale figure outlives the run that produced it.
    tot = collections.Counter()
    for closure in ("self-host", "v2-native-cli"):
        j = load(d, closure)
        if j:
            tot.update(j["counts"])
    L.append(
        "The split matters for what this PR can and cannot fix. The "
        f"{tot.get('genuine-disagreement', 0)} genuine rows across both closures are this PR's "
        f"subject. The {tot.get('uninformative-stored-binding', 0)} placeholder-bound rows are an "
        "upstream typing defect wearing a call-site symptom, and the "
        f"{tot.get('identical-rendering', 0)} identical-rendering rows are an alias/equality "
        "question -- both named and left open rather than folded into this PR or silenced.")
    text = "\n".join(L) + "\n"
    if out:
        open(out, "w").write(text)
        print("wrote", out, len(text), "bytes")
    else:
        sys.stdout.write(text)


if __name__ == "__main__":
    main()
