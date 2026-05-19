#!/usr/bin/env python3
"""Small unit checks for `strict_deprose_dag.py` (run: `python3 scripts/test_strict_deprose_dag.py`).

Inputs are declared in-process strings only (TESTING.md §1 hermetic discipline for new tests).
"""

from __future__ import annotations

import sys
from contextlib import redirect_stderr
from io import StringIO
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


def test_verilog_yellow_ref_post_terminal_footer_inline() -> None:
    # Merge-base places `#3200 RE-SCOPE` after `// Terminal:`; classifier must still see it.
    tail = (
        "// Coproduct dissolution (Practice 4 / modeling-discipline.md §4) —\n"
        "// 🟡 YELLOW (deferred-on-consumer; re-scoped under #3200 rule-change\n"
        "// Terminal: closed classifier.\n"
        "//\n"
        "// #3200 RE-SCOPE (rule-change 2026-05-16): post-terminal footer.\n"
    )
    assert s.verilog_yellow_ref(tail) == "SL-3229-VERILOG-D3200"


def test_inject_fails_when_coproduct_missing_from_tag_map() -> None:
    body = "type FreshCoproduct\n  = A\n  | B\n"
    buf = StringIO()
    with redirect_stderr(buf):
        try:
            s.inject_coproduct_tags(body, "src/v4/extdeps/languages/verilog.dag", {})
        except SystemExit as e:
            assert e.code == 1
        else:
            raise AssertionError("expected SystemExit")


def test_required_ledger_fails_on_unknown_live_coproduct() -> None:
    buf = StringIO()
    with redirect_stderr(buf):
        try:
            s.required_ledger_slugs("x.dag", {}, {"NotInManifest"})
        except SystemExit as e:
            assert e.code == 1
        else:
            raise AssertionError("expected SystemExit")


def test_strip_removes_grounded_tags_fail_closed() -> None:
    """Disk `// … grounded.` must not survive strip — inject_grounded_tags recomputes."""
    body = (
        "type X\n  = A | B\n"
        "// 🟢 grounded.\n"
        "type R {\n  rhs_lexeme: String\n}\n"
    )
    stripped = s.strip_body_comments(body)
    assert "grounded" not in stripped
    assert "rhs_lexeme" in stripped


def test_inject_grounded_recomputes_yellow_from_lexeme_string_field() -> None:
    bl = [
        "type R {",
        "  rhs_lexeme: String",
        "}",
    ]
    out = s.inject_grounded_tags(bl)
    assert out[0] == "// 🟡 grounded."


def test_inject_grounded_recomputes_green_without_lexeme_string_field() -> None:
    bl = [
        "type R {",
        "  x: Nat",
        "}",
    ]
    out = s.inject_grounded_tags(bl)
    assert out[0] == "// 🟢 grounded."


def test_inject_grounded_after_coproduct_dissolution_line() -> None:
    """Braced records must still get a tag when the prior out line is a dissolution banner."""
    bl = [
        "// 🟢 coproduct dissolution — DECISIONS.md Part 6 · CP-3229-GREEN-TERMINAL.",
        "type R {",
        "  x: Nat",
        "}",
    ]
    out = s.inject_grounded_tags(bl)
    assert out[2] == "// 🟢 grounded."
    assert out[3] == "type R {"


def main() -> None:
    test_inject_rewrites_stale_verilog_slug()
    test_verilog_yellow_ref_post_terminal_footer_inline()
    test_inject_fails_when_coproduct_missing_from_tag_map()
    test_required_ledger_fails_on_unknown_live_coproduct()
    test_strip_removes_grounded_tags_fail_closed()
    test_inject_grounded_recomputes_yellow_from_lexeme_string_field()
    test_inject_grounded_recomputes_green_without_lexeme_string_field()
    test_inject_grounded_after_coproduct_dissolution_line()
    print("OK: scripts/test_strict_deprose_dag.py")


if __name__ == "__main__":
    main()
