#!/usr/bin/env bash
#
# P2 single-authority: the pinned rustc channel string lives only in
# `rust-toolchain.toml` `[toolchain].channel`. `dsl/extdeps/rustup.dag` must not
# reintroduce that value as a quoted literal, as a bare semver-like token (catches
# unquoted comment drift such as `// pin 1.93.0`), nor via the retired
# `ci_pinned_toolchain` symbol — see PR #1794 / INVARIANTS P2. Word channels (e.g.
# `stable`) are only checked in quoted form to avoid unrelated prose false positives.
#
# Also fail if any `.github/workflows/*.{yml,yaml}` pairs `toolchain:` with
# `actions-rust-lang/setup-rust-toolchain` in the same step (the action ignores
# rust-toolchain.toml when that input is present — same authority drift class).
#
# Dissolution: delete this script and its CI step if extdeps + workflow toolchain
# selection are generated or schema-checked so this shell guard is redundant.

set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)
rustup_dag="$repo_root/dsl/extdeps/rustup.dag"
toolchain_toml="$repo_root/rust-toolchain.toml"
workflows_dir="$repo_root/.github/workflows"

if [ ! -r "$rustup_dag" ]; then
  echo "::error::missing $rustup_dag"
  exit 2
fi

if [ ! -r "$toolchain_toml" ]; then
  echo "::error::missing $toolchain_toml"
  exit 2
fi

if [ ! -d "$workflows_dir" ]; then
  echo "::error::missing $workflows_dir"
  exit 2
fi

if ! grep -Eq '^[[:space:]]*channel[[:space:]]*=[[:space:]]*"' "$toolchain_toml"; then
  echo "::error::rust-toolchain.toml must contain a quoted [toolchain].channel line"
  exit 1
fi

channel=$(
  sed -n 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$toolchain_toml" | head -n1
)
if [ -z "$channel" ]; then
  echo "::error::could not parse [toolchain].channel from rust-toolchain.toml"
  exit 1
fi

quoted_channel="\"${channel}\""
if grep -Fq "$quoted_channel" "$rustup_dag"; then
  echo "::error::dsl/extdeps/rustup.dag contains the pinned channel literal ${quoted_channel} — duplicate authority (keep the channel only in rust-toolchain.toml)."
  exit 1
fi

# Semver-like channels: also reject the bare token so unquoted comment/data drift
# cannot reintroduce the pin (e.g. `// use 1.93.0`). Skipped for word channels like
# `stable` where this substring can appear in unrelated prose.
if [[ "$channel" =~ ^[0-9]+\.[0-9]+ ]]; then
  if grep -Fq "$channel" "$rustup_dag"; then
    echo "::error::dsl/extdeps/rustup.dag contains bare channel token '${channel}' — duplicate authority (keep the channel only in rust-toolchain.toml)."
    exit 1
  fi
fi

if grep -Eq '^[[:space:]]*data[[:space:]]+ci_pinned_toolchain' "$rustup_dag"; then
  echo "::error::dsl/extdeps/rustup.dag declares ci_pinned_toolchain — retired duplicate authority symbol. Use rust-toolchain.toml only."
  exit 1
fi

# Indented YAML key `toolchain:` under `with:` for `actions-rust-lang/setup-rust-toolchain`
# ignores rust-toolchain.toml — forbid that pairing only (not unrelated `toolchain:` keys
# in other actions). Python scan walks each `toolchain:` line up to its Actions step head,
# then scans the **full** step (through the next sibling `-` at the same list indent) so a
# pathological `with.toolchain` **before** `uses: …/setup-rust-toolchain` cannot evade the check.
if ! command -v python3 >/dev/null 2>&1; then
  echo "::error::python3 is required for workflow toolchain guard (setup-rust-toolchain scope)"
  exit 2
fi

python3 - "$workflows_dir" "$repo_root" <<'PY'
import pathlib
import re
import sys

SETUP = "actions-rust-lang/setup-rust-toolchain"


def _step_span(lines, step_start):
    """Return [step_start, end) line indices for one GitHub Actions `steps:` list item."""
    m0 = re.match(r"^(\s*)-\s", lines[step_start])
    if not m0:
        return step_start, min(step_start + 1, len(lines))
    base = len(m0.group(1))
    k = step_start + 1
    while k < len(lines):
        m = re.match(r"^(\s*)-\s", lines[k])
        if m is not None and len(m.group(1)) == base:
            break
        k += 1
    return step_start, k


def violation_in_file(wf_path):
    lines = wf_path.read_text(encoding="utf-8").splitlines()
    for i, line in enumerate(lines):
        m_tc = re.match(r"^(\s+)toolchain\s*:", line)
        if not m_tc:
            continue
        tc_ws = len(m_tc.group(1))
        j = i - 1
        while j >= 0:
            m_dash = re.match(r"^(\s*)-\s", lines[j])
            if m_dash is not None and len(m_dash.group(1)) < tc_ws:
                break
            j -= 1
        if j < 0:
            continue
        _, k = _step_span(lines, j)
        block = "\n".join(lines[j:k])
        if SETUP in block:
            return i + 1, line.strip()
    return None


def main():
    workflows_dir = pathlib.Path(sys.argv[1])
    repo_root = pathlib.Path(sys.argv[2])
    files = sorted(workflows_dir.glob("*.yml")) + sorted(workflows_dir.glob("*.yaml"))
    if not files:
        print(f"::error::no *.yml or *.yaml under {workflows_dir}")
        return 2
    for wf in files:
        hit = violation_in_file(wf)
        if hit is not None:
            lineno, preview = hit
            rel = wf.resolve().relative_to(repo_root.resolve())
            print(
                f"::error::file={rel},line={lineno}::explicit `toolchain:` input on "
                f"{SETUP} — rust-toolchain.toml would be ignored. Remove it from that step's `with:`."
            )
            print(f"{rel}:{lineno}: {preview}")
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
PY

echo "Rust toolchain single-authority check OK (channel=${channel}; rustup.dag + workflow guard)."
