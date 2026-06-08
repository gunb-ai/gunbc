#!/usr/bin/env python3
"""Self-test for scripts/check_symbol_tag_shadow_census.py."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts/check_symbol_tag_shadow_census.py"


def run_checker(*args: str) -> str:
    result = subprocess.run(
        [sys.executable, str(CHECKER), *args],
        cwd=ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise SystemExit(result.stderr or result.stdout)
    return result.stdout


def main() -> None:
    clean = run_checker()
    if "OK: Symbol-tag shadow census is at or below the enforced baseline." not in clean:
        raise SystemExit("clean census check did not print the expected OK receipt")

    perturb = run_checker("--perturb-check")
    if "OK: Symbol-tag shadow census catches a planted bridge." not in perturb:
        raise SystemExit("perturb census check did not print the expected detection receipt")

    print("OK: check_symbol_tag_shadow_census self-test passed.")


if __name__ == "__main__":
    main()
