#!/usr/bin/env python3
"""Hermetic unit checks for `l1_1_discriminant_predicate.py` (TESTING.md §1).

Run: `python3 scripts/test_l1_1_discriminant_predicate.py`
"""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import l1_1_discriminant_predicate as l1  # noqa: E402

FIXTURE_DIRECT = """
module t.fixture.direct

type Nat
  = Zero
  | Succ { prev: Nat }

fn nat_is_zero(n: Nat) -> Bool {
  match n {
    Zero => true
    Succ { prev: _ } => false
  }
}
"""

FIXTURE_COMPUTED_PREDICATE = """
module t.fixture.computed

type Nat
  = Zero
  | Succ { prev: Nat }

fn nat_is_weird(n: Nat) -> Bool {
  match n {
    Zero => true
    Succ { prev: p } => nat_is_weird(n: p)
  }
}
"""

FIXTURE_GENUINE_CAT = """
module t.fixture.cat

type Nat
  = Zero
  | Succ { prev: Nat }

fn nat_add(a: Nat, b: Nat) -> Nat {
  match a {
    Zero => b
    Succ { prev: p } => Succ { prev: nat_add(a: p, b: b) }
  }
}
"""

FIXTURE_FOLD_LAUNDERED = """
module t.fixture.foldl

type FM
  = Empty
  | Cell { tail: FM }

fn fm_is_empty_via_fold(xs: FM) -> Bool {
  fold(xs, init: true, f: fn(_, _){ false })
}
"""

FIXTURE_FOLD_USES_ACC = """
module t.fixture.foldacc

type FM
  = Empty
  | Cell { tail: FM }

fn fm_len(xs: FM) -> Int {
  fold(xs, init: 0, f: fn(acc, _) {
    acc + 1
  })
}
"""


def test_flags_direct_discriminant_match() -> None:
    fs = l1.findings_in_text("direct.dag", FIXTURE_DIRECT)
    kinds = {f.kind for f in fs}
    assert "direct" in kinds, fs


def test_flags_fold_laundered_constant_algebra() -> None:
    fs = l1.findings_in_text("foldl.dag", FIXTURE_FOLD_LAUNDERED)
    kinds = {f.kind for f in fs}
    assert "fold-laundered" in kinds, fs


def test_computed_predicate_not_flagged() -> None:
    fs = l1.findings_in_text("computed.dag", FIXTURE_COMPUTED_PREDICATE)
    assert fs == [], fs


def test_genuine_catamorphism_not_flagged() -> None:
    fs = l1.findings_in_text("cat.dag", FIXTURE_GENUINE_CAT)
    assert fs == [], fs


def test_fold_using_accumulator_not_flagged() -> None:
    fs = l1.findings_in_text("foldacc.dag", FIXTURE_FOLD_USES_ACC)
    assert fs == [], fs


def test_scan_files_outside_repo_relpath_fallback() -> None:
    """`scan_files` must not rely on Path.is_relative_to (3.9+); use relative_to + ValueError."""
    body = """module t.o

type A
  = X
  | Y

fn f(x: A) -> Bool {
  match x {
    X => true
    Y => false
  }
}
"""
    with tempfile.TemporaryDirectory() as d:
        p = Path(d) / "orphan.dag"
        p.write_text(body, encoding="utf-8")
        fs = l1.scan_files([p])
    assert len(fs) == 1
    assert fs[0].rel == str(p)
    assert fs[0].fn_name == "f"
    assert fs[0].kind == "direct"


def main() -> None:
    test_flags_direct_discriminant_match()
    test_flags_fold_laundered_constant_algebra()
    test_computed_predicate_not_flagged()
    test_genuine_catamorphism_not_flagged()
    test_fold_using_accumulator_not_flagged()
    test_scan_files_outside_repo_relpath_fallback()
    print("OK: scripts/test_l1_1_discriminant_predicate.py")


if __name__ == "__main__":
    main()
