#!/usr/bin/env python3
"""Self-test for scripts/check_v4_layering_imports.py."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts/check_v4_layering_imports.py"


def run_checker(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(CHECKER), *args],
        cwd=ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def main() -> None:
    clean = run_checker()
    if clean.returncode != 0:
        raise SystemExit(clean.stderr or clean.stdout)
    if "OK: no std/ or extdeps/ imports" not in clean.stdout:
        raise SystemExit("clean scan did not print the expected OK receipt")

    perturb = run_checker("--perturb-check")
    if perturb.returncode != 0:
        raise SystemExit(perturb.stderr or perturb.stdout)
    if "OK: layering import gate detects planted" not in perturb.stdout:
        raise SystemExit("perturb-check did not print the expected OK receipt")

    print("OK: check_v4_layering_imports self-test passed.")


if __name__ == "__main__":
    main()
