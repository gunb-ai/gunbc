#!/usr/bin/env python3
"""Migrate v4 .dag Outcome callsites: Produced/Rejected{diagnostic} -> Accepted/Rejected{diagnostics}."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
V4 = ROOT / "src" / "v4"


def protect_frontier(text: str) -> tuple[str, list[str]]:
    tokens: list[str] = []

    def repl(m: re.Match[str]) -> str:
        tokens.append(m.group(0))
        return f"__FWP{len(tokens)-1}__"

    text = re.sub(r"FrontierProduced|FrontierRejected", repl, text)
    return text, tokens


def unprotect_frontier(text: str, tokens: list[str]) -> str:
    for i, t in enumerate(tokens):
        text = text.replace(f"__FWP{i}__", t)
    return text


def migrate_imports(text: str) -> str:
    def fix_block(m: re.Match[str]) -> str:
        names = [n.strip() for n in m.group(1).split(",") if n.strip()]
        names = [n for n in names if n not in ("Produced", "Rejected")]
        if "Outcome" in m.group(0) or "Outcome" in names:
            for add in ("Accepted", "None", "diagnostics_singleton"):
                if add not in names:
                    names.append(add)
        return "import v4.std.diagnostic {" + ", ".join(names) + "}"

    return re.sub(r"import v4\.std\.diagnostic \{([^}]+)\}", fix_block, text)


def read_balanced(text: str, start: int, open_c: str, close_c: str) -> tuple[str, int]:
    assert text[start] == open_c
    depth = 0
    i = start
    while i < len(text):
        if text[i] == open_c:
            depth += 1
        elif text[i] == close_c:
            depth -= 1
            if depth == 0:
                return text[start : i + 1], i + 1
        i += 1
    raise ValueError(f"unbalanced {open_c}{close_c} at {start}")


def replace_rejected_diagnostic(text: str) -> str:
    out: list[str] = []
    i = 0
    pat = re.compile(r"Rejected\s*\{\s*diagnostic:\s*", re.MULTILINE)
    while i < len(text):
        m = pat.search(text, i)
        if not m:
            out.append(text[i:])
            break
        j = m.start()
        out.append(text[i:j])
        expr_start = m.end()
        # scan to end of diagnostic expression (comma-aware brace depth from expr_start)
        depth = 0
        k = expr_start
        while k < len(text):
            c = text[k]
            if c == "{":
                depth += 1
            elif c == "}":
                if depth == 0:
                    break
                depth -= 1
            k += 1
        expr = text[expr_start:k].strip()
        out.append(
            f"Rejected {{ diagnostics: diagnostics_singleton(d: {expr}) }}"
        )
        i = k + 1  # skip closing brace of Rejected
    return "".join(out)


def replace_produced(text: str) -> str:
    out: list[str] = []
    i = 0
    pat = re.compile(r"Produced\s*\{\s*value:\s*", re.MULTILINE)
    while i < len(text):
        m = pat.search(text, i)
        if not m:
            out.append(text[i:])
            break
        j = m.start()
        out.append(text[i:j])
        val_start = m.end()
        depth = 0
        k = val_start
        while k < len(text):
            c = text[k]
            if c == "{":
                depth += 1
            elif c == "}":
                if depth == 0:
                    break
                depth -= 1
            k += 1
        value = text[val_start:k].strip()
        out.append(f"Accepted {{ value: {value}, diagnostics: None }}")
        i = k + 1
    return "".join(out)


def migrate_file(path: Path) -> bool:
    if path.name == "diagnostic.dag":
        return False
    orig = path.read_text()
    if "Produced" not in orig and "Rejected { diagnostic" not in orig:
        return False
    text, fw = protect_frontier(orig)
    text = migrate_imports(text)
    text = replace_rejected_diagnostic(text)
    text = replace_produced(text)
    text = unprotect_frontier(text, fw)
    if text != orig:
        path.write_text(text)
        return True
    return False


def main() -> int:
    changed = [p for p in sorted(V4.rglob("*.dag")) if migrate_file(p)]
    print(f"migrated {len(changed)} files")
    for p in changed:
        print(p.relative_to(ROOT))
    rem_p = sum(1 for _ in V4.rglob("*.dag") for line in _.read_text().splitlines() if "Produced {" in line)
    rem_r = sum(
        1
        for p in V4.rglob("*.dag")
        for line in p.read_text().splitlines()
        if "Rejected { diagnostic" in line
    )
    print(f"remaining Produced lines: {rem_p}, Rejected diagnostic lines: {rem_r}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
