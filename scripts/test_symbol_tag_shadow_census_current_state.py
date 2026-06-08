#!/usr/bin/env python3
"""Assert the current genuine Symbol-tag shadow bridge census.

This is a symmetric current-state test, not a ratchet. It pins the exact set of
current bridge rows whose arm RHS references a same-file self-named tag
declaration of the form:

    data NAME: Symbol = NAME
"""

from __future__ import annotations

import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
V4_ROOT = ROOT / "src/v4"

TAG_RE = re.compile(r"^\s*data\s+(\w+)\s*:\s*Symbol\s*=\s*(\w+)\s*$", re.M)
BRIDGE_HDR_RE = re.compile(r"^\s*fn\s+(\w+)\s*\([^)]*\)\s*->\s*Symbol\s*\{", re.M)
ARM_RE = re.compile(
    r"\b([A-Z]\w*)\s*(?:\{[^{}]*\}|\([^()]*\))?\s*=>\s*([a-z]\w*)\b",
    re.S,
)

EXPECTED = {
    "lens/leaf_model_verification.dag": {
        "rust_r3_internal_value_emit_kind_label": (
            "rust_r3_internal_emit_kind_rejected",
        ),
    },
    "std/node.dag": {
        "named_edge_sort_key": (
            "canonical_tag_positional_edge",
        ),
    },
    "std/target_model.dag": {
        "target_type_expression_spelling": (
            "target_type_expression_decode_invalid",
        ),
    },
}


def brace_body(text: str, open_idx: int) -> str:
    depth = 0
    for i in range(open_idx, len(text)):
        char = text[i]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return text[open_idx : i + 1]
    return text[open_idx:]


def current_precise_census() -> dict[str, dict[str, tuple[str, ...]]]:
    rows: dict[str, dict[str, tuple[str, ...]]] = {}
    for path in sorted(V4_ROOT.rglob("*.dag")):
        text = path.read_text(encoding="utf-8", errors="replace")
        tags = {m.group(1) for m in TAG_RE.finditer(text) if m.group(1) == m.group(2)}
        bridge_rows: dict[str, tuple[str, ...]] = {}
        for match in BRIDGE_HDR_RE.finditer(text):
            body = brace_body(text, match.end() - 1)
            if "match" not in body:
                continue
            syms = {rhs for _constructor, rhs in ARM_RE.findall(body)}
            genuine = tuple(sorted(syms & tags))
            if genuine:
                bridge_rows[match.group(1)] = genuine
        if bridge_rows:
            rows[path.relative_to(V4_ROOT).as_posix()] = bridge_rows
    return rows


def main() -> None:
    observed = current_precise_census()
    if observed != EXPECTED:
        raise SystemExit(
            "precise Symbol-tag shadow census changed\n"
            f"expected: {EXPECTED!r}\n"
            f"observed: {observed!r}"
        )
    print("OK: precise Symbol-tag shadow census matches current state.")


if __name__ == "__main__":
    main()
