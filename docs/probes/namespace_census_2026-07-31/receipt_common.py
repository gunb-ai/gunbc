"""Shared pinned-input authority for the namespace census receipt tools."""

import json
import pathlib
import subprocess


def load_summary(path: pathlib.Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def require_pinned_repo(repo: pathlib.Path, summary: dict) -> pathlib.Path:
    resolved = repo.resolve()
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=resolved, capture_output=True, text=True, check=False,
    )
    if result.returncode != 0:
        raise SystemExit(f"cannot identify corpus checkout: {result.stderr.strip()}")
    actual = result.stdout.strip()
    expected = summary["inputs"]["corpus_commit"]
    if actual != expected:
        raise SystemExit(f"corpus checkout drift: expected {expected}, got {actual}")
    return resolved
