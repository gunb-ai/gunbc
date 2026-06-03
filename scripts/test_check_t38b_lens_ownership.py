#!/usr/bin/env python3
"""Self-test for scripts/check_t38b_lens_ownership.py."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    result = subprocess.run(
        [sys.executable, str(ROOT / "scripts/check_t38b_lens_ownership.py")],
        cwd=ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise SystemExit(result.stderr or result.stdout)
    if "OK: T-38B lens_ownership subject roster + family_receipt" not in result.stdout:
        raise SystemExit("checker did not print the expected OK receipt")


if __name__ == "__main__":
    main()
