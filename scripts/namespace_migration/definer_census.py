#!/usr/bin/env python3
"""Find import rows that likely rely on re-export transitivity (import-from-definer migration).

Heuristic: for `import module { Sym }`, flag when Sym does not appear as a top-level
declaration in the module's source file (type/fn/data/module decl). Not a proof —
run `gunbc compile` after repointing to confirm.

Usage:
  python3 scripts/namespace_migration/definer_census.py [--roots dag src]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

IMPORT_START = re.compile(r"^\s*import\s+([A-Za-z0-9_.]+)\s*(?:\{|$)")
MODULE_RE = re.compile(r"^\s*module\s+([A-Za-z0-9_.]+)")
DECL_RE = re.compile(
    r"^\s*(?:type|fn|data|func|test\s+fn|test\s+data)\s+([A-Za-z_][A-Za-z0-9_]*)"
)


def module_path_to_file(root: Path, module_qn: str) -> Path | None:
    """Map dotted module path to a .dag file under root."""
    parts = module_qn.split(".")
    for prefix_len in range(len(parts), 0, -1):
        rel = "/".join(parts[:prefix_len]) + ".dag"
        candidate = root / rel
        if candidate.is_file():
            return candidate
        # v1 compiler modules often live under src/v1/
        if parts[0] == "v1" and root.name != "v1":
            candidate = root / "v1" / "/".join(parts[1:]) + ".dag"
            if candidate.is_file():
                return candidate
    # flat search fallback (slow but rare)
    tail = parts[-1] + ".dag"
    matches = list(root.rglob(tail))
    if len(matches) == 1:
        return matches[0]
    return None


def file_declared_symbols(text: str) -> set[str]:
    names: set[str] = set()
    for line in text.splitlines():
        m = DECL_RE.match(line)
        if m:
            names.add(m.group(1))
    return names


def parse_import_blocks(lines: list[str]) -> list[tuple[int, str, list[str]]]:
    """Return (line_no, module, symbols) for each import block."""
    results: list[tuple[int, str, list[str]]] = []
    i = 0
    while i < len(lines):
        m = IMPORT_START.match(lines[i])
        if not m:
            i += 1
            continue
        module = m.group(1)
        start = i + 1
        rest = lines[i].split("{", 1)
        if len(rest) == 1:
            # import module only
            results.append((start, module, []))
            i += 1
            continue
        block = rest[1]
        while "}" not in block and i + 1 < len(lines):
            i += 1
            block += "\n" + lines[i]
        body, _, _ = block.partition("}")
        syms = [
            s.strip()
            for s in body.replace("\n", " ").split(",")
            if s.strip() and re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", s.strip())
        ]
        results.append((start, module, syms))
        i += 1
    return results


def build_module_index(roots: list[Path]) -> dict[str, tuple[Path, set[str]]]:
    index: dict[str, tuple[Path, set[str]]] = {}
    for root in roots:
        if not root.is_dir():
            continue
        for path in root.rglob("*.dag"):
            try:
                text = path.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            mm = MODULE_RE.search(text)
            if not mm:
                continue
            qn = mm.group(1)
            index[qn] = (path, file_declared_symbols(text))
    return index


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--roots",
        nargs="+",
        default=["dag", "src"],
        help="Source roots to scan (default: dag src)",
    )
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[2]
    roots = [repo / r for r in args.roots]
    module_index = build_module_index(roots)

    suspects: list[str] = []
    for root in roots:
        for path in sorted(root.rglob("*.dag")):
            text = path.read_text(encoding="utf-8", errors="replace")
            for line_no, module, syms in parse_import_blocks(text.splitlines()):
                if not syms:
                    continue
                entry = module_index.get(module)
                if entry is None:
                    suspects.append(
                        f"{path}:{line_no}: import {module} {{...}} — module file not indexed"
                    )
                    continue
                _, declared = entry
                for sym in syms:
                    if sym not in declared:
                        suspects.append(
                            f"{path}:{line_no}: import {module} {{ {sym} }} — "
                            f"not declared in {module} (re-export suspect)"
                        )

    print(f"definer_census: {len(suspects)} re-export suspect rows")
    for row in suspects[:200]:
        print(row)
    if len(suspects) > 200:
        print(f"... and {len(suspects) - 200} more")
    return 0


if __name__ == "__main__":
    sys.exit(main())
