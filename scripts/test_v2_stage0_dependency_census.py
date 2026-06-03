#!/usr/bin/env python3
"""Self-test for scripts/v2_stage0_dependency_census.py."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/v2_stage0_dependency_census.py"


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="v2-stage0-census-test-") as tmp:
        stage0 = Path(tmp)
        write(
            stage0 / "lib.rs",
            "\n".join(
                (
                    "pub mod alpha;",
                    "pub mod beta;",
                    "pub mod gamma;",
                    "pub mod isolated;",
                )
            ),
        )
        write(stage0 / "alpha.rs", "pub fn a() { crate::beta::b(); }\n")
        write(stage0 / "beta.rs", "pub fn b() { crate::alpha::a(); crate::gamma::g(); }\n")
        write(stage0 / "gamma.rs", "pub fn g() {}\n")
        write(stage0 / "isolated.rs", "pub fn i() {}\n")

        result = subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                "--stage0-src",
                str(stage0),
                "--format",
                "json",
            ],
            cwd=ROOT,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if result.returncode != 0:
            raise SystemExit(result.stderr or result.stdout)

        data = json.loads(result.stdout)
        summary = data["summary"]
        if summary["module_count"] != 4:
            raise SystemExit(f"expected 4 modules, got {summary['module_count']}")
        if summary["edge_count"] != 3:
            raise SystemExit(f"expected 3 edges, got {summary['edge_count']}")
        if summary["cyclic_component_count"] != 1:
            raise SystemExit(
                f"expected 1 cyclic component, got {summary['cyclic_component_count']}"
            )

        components = {tuple(component["modules"]) for component in data["components"]}
        if ("alpha", "beta") not in components:
            raise SystemExit(f"missing alpha/beta SCC: {components}")
        cyclic = next(
            component for component in data["components"] if component["modules"] == ["alpha", "beta"]
        )
        if sorted(cyclic["internal_edges"]) != [["alpha", "beta"], ["beta", "alpha"]]:
            raise SystemExit(f"unexpected internal SCC edges: {cyclic['internal_edges']}")

        markdown = subprocess.run(
            [sys.executable, str(SCRIPT), "--stage0-src", str(stage0), "--top", "3"],
            cwd=ROOT,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if markdown.returncode != 0:
            raise SystemExit(markdown.stderr or markdown.stdout)
        if "v2 Stage0 Dependency Census" not in markdown.stdout:
            raise SystemExit("markdown output missing title")
        if "`alpha`, `beta`" not in markdown.stdout:
            raise SystemExit("markdown output missing cyclic component")

    print("OK: v2 stage0 dependency census self-test.")


if __name__ == "__main__":
    main()
