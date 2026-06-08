#!/usr/bin/env python3
"""Assert the current genuine Symbol-tag shadow bridge census.

This is a symmetric current-state test, not a ratchet. It pins the exact set of
current bridge rows whose arm RHS references a same-file self-named tag
declaration of the form:

    data NAME: Symbol = NAME
"""

from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CENSUS_SCRIPT = ROOT / "scripts/symbol_tag_shadow_census.py"
V4_ROOT = ROOT / "src/v4"

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


def load_census_module():
    spec = importlib.util.spec_from_file_location("symbol_tag_shadow_census", CENSUS_SCRIPT)
    if spec is None or spec.loader is None:
        raise SystemExit("failed to load symbol_tag_shadow_census.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def current_precise_census() -> dict[str, dict[str, tuple[str, ...]]]:
    census = load_census_module()
    rows: dict[str, dict[str, tuple[str, ...]]] = {}
    for path in sorted(V4_ROOT.rglob("*.dag")):
        text = path.read_text(encoding="utf-8", errors="replace")
        tags, bridges, _shadow_syms, _pin_tests = census.analyze(str(path), text)
        bridge_rows: dict[str, tuple[str, ...]] = {}
        for name, _arm_count, syms in bridges:
            genuine = tuple(sorted(syms & tags))
            if genuine:
                bridge_rows[name] = genuine
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
