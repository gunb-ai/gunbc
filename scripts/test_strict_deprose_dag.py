#!/usr/bin/env python3
"""Hermetic checks for `strict_deprose_dag.py` (run: `python3 scripts/test_strict_deprose_dag.py`)."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import strict_deprose_dag as s  # noqa: E402


def test_inject_rewrites_stale_verilog_slug() -> None:
    body = (
        "// 🟡 coproduct dissolution — DECISIONS.md Part 6 · SL-3229-VERILOG-NONEMPTY.\n"
        "type NonTriregNetKind\n"
        "  = Wire\n"
        "  | Tri\n"
    )
    tag_map = {"NonTriregNetKind": ("🟡", "SL-3229-VERILOG-D3200")}
    out = s.inject_coproduct_tags(body, "src/v4/extdeps/languages/verilog.dag", tag_map)
    assert "SL-3229-VERILOG-D3200" in out, out
    assert "SL-3229-VERILOG-NONEMPTY" not in out, out


def test_verilog_yellow_ref_sees_post_terminal_footer() -> None:
    mb = s.git_merge_base_lines("src/v4/extdeps/languages/verilog.dag")
    for i, ln in enumerate(mb):
        if s.is_coproduct(mb, i) == "NonTriregNetKind":
            tail = s.practice4_tail_for_face(mb, i)
            assert s.verilog_yellow_ref(tail) == "SL-3229-VERILOG-D3200", tail[:500]
            return
    raise AssertionError("NonTriregNetKind not found in merge-base verilog.dag")


def main() -> None:
    test_inject_rewrites_stale_verilog_slug()
    test_verilog_yellow_ref_sees_post_terminal_footer()
    print("OK: scripts/test_strict_deprose_dag.py")


if __name__ == "__main__":
    main()
