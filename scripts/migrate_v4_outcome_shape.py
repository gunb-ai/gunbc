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

    text = re.sub(r"FrontierProduced", repl, text)
    return text, tokens


def unprotect_frontier(text: str, tokens: list[str]) -> str:
    for i, t in enumerate(tokens):
        text = text.replace(f"__FWP{i}__", t)
    return text


def migrate_imports(text: str) -> str:
    def fix_block(m: re.Match[str]) -> str:
        names = [n.strip() for n in m.group(1).split(",") if n.strip()]
        names = [n for n in names if n not in ("Produced", "Rejected")]
        if any("Outcome" in n or n == "Outcome" for n in names) or "Outcome" in m.group(0):
            for add in ("Accepted", "None", "diagnostics_singleton"):
                if add not in names:
                    names.append(add)
        return "import v4.std.diagnostic {" + ", ".join(names) + "}"

    return re.sub(r"import v4\.std\.diagnostic \{([^}]+)\}", fix_block, text)


def replace_produced(text: str) -> str:
    out: list[str] = []
    i = 0
    while i < len(text):
        m = re.match(r"Produced\s*\{\s*value:\s*", text[i:])
        if not m:
            out.append(text[i])
            i += 1
            continue
        i += m.end()
        depth = 1
        vs = i
        while i < len(text) and depth > 0:
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
            i += 1
        value = text[vs : i - 1].strip()
        out.append(f"Accepted {{ value: {value}, diagnostics: None }}")
    return "".join(out)


def migrate_file(path: Path) -> bool:
    if path.name == "diagnostic.dag":
        return False
    orig = path.read_text()
    text = orig
    if "Produced" not in text and "Rejected { diagnostic" not in text:
        return False
    text, fw = protect_frontier(text)
    text = migrate_imports(text)
    text = re.sub(
        r"Rejected\s*\{\s*diagnostic:\s*",
        "Rejected { diagnostics: diagnostics_singleton(d: ",
        text,
    )
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
    return 0


if __name__ == "__main__":
    sys.exit(main())
