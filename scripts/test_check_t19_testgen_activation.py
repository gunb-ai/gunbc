#!/usr/bin/env python3
"""Self-test for check_t19_testgen_activation.py."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    proc = subprocess.run(
        [sys.executable, str(ROOT / "scripts/check_t19_testgen_activation.py")],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise SystemExit(f"check failed:\n{proc.stdout}\n{proc.stderr}")
    if "OK:" not in proc.stdout:
        raise SystemExit(f"unexpected output: {proc.stdout!r}")
    print("OK: scripts/test_check_t19_testgen_activation.py")


if __name__ == "__main__":
    main()
