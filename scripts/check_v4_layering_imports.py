#!/usr/bin/env python3
"""Cross-layer import gate — INVARIANTS P2 dependency-direction violation.

Module layers form a strict DAG (std/ <- extdeps/ <- compiler/ <- workflow/).
Files under std/ or extdeps/ must not import v3.compiler.* or v4.compiler.*.

Usage:
  check_v4_layering_imports.py              # scan the repo (CI clean-tree gate)
  check_v4_layering_imports.py --perturb-check
      # plant extdeps->compiler and std->compiler imports in a temp tree;
      # require the scanner to fail (detection-test receipt)

Exit 0 when no violations; 1 on any wrong-direction import or perturb miss.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

IMPORT_RE = re.compile(r"^\s*import\s+([\w.]+)")
FORBIDDEN_PREFIXES = ("v3.compiler.", "v4.compiler.")
FORBIDDEN_EXACT = ("v3.compiler", "v4.compiler")

# (layer_label, repo-relative root). Path segment `/std/` or `/extdeps/` is the authority.
LAYER_ROOTS: tuple[tuple[str, str], ...] = (
    ("std", "src/v4/std"),
    ("std", "src/v3/std"),
    ("std", "dsl/std"),
    ("extdeps", "src/v4/extdeps"),
    ("extdeps", "dsl/extdeps"),
)


@dataclass(frozen=True)
class Violation:
    layer: str
    path: str
    line_no: int
    module: str

    def format(self) -> str:
        return (
            f"{self.path}:{self.line_no}: {self.layer}/ must not import compiler "
            f"(wrong-direction layering) -> import {self.module}"
        )


def repo_root(explicit: Path | None) -> Path:
    if explicit is not None:
        return explicit.resolve()
    out = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True,
        text=True,
        check=True,
    )
    return Path(out.stdout.strip())


def layer_for_path(relpath: str) -> str | None:
    normalized = relpath.replace("\\", "/")
    if "/std/" in normalized or normalized.endswith("/std") or normalized.startswith("std/"):
        return "std"
    if "/extdeps/" in normalized or normalized.endswith("/extdeps") or normalized.startswith("extdeps/"):
        return "extdeps"
    return None


def is_forbidden_compiler_import(module: str) -> bool:
    if module in FORBIDDEN_EXACT:
        return True
    return any(module.startswith(prefix) for prefix in FORBIDDEN_PREFIXES)


def iter_layer_dag_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for _layer, rel_root in LAYER_ROOTS:
        base = root / rel_root
        if not base.is_dir():
            continue
        files.extend(sorted(base.rglob("*.dag")))
    return files


def scan_file(root: Path, path: Path) -> list[Violation]:
    relpath = path.relative_to(root).as_posix()
    layer = layer_for_path(relpath)
    if layer is None:
        return []
    out: list[Violation] = []
    text = path.read_text(encoding="utf-8")
    for line_no, line in enumerate(text.splitlines(), 1):
        match = IMPORT_RE.match(line)
        if not match:
            continue
        module = match.group(1)
        if is_forbidden_compiler_import(module):
            out.append(Violation(layer=layer, path=relpath, line_no=line_no, module=module))
    return out


def scan(root: Path) -> list[Violation]:
    violations: list[Violation] = []
    for path in iter_layer_dag_files(root):
        violations.extend(scan_file(root, path))
    violations.sort(key=lambda v: (v.path, v.line_no, v.module))
    return violations


def run_scan(root: Path) -> int:
    violations = scan(root)
    for violation in violations:
        print(violation.format())
    if violations:
        print(
            f"\nFAIL: {len(violations)} wrong-direction compiler import(s) in std/ or extdeps/.",
            file=sys.stderr,
        )
        return 1
    print("OK: no std/ or extdeps/ imports from v3.compiler / v4.compiler.")
    return 0


def plant_perturb_fixtures(root: Path) -> tuple[str, str]:
    std_dir = root / "src/v4/std"
    extdeps_dir = root / "src/v4/extdeps"
    std_dir.mkdir(parents=True)
    extdeps_dir.mkdir(parents=True)

    std_rel = "src/v4/std/_perturb_layering_std_to_compiler.dag"
    extdeps_rel = "src/v4/extdeps/_perturb_layering_extdeps_to_compiler.dag"

    (root / std_rel).write_text(
        "\n".join(
            [
                "module v4.std._perturb_layering_std_to_compiler",
                "import v4.compiler.tokenize { tokenize }",
                "data perturb_std_to_compiler: Bool = true",
                "",
            ]
        ),
        encoding="utf-8",
    )
    (root / extdeps_rel).write_text(
        "\n".join(
            [
                "module v4.extdeps._perturb_layering_extdeps_to_compiler",
                "import v4.compiler.parse { parse }",
                "data perturb_extdeps_to_compiler: Bool = true",
                "",
            ]
        ),
        encoding="utf-8",
    )
    return std_rel, extdeps_rel


def run_perturb_check(root: Path) -> int:
    clean_rc = run_scan(root)
    if clean_rc != 0:
        print("FAIL: clean tree must be green before perturb-check.", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory(prefix="layering-perturb-") as tmp:
        tmp_root = Path(tmp)
        std_rel, extdeps_rel = plant_perturb_fixtures(tmp_root)
        violations = scan(tmp_root)
        found_paths = {v.path for v in violations}
        expected = {std_rel, extdeps_rel}
        if found_paths != expected:
            print(
                "FAIL: perturb-check expected violations exactly in "
                f"{sorted(expected)!r}, got {sorted(found_paths)!r}.",
                file=sys.stderr,
            )
            for violation in violations:
                print(violation.format(), file=sys.stderr)
            return 1
        if len(violations) != 2:
            print(
                f"FAIL: perturb-check expected 2 violations, got {len(violations)}.",
                file=sys.stderr,
            )
            return 1

    print("OK: layering import gate detects planted std->compiler and extdeps->compiler violations.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=None,
        help="repository root (default: git rev-parse --show-toplevel)",
    )
    parser.add_argument(
        "--perturb-check",
        action="store_true",
        help="plant wrong-direction imports in a temp tree and require detection",
    )
    args = parser.parse_args()
    root = repo_root(args.root)
    if args.perturb_check:
        return run_perturb_check(root)
    return run_scan(root)


if __name__ == "__main__":
    raise SystemExit(main())
