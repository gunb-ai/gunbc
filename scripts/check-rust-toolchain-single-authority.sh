#!/usr/bin/env bash
# P2 single-authority: the pinned Rust channel + components live only in repo-root
# rust-toolchain.toml (read by rustup locally and by setup-rust-toolchain in CI).
# dsl/extdeps/rustup.dag documents rustup; it must not reintroduce a parallel
# numeric channel constant. GitHub workflows must not reintroduce parallel
# `toolchain:` / `components:` inputs on setup-rust-toolchain (same authority).
#
# Usage: ./scripts/check-rust-toolchain-single-authority.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

rustup_dag="dsl/extdeps/rustup.dag"
toolchain_toml="rust-toolchain.toml"

if [[ ! -f "$rustup_dag" ]] || [[ ! -f "$toolchain_toml" ]]; then
  echo "error: missing $rustup_dag or $toolchain_toml" >&2
  exit 1
fi

if grep -n 'ci_pinned_toolchain' "$rustup_dag" 2>/dev/null; then
  echo "error: ci_pinned_toolchain must not exist in $rustup_dag (sole pin: $toolchain_toml)" >&2
  exit 1
fi

if grep -nE '^\s*data\s+\w+\s*:\s*NonEmptyStr\s*=\s*"[0-9]+\.[0-9]+\.[0-9]+"' "$rustup_dag" 2>/dev/null; then
  echo "error: semver-like NonEmptyStr data in $rustup_dag — channel pin belongs only in $toolchain_toml" >&2
  exit 1
fi

if ! grep -qE '^channel\s*=' "$toolchain_toml"; then
  echo "error: $toolchain_toml missing channel = entry" >&2
  exit 1
fi

# Fail if any workflow reintroduces `toolchain:` / `components:` mapping keys
# (rust-toolchain.toml is sole authority). Keys are lines with leading
# whitespace then the token — excludes full-line `#` comments.
workflow_dir=".github/workflows"
if [[ -d "$workflow_dir" ]]; then
  while IFS= read -r -d '' wf; do
    if grep -nE '^[[:space:]]+toolchain:[[:space:]]' "$wf" 2>/dev/null; then
      echo "error: explicit toolchain: pin in $wf — use rust-toolchain.toml only" >&2
      exit 1
    fi
    if grep -nE '^[[:space:]]+components:[[:space:]]' "$wf" 2>/dev/null; then
      echo "error: explicit components: list in $wf — use rust-toolchain.toml only" >&2
      exit 1
    fi
  done < <(find "$workflow_dir" -maxdepth 1 -type f \( -name '*.yml' -o -name '*.yaml' \) -print0)
fi

echo "check-rust-toolchain-single-authority: ok"
