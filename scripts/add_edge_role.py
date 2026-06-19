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


def transform_content(content: str) -> str:
    result: list[str] = []
    i = 0
    n = len(content)
    while i < n:
        matched = None
        for type_name in ("EdgeShape", "Edge"):
            prefix = f"{type_name} {{"
            if content.startswith(prefix, i):
                matched = type_name
                break
        if matched is None:
            result.append(content[i])
            i += 1
            continue

        start = i
        i += len(matched) + 2  # past "Type {"
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
            role = infer_role(full)
            inner = body.strip()
            result.append(f"{matched} {{ role: {role}, {inner} }}")
    return "".join(result)


def patch_edge_copies(content: str) -> str:
    replacements = [
        (
            "Edge { label: edge.label, target:",
            "Edge { label: edge.label, role: edge.role, target:",
        ),
        (
            "Edge { label: e.label, target:",
            "Edge { label: e.label, role: e.role, target:",
        ),
    ]
    for old, new in replacements:
        content = content.replace(old, new)
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
