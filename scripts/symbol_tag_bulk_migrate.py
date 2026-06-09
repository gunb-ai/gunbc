#!/usr/bin/env python3
"""Migrate tautological `data NAME: Symbol = NAME` to `^NAME` and delete declarations.

Safe-bulk scope: excludes load-bearing std, compiler/, and gates per E2 plan.
Aliased `data A: Symbol = B` rewrites refs to `^B` and deletes declarations.
"""
from __future__ import annotations

import argparse
import os
import re
import sys
from collections import defaultdict

TAG_RE = re.compile(r"^\s*data\s+(\w+)\s*:\s*Symbol\s*=\s*(\w+)\s*$", re.M)

LOAD_BEARING_STD = {
    "std/node.dag",
    "std/verification.dag",
    "std/target_model.dag",
    "std/grammar.dag",
    "std/algebra.dag",
    "std/cardinality.dag",
    "std/model_core.dag",
    "std/refinement.dag",
    "std/effects.dag",
    "std/grounding.dag",
    "std/coercion.dag",
    "std/find_witness.dag",
    "std/leaf_model_verification.dag",
    "std/logic.dag",
}
LOAD_BEARING_GATES = {"workflow/lens_ci_gate.dag"}


def rel_path(root: str, path: str) -> str:
    return os.path.relpath(path, root)


def in_scope(file: str, include_compiler: bool) -> bool:
    if file.startswith("compiler/"):
        return include_compiler
    if file in LOAD_BEARING_STD or file in LOAD_BEARING_GATES:
        return False
    return True


def collect_tags(text: str) -> list[tuple[str, str, int]]:
    out = []
    for m in TAG_RE.finditer(text):
        name, rhs = m.group(1), m.group(2)
        line = text[: m.start()].count("\n") + 1
        out.append((name, rhs, line))
    return out


def rewrite_file(text: str, tags: list[tuple[str, str, int]]) -> tuple[str, list[str]]:
    """Return (new_text, actions)."""
    if not tags:
        return text, []
    actions = []
    new_text = text
    # Delete declarations bottom-up to preserve offsets.
    decl_spans = []
    for m in TAG_RE.finditer(text):
        name, rhs = m.group(1), m.group(2)
        if not any(t[0] == name and t[1] == rhs for t in tags):
            continue
        start = m.start()
        end = m.end()
        # swallow trailing blank line
        if end < len(text) and text[end : end + 1] == "\n":
            end += 1
        decl_spans.append((start, end, name, rhs))
    for start, end, name, rhs in sorted(decl_spans, key=lambda x: x[0], reverse=True):
        actions.append(f"delete data {name}: Symbol = {rhs}")
        new_text = new_text[:start] + new_text[end:]

    # Replace identifier refs with ^RHS (aliased uses rhs spelling).
    replace_map = {}
    for name, rhs, _ in tags:
        replace_map[name] = f"^{rhs}"

    # Longest names first to avoid partial replacement.
    for name in sorted(replace_map, key=len, reverse=True):
        caret = replace_map[name]
        new_text = re.sub(r"(?<!\^)\b" + re.escape(name) + r"\b", caret, new_text)
        actions.append(f"refs {name} -> {caret}")

    return new_text, actions


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("root", nargs="?", default="src/v4")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--include-compiler", action="store_true")
    ap.add_argument("--files", nargs="*", help="Restrict to these rel paths")
    args = ap.parse_args()

    changed = 0
    total_tags = 0
    for dirpath, _, files in os.walk(args.root):
        for fn in sorted(files):
            if not fn.endswith(".dag"):
                continue
            path = os.path.join(dirpath, fn)
            file = rel_path(args.root, path)
            if args.files and file not in args.files:
                continue
            if not in_scope(file, args.include_compiler):
                continue
            with open(path, encoding="utf-8", errors="replace") as f:
                text = f.read()
            all_tags = collect_tags(text)
            # Only tautological and aliased (both handled via rhs spelling).
            tags = [(n, r, ln) for n, r, ln in all_tags]
            if not tags:
                continue
            new_text, actions = rewrite_file(text, tags)
            if new_text == text:
                continue
            total_tags += len(tags)
            print(f"{file}: {len(tags)} tags")
            for a in actions:
                print(f"  {a}")
            if not args.dry_run:
                with open(path, "w", encoding="utf-8") as f:
                    f.write(new_text)
            changed += 1
    print(f"\n{'would change' if args.dry_run else 'changed'} {changed} files, {total_tags} tags", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
