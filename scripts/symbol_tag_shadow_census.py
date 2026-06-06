#!/usr/bin/env python3
"""Census of the "Symbol-tag shadow taxonomy" anti-pattern.

A coproduct `type X = A | B | ...` whose arm-set is mirrored by a parallel set of
self-named `data x_arm_a: Symbol = x_arm_a` constants, bridged by a hand-written
`fn ..._discriminant(v: X) -> Symbol { match v { A{..} => x_arm_a ... } }`, and
often pinned by a roster test (`discriminant(probe()) == x_arm_a && ...`).

This whole stack dissolves if a coproduct constructor reifies its own name as a
Symbol. The census measures, per file, how much of it is present.

Emits CSV to stdout.
"""
import os
import re
import sys

ROOT = sys.argv[1] if len(sys.argv) > 1 else "src/v4"

# self-named symbol tag:  data foo: Symbol = foo
TAG_RE = re.compile(r"^\s*data\s+(\w+)\s*:\s*Symbol\s*=\s*(\w+)\s*$", re.M)
# bridge fn header: fn name(...) -> Symbol {
BRIDGE_HDR_RE = re.compile(r"^\s*fn\s+(\w+)\s*\([^)]*\)\s*->\s*Symbol\s*\{", re.M)
# a match arm mapping a Constructor (Capitalized, optional {..}/(..)) to a symbol-ident.
# DOTALL on the payload so multi-line `{ ... }` arm patterns still match.
ARM_RE = re.compile(
    r"\b([A-Z]\w*)\s*(?:\{[^{}]*\}|\([^()]*\))?\s*=>\s*([a-z]\w*)\b", re.S
)
# roster-pin test: fn ... -> Bool whose body calls *_discriminant( and compares == *_arm*/symbol
DISC_CALL_RE = re.compile(r"\w*discriminant\s*\(")


def brace_body(text, open_idx):
    """Return substring of the balanced {..} block starting at open_idx (index of '{')."""
    depth = 0
    for i in range(open_idx, len(text)):
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return text[open_idx : i + 1]
    return text[open_idx:]


def analyze(path, text):
    tags = {m.group(1) for m in TAG_RE.finditer(text) if m.group(1) == m.group(2)}
    bridges = []  # (fn_name, arm_count, symbols_referenced)
    for m in BRIDGE_HDR_RE.finditer(text):
        body = brace_body(text, m.end() - 1)
        if "match" not in body:
            continue
        arms = ARM_RE.findall(body)
        # broad: any arm mapping Constructor => lowercase-ident is a discriminant bridge,
        # whether the target symbol is declared in this file or imported from the probe module.
        if len(arms) >= 2:
            bridges.append((m.group(1), len(arms), {r for _, r in arms}))
    # roster-pin tests: -> Bool fns invoking a discriminant and comparing to tags
    pin_tests = 0
    for m in re.finditer(r"^\s*fn\s+(\w+)\s*\([^)]*\)\s*->\s*Bool\s*\{", text, re.M):
        body = brace_body(text, m.end() - 1)
        # a roster-pin test calls a *discriminant( and compares results with ==
        if DISC_CALL_RE.search(body) and body.count("==") >= 2:
            pin_tests += 1
    shadow_syms = set()
    for _, _, syms in bridges:
        shadow_syms |= syms
    return tags, bridges, shadow_syms, pin_tests


def main():
    rows = []
    for dirpath, _, files in os.walk(ROOT):
        for fn in files:
            if not fn.endswith(".dag"):
                continue
            p = os.path.join(dirpath, fn)
            with open(p, encoding="utf-8", errors="replace") as f:
                text = f.read()
            tags, bridges, shadow_syms, pin_tests = analyze(p, text)
            if not bridges:
                continue  # only count files with an actual coproduct->symbol bridge
            arm_total = sum(b[1] for b in bridges)
            # dissolvable lines ~= shadow tag decls + bridge-fn arm lines + pin-test arm lines
            dissolvable = len(shadow_syms) + arm_total
            rows.append(
                {
                    "file": os.path.relpath(p, ROOT),
                    "bridge_fns": len(bridges),
                    "bridge_arms": arm_total,
                    "shadow_symbol_tags": len(shadow_syms),
                    "total_symbol_tags": len(tags),
                    "roster_pin_tests": pin_tests,
                    "est_lines_dissolvable": dissolvable,
                    "bridge_fn_names": ";".join(b[0] for b in bridges),
                }
            )
    rows.sort(key=lambda r: r["est_lines_dissolvable"], reverse=True)
    cols = [
        "file",
        "bridge_fns",
        "bridge_arms",
        "shadow_symbol_tags",
        "total_symbol_tags",
        "roster_pin_tests",
        "est_lines_dissolvable",
        "bridge_fn_names",
    ]
    print(",".join(cols))
    for r in rows:
        print(",".join(str(r[c]) for c in cols))
    # totals to stderr so CSV stays clean
    print(
        f"\nTOTAL files={len(rows)} "
        f"bridge_fns={sum(r['bridge_fns'] for r in rows)} "
        f"bridge_arms={sum(r['bridge_arms'] for r in rows)} "
        f"shadow_tags={sum(r['shadow_symbol_tags'] for r in rows)} "
        f"pin_tests={sum(r['roster_pin_tests'] for r in rows)} "
        f"est_lines_dissolvable={sum(r['est_lines_dissolvable'] for r in rows)}",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
