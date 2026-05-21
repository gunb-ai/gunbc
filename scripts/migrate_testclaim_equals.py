#!/usr/bin/env python3
"""Migrate TestClaim { kind: Equals, ... expected: ... } to DiagnosticClaim."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FILES = [
    "src/v4/test/claim/diagnostic_correction/show_correct_code.dag",
    "src/v4/test/claim/impossible_bug/unhandled_diagnostic_paths.dag",
    "src/v4/test/claim/impossible_bug/transport_type_drift.dag",
    "src/v4/test/claim/impossible_bug/nested_optional_flatten.dag",
    "src/v4/test/claim/impossible_bug/idempotency_contract.dag",
    "src/v4/test/claim/impossible_bug/unenumerated_effects.dag",
    "src/v4/test/claim/impossible_bug/suboptimal_complexity.dag",
    "src/v4/test/claim/algebra_laws/nat_semiring.dag",
]


def migrate_block(block: str) -> str:
    if "kind: Equals" not in block:
        return block
    block = re.sub(
        r"data (\w+): TestClaim = TestClaim \{\s*\n\s*kind: Equals,\s*\n",
        r"data \1: TestClaim = DiagnosticClaim {\n",
        block,
        count=1,
    )
    block = block.replace("expected:", "expected_outcome:", 1)
    return block


def migrate_file(path: Path) -> bool:
    text = path.read_text()
    if "kind: Equals" not in text:
        return False
    parts = re.split(r"(?=^data \w+: TestClaim = )", text, flags=re.M)
    new_parts = [migrate_block(p) if p.startswith("data ") else p for p in parts]
    new_text = "".join(new_parts)
    if new_text == text:
        return False
    path.write_text(new_text)
    return True


def main() -> int:
    changed = 0
    for rel in FILES:
        p = ROOT / rel
        if migrate_file(p):
            print("migrated", rel)
            changed += 1
    return 0 if changed else 1


if __name__ == "__main__":
    sys.exit(main())
