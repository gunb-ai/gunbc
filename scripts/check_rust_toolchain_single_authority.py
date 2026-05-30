#!/usr/bin/env python3
"""P2 single-authority check for the pinned Rust toolchain."""

from __future__ import annotations

import re
import sys
from pathlib import Path

from ci_process import gha_error, repo_root


SETUP_ACTION = "actions-rust-lang/setup-rust-toolchain"


def parse_channel(toolchain_toml: Path) -> str | None:
    for line in toolchain_toml.read_text(encoding="utf-8").splitlines():
        match = re.match(r'^\s*channel\s*=\s*"([^"]*)"', line)
        if match:
            return match.group(1)
    return None


def step_span(lines: list[str], step_start: int) -> tuple[int, int]:
    match = re.match(r"^(\s*)-\s", lines[step_start])
    if not match:
        return step_start, min(step_start + 1, len(lines))
    base = len(match.group(1))
    idx = step_start + 1
    while idx < len(lines):
        next_step = re.match(r"^(\s*)-\s", lines[idx])
        if next_step is not None and len(next_step.group(1)) == base:
            break
        idx += 1
    return step_start, idx


def setup_toolchain_input_violation(path: Path) -> tuple[int, str] | None:
    lines = path.read_text(encoding="utf-8").splitlines()
    for idx, line in enumerate(lines):
        match = re.match(r"^(\s+)toolchain\s*:", line)
        if not match:
            continue
        toolchain_indent = len(match.group(1))
        step_start = idx - 1
        while step_start >= 0:
            step_match = re.match(r"^(\s*)-\s", lines[step_start])
            if step_match is not None and len(step_match.group(1)) < toolchain_indent:
                break
            step_start -= 1
        if step_start < 0:
            continue
        _, step_end = step_span(lines, step_start)
        if SETUP_ACTION in "\n".join(lines[step_start:step_end]):
            return idx + 1, line.strip()
    return None


def main() -> int:
    root = repo_root()
    rustup_dag = root / "dsl/extdeps/rustup.dag"
    toolchain_toml = root / "rust-toolchain.toml"
    workflows_dir = root / ".github/workflows"

    for path in [rustup_dag, toolchain_toml]:
        if not path.is_file():
            gha_error(f"missing {path}")
            return 2
    if not workflows_dir.is_dir():
        gha_error(f"missing {workflows_dir}")
        return 2

    channel = parse_channel(toolchain_toml)
    if not channel:
        gha_error("rust-toolchain.toml must contain a quoted [toolchain].channel line")
        return 1

    rustup_text = rustup_dag.read_text(encoding="utf-8")
    quoted_channel = f'"{channel}"'
    if quoted_channel in rustup_text:
        gha_error(
            f"dsl/extdeps/rustup.dag contains the pinned channel literal {quoted_channel} - "
            "duplicate authority (keep the channel only in rust-toolchain.toml)."
        )
        return 1
    if re.match(r"^[0-9]+\.[0-9]+", channel) and channel in rustup_text:
        gha_error(
            f"dsl/extdeps/rustup.dag contains bare channel token '{channel}' - "
            "duplicate authority (keep the channel only in rust-toolchain.toml)."
        )
        return 1
    if re.search(r"(?m)^\s*data\s+ci_pinned_toolchain", rustup_text):
        gha_error("dsl/extdeps/rustup.dag declares ci_pinned_toolchain - retired duplicate authority symbol. Use rust-toolchain.toml only.")
        return 1

    workflow_files = sorted(workflows_dir.glob("*.yml")) + sorted(workflows_dir.glob("*.yaml"))
    if not workflow_files:
        gha_error(f"no *.yml or *.yaml under {workflows_dir}")
        return 2
    for workflow in workflow_files:
        hit = setup_toolchain_input_violation(workflow)
        if hit is None:
            continue
        lineno, preview = hit
        rel = workflow.relative_to(root).as_posix()
        print(
            f"::error file={rel},line={lineno}::explicit `toolchain:` input on {SETUP_ACTION} - "
            "rust-toolchain.toml would be ignored. Remove it from that step's `with:`.",
            file=sys.stderr,
        )
        print(f"{rel}:{lineno}: {preview}", file=sys.stderr)
        return 1

    print(f"Rust toolchain single-authority check OK (channel={channel}; rustup.dag + workflow guard).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
