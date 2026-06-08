#!/usr/bin/env python3
"""Fail-closed gate for the Symbol-tag shadow taxonomy census.

This wraps ``scripts/symbol_tag_shadow_census.py`` with a ratchet: known residual
bridge files may remain only at or below their measured counts, and no new files
may enter the census. Once the hardened census parser from the Mgr-ENF-2 lane is
present, the allowed residual floor is zero by construction.

Usage:
  check_symbol_tag_shadow_census.py
  check_symbol_tag_shadow_census.py --perturb-check
"""

from __future__ import annotations

import argparse
import csv
import importlib.util
import shutil
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CENSUS_SCRIPT = ROOT / "scripts/symbol_tag_shadow_census.py"

# Baseline measured on main before the hardened parser from PR #4570. The checker
# accepts lower counts, so after that parser lands this ratchet naturally becomes
# a zero-floor gate without another edit.
MAX_RESIDUALS: dict[str, tuple[int, int, int]] = {
    "std/target_model.dag": (3, 10, 4),
    "extdeps/languages/swift.dag": (1, 4, 1),
    "lens/leaf_model_verification.dag": (1, 2, 2),
    "std/node.dag": (1, 2, 2),
    "test/claim/manual/infer_ground_add_mvp.dag": (1, 2, 1),
}


@dataclass(frozen=True)
class CensusRow:
    file: str
    bridge_fns: int
    bridge_arms: int
    shadow_symbol_tags: int


def load_census_module():
    spec = importlib.util.spec_from_file_location("symbol_tag_shadow_census", CENSUS_SCRIPT)
    if spec is None or spec.loader is None:
        raise SystemExit("failed to load scripts/symbol_tag_shadow_census.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def census_rows(root: Path) -> list[CensusRow]:
    census = load_census_module()
    rows: list[CensusRow] = []
    for path in sorted(root.rglob("*.dag")):
        text = path.read_text(encoding="utf-8", errors="replace")
        _tags, bridges, shadow_syms, _pin_tests = census.analyze(str(path), text)
        if not bridges:
            continue
        rows.append(
            CensusRow(
                file=path.relative_to(root).as_posix(),
                bridge_fns=len(bridges),
                bridge_arms=sum(bridge[1] for bridge in bridges),
                shadow_symbol_tags=len(shadow_syms),
            )
        )
    rows.sort(key=lambda row: row.file)
    return rows


def violations(rows: list[CensusRow]) -> list[str]:
    out: list[str] = []
    for row in rows:
        allowed = MAX_RESIDUALS.get(row.file)
        if allowed is None:
            out.append(f"{row.file}: new Symbol-tag shadow bridge file")
            continue
        max_bridge_fns, max_bridge_arms, max_shadow_tags = allowed
        if row.bridge_fns > max_bridge_fns:
            out.append(
                f"{row.file}: bridge_fns {row.bridge_fns} exceeds baseline {max_bridge_fns}"
            )
        if row.bridge_arms > max_bridge_arms:
            out.append(
                f"{row.file}: bridge_arms {row.bridge_arms} exceeds baseline {max_bridge_arms}"
            )
        if row.shadow_symbol_tags > max_shadow_tags:
            out.append(
                f"{row.file}: shadow_symbol_tags {row.shadow_symbol_tags} exceeds baseline {max_shadow_tags}"
            )
    return out


def print_rows(rows: list[CensusRow]) -> None:
    writer = csv.writer(sys.stdout)
    writer.writerow(["file", "bridge_fns", "bridge_arms", "shadow_symbol_tags"])
    for row in rows:
        writer.writerow([row.file, row.bridge_fns, row.bridge_arms, row.shadow_symbol_tags])


def run_check(root: Path) -> int:
    rows = census_rows(root)
    errs = violations(rows)
    if errs:
        for err in errs:
            print(err, file=sys.stderr)
        print("\nObserved census:", file=sys.stderr)
        print_rows(rows)
        return 1
    print_rows(rows)
    print("OK: Symbol-tag shadow census is at or below the enforced baseline.")
    return 0


def plant_perturb(root: Path) -> None:
    fixture = root / "src/v4/test/claim/manual/_perturb_symbol_tag_shadow_census.dag"
    fixture.parent.mkdir(parents=True, exist_ok=True)
    fixture.write_text(
        "\n".join(
            [
                "module v4.test.claim.manual._perturb_symbol_tag_shadow_census",
                "",
                "type PerturbSymbolTagShadow =",
                "  | PerturbAlpha { value: Symbol }",
                "  | PerturbBeta { value: Symbol }",
                "",
                "data perturb_alpha_tag: Symbol = perturb_alpha_tag",
                "data perturb_beta_tag: Symbol = perturb_beta_tag",
                "",
                "fn perturb_symbol_tag_shadow_discriminant(v: PerturbSymbolTagShadow) -> Symbol {",
                "  match v {",
                "    PerturbAlpha { value: _ } => perturb_alpha_tag",
                "    PerturbBeta { value: _ } => perturb_beta_tag",
                "  }",
                "}",
                "",
            ]
        ),
        encoding="utf-8",
    )


def run_perturb_check(root: Path) -> int:
    if run_check(root) != 0:
        print("FAIL: clean tree must pass before perturb-check.", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory(prefix="symbol-tag-census-") as tmp:
        tmp_root = Path(tmp) / "src" / "v4"
        shutil.copytree(root, tmp_root)
        plant_perturb(tmp_root.parent.parent)
        rows = census_rows(tmp_root)
        errs = violations(rows)
        if not any("_perturb_symbol_tag_shadow_census.dag" in err for err in errs):
            print("FAIL: perturb-check did not catch planted Symbol-tag shadow bridge.", file=sys.stderr)
            print_rows(rows)
            return 1

    print("OK: Symbol-tag shadow census catches a planted bridge.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=ROOT / "src/v4",
        help="v4 root to scan (default: src/v4)",
    )
    parser.add_argument(
        "--perturb-check",
        action="store_true",
        help="plant a bridge in a temp tree and require detection",
    )
    args = parser.parse_args()
    root = args.root.resolve()
    if args.perturb_check:
        return run_perturb_check(root)
    return run_check(root)


if __name__ == "__main__":
    raise SystemExit(main())
