#!/usr/bin/env python3
"""Triage the Acc's conflict diagnostics out of a two-closure gate run.

Why this exists. The Acc turns main's silent first-wins rebind into a reported conflict, so the
head's self-host run carries hard type-mismatch diagnostics that main does not, and hard
diagnostics stop emission (measured: head emitted=0, main emitted=2). The push gate asks for the
per-module count AND a disposition per refusal, so "43" is not enough: each row has to be said to
be one of a small number of things, by a mechanical predicate, from the log text -- not by me
reading twenty of them and eyeballing the rest.

Classes, in order, decided by the text of the refusal itself:

  uninformative-stored-binding
      The binding that is being REBOUND (the "expected" side, which is the binding the first
      argument left behind) is a known fabricated placeholder rather than a type anyone wrote:
      `Product(Unit<>)`, or a `Primitive()` whose name rendered empty. Reporting a conflict there
      blames the call site for an upstream fabrication -- the same family as the RFM row
      error_type_fabricated_as_a_type_by_total_accessors, and the nameless-leaf class. Under a
      refine-don't-fix rule these would not be conflicts at all.

  identical-rendering
      Expected and got render the SAME string, so the authority refused two structurally identical
      printings. A reader sees "expected X, got X", which looks like a compiler bug even when the
      refusal is deliberate (hidden identity: an alias vs its peeled rhs, the
      peel_alias_rhs_once / text-judgment class). These need the fold's own equality to be the
      thing that decides, or the alias question settled -- they must not ship as unexplained.

  genuine-disagreement
      Neither of the above: two different types, first-wins hid it, the Acc shows it. This is the
      class the PR exists to surface.

Usage: triage_conflicts_14152.py LOG_DIR [LOG_DIR2 ...]
Reads head_self-host.log / head_v2-native-cli.log (and main_* for the contrast).
Prints counts per class, per module, and the machine-readable JSON next to each log.
"""
import collections
import json
import os
import re
import sys

# `type mismatch: expected 'type variable A bound as <T1> (first argument)', got '<T2> (later argument)' (module:ln-ln)`
ROW = re.compile(
    r"type mismatch: expected 'type variable (?P<bind>\S+) bound as (?P<exp>.*?)"
    r"(?: \((?:first|later) argument\))?', got '(?P<got>.*?)(?: \((?:first|later) argument\))?' "
    r"(?:\((?P<mod>[^()]+:\d+-\d+)\))?"
)
# Continuation lines in the emitted log carry no module prefix; the module, when present, is in
# trailing parentheses, and long rows wrap, so a row may be split across lines. Join first.
UNIT = re.compile(
    r"^\s*(?:(?P<mod>\S+?\.dag|<kernel:\w+>):(?P<ln>\d+)-(?P<ln2>\d+): )?"
    r"(?:error: )?(?P<body>type mismatch: .*)$"
)

UNINFORMATIVE = ("Product(Unit<>)", "Primitive()", "Product()")


def rows_from(path):
    """Parse refusal rows, joining wrapped lines, and never inventing a module."""
    raw = open(path, errors="replace").read()
    raw = re.sub(r"\x1b\[[0-9;]*m", "", raw)
    # Re-join wrapped rows: a row starts at `error: type mismatch:` and ends at the line whose
    # trailing `)` closes the module paren (or the next row start).
    lines = raw.splitlines()
    out = []
    buf = None
    for line in lines:
        m = UNIT.match(line)
        if m:
            if buf:
                out.append(buf)
            buf = {"mod": m.group("mod") or "", "ln": m.group("ln") or "", "body": m.group("body")}
            continue
        if buf is not None:
            # continuation of the wrapped row
            buf["body"] += line.strip()
    if buf:
        out.append(buf)

    parsed = []
    for r in out:
        m = ROW.search(r["body"])
        if not m:
            continue
        exp = m.group("exp").strip()
        got = m.group("got").strip()
        mod = r["mod"]
        if not mod:
            mm = re.search(r"\(([^()]+:\d+-\d+)\)\s*$", r["body"])
            if mm:
                mod = mm.group(1)
        parsed.append({"module": mod.split(":")[0], "span": mod, "bind": m.group("bind"),
                       "expected": exp, "got": got})
    return parsed


def classify(row):
    if any(u in row["expected"] for u in UNINFORMATIVE):
        return "uninformative-stored-binding"
    if row["expected"] == row["got"]:
        return "identical-rendering"
    return "genuine-disagreement"


def main():
    dirs = sys.argv[1:] or ["/tmp/gate_mergedout"]
    for d in dirs:
        print("=" * 78)
        print("LOG DIR", d)
        for closure in ("self-host", "v2-native-cli"):
            path = os.path.join(d, f"head_{closure}.log")
            if not os.path.exists(path):
                print(f"  [{closure}] no log at {path}")
                continue
            rows = [r for r in rows_from(path) if "bound as" in r["expected"] or r["expected"]]
            for r in rows:
                r["class"] = classify(r)
            main_path = os.path.join(d, f"main_{closure}.log")
            main_rows = rows_from(main_path) if os.path.exists(main_path) else []
            by_class = collections.Counter(r["class"] for r in rows)
            by_mod = collections.Counter(r["module"] for r in rows)
            print(f"\n  CLOSURE {closure}: {len(rows)} conflict rows "
                  f"(main side of the same closure: {len(main_rows)})")
            for k in ("uninformative-stored-binding", "identical-rendering", "genuine-disagreement"):
                print(f"    {k:32s} {by_class.get(k, 0)}")
            print("    per module:")
            for mod, n in by_mod.most_common():
                cls = collections.Counter(r["class"] for r in rows if r["module"] == mod)
                tag = ",".join(f"{k.split('-')[0]}:{v}" for k, v in sorted(cls.items()))
                print(f"      {n:3d}  {mod:48s} [{tag}]")
            out = os.path.join(d, f"triage_{closure.replace('-', '_')}.json")
            json.dump({"closure": closure, "rows": rows, "counts": dict(by_class)},
                      open(out, "w"), indent=1)
            print(f"    -> {out}")
            # Sanity guard: a row that parsed without a module is a parser bug, not a finding.
            nomod = [r for r in rows if not r["module"]]
            if nomod:
                print(f"    WARNING {len(nomod)} rows parsed WITHOUT a module -- parser needs work")
                for r in nomod[:3]:
                    print("      e.g.", r["expected"][:60], "||", r["got"][:60])


if __name__ == "__main__":
    main()
