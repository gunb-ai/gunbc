#!/usr/bin/env python3
"""Add role field to Edge and EdgeShape literals in .dag files (L-NEW-b codemod)."""

from __future__ import annotations

import sys
from pathlib import Path

REFERENCE_MARKERS = (
    "^dependency_binds_to_edge",
    "^dependency_module_import_edge",
    "^dependency_bootstrap_depends_edge",
)

DEFAULT_CONTAINMENT = "Containment { inheritable: false }"


def infer_role(block: str) -> str:
    for marker in REFERENCE_MARKERS:
        if marker in block:
            return "Reference"
    return DEFAULT_CONTAINMENT


def is_literal_start(content: str, i: int, type_name: str) -> bool:
    prefix = f"{type_name} {{"
    if not content.startswith(prefix, i):
        return False
    if i > 0 and (content[i - 1].isalnum() or content[i - 1] == "_"):
        return False
    j = i - 1
    while j >= 0 and content[j] in " \t":
        j -= 1
    if j >= 1 and content[j - 1 : j + 1] == "->":
        return False
    return True


def transform_content_once(content: str) -> str:
    result: list[str] = []
    i = 0
    n = len(content)
    changed = False
    while i < n:
        matched = None
        for type_name in ("EdgeShape", "Edge"):
            if is_literal_start(content, i, type_name):
                matched = type_name
                break
        if matched is None:
            result.append(content[i])
            i += 1
            continue

        start = i
        i += len(matched) + 2
        depth = 1
        body_start = i
        while i < n and depth > 0:
            ch = content[i]
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
            i += 1
        full = content[start:i]
        body = content[body_start : i - 1]
        if "role:" in body:
            result.append(full)
        else:
            changed = True
            role = infer_role(full)
            inner = body.strip()
            result.append(f"{matched} {{ role: {role}, {inner} }}")
    if changed:
        return "".join(result)
    return content


def transform_content(content: str) -> str:
    while True:
        updated = transform_content_once(content)
        if updated == content:
            return updated
        content = updated


def patch_edge_copies(content: str) -> str:
    for var in ("edge", "e"):
        old = f"Edge {{ label: {var}.label, target:"
        new = f"Edge {{ label: {var}.label, role: {var}.role, target:"
        idx = 0
        while True:
            pos = content.find(old, idx)
            if pos == -1:
                break
            head = content[pos : pos + len(old) + 40]
            if "role:" in head.split("target:")[0]:
                idx = pos + len(old)
                continue
            content = content[:pos] + new + content[pos + len(old) :]
            idx = pos + len(new)
    return content


def process_file(path: Path) -> bool:
    original = path.read_text(encoding="utf-8")
    updated = patch_edge_copies(transform_content(original))
    if updated != original:
        path.write_text(updated, encoding="utf-8")
        return True
    return False


def main() -> int:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
    changed = 0
    for path in sorted(root.rglob("*.dag")):
        if process_file(path):
            changed += 1
            print(path)
    print(f"updated {changed} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
