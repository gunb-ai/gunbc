#!/usr/bin/env python3
"""Small process helpers for CI Python gates.

The workflow shell-elimination path keeps command execution at Python
edges with explicit argv lists.  Helpers here intentionally avoid shell=True.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path
from typing import Iterable, Sequence


def repo_root() -> Path:
    proc = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return Path(proc.stdout.strip())


def run(argv: Sequence[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run(list(argv), check=True, text=True, **kwargs)


def output(argv: Sequence[str], **kwargs: object) -> str:
    proc = subprocess.run(
        list(argv),
        check=True,
        stdout=subprocess.PIPE,
        text=True,
        **kwargs,
    )
    return proc.stdout


def git_files(patterns: Iterable[str], cwd: Path) -> list[Path]:
    proc = subprocess.run(
        ["git", "ls-files", "-z", *patterns],
        cwd=cwd,
        check=True,
        stdout=subprocess.PIPE,
    )
    names = [p for p in proc.stdout.decode().split("\0") if p]
    return [cwd / name for name in names]


def gha_error(message: str) -> None:
    print(f"::error::{message}", file=sys.stderr)


def append_github_env(name: str, value: str) -> None:
    env_path = os.environ.get("GITHUB_ENV")
    if not env_path:
        print(f"{name}={value}")
        return
    with open(env_path, "a", encoding="utf-8") as handle:
        handle.write(f"{name}={value}\n")


def append_github_output(name: str, value: str) -> None:
    output_path = os.environ.get("GITHUB_OUTPUT")
    if not output_path:
        print(f"{name}={value}")
        return
    with open(output_path, "a", encoding="utf-8") as handle:
        handle.write(f"{name}={value}\n")
