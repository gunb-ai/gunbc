#!/usr/bin/env bash
# P2 single-authority: the pinned Rust channel lives only in repo-root
# rust-toolchain.toml (read by rustup locally and by setup-rust-toolchain in CI).
# dsl/extdeps/rustup.dag documents rustup; it must not reintroduce a parallel
# numeric channel constant.
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

echo "check-rust-toolchain-single-authority: ok"
