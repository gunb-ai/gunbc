#!/usr/bin/env bash
#
# P2 single-authority: the pinned rustc channel string must not exist in two
# independent editable places. Authoritative channel is `rust-toolchain.toml`
# `[toolchain].channel` only. `dsl/extdeps/rustup.dag` documents rustup
# installer behavior and must not reintroduce a parallel `ci_pinned_toolchain`
# data declaration (fail closed — see PR #1794 / INVARIANTS P2).
#
# Dissolution: delete this script and its CI step if extdeps gains a generated
# conformance link to rust-toolchain.toml instead of a negative guard.

set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)
rustup_dag="$repo_root/dsl/extdeps/rustup.dag"

if [ ! -r "$rustup_dag" ]; then
  echo "::error::missing $rustup_dag"
  exit 2
fi

if grep -Eq '^[[:space:]]*data[[:space:]]+ci_pinned_toolchain' "$rustup_dag"; then
  echo "::error::dsl/extdeps/rustup.dag declares ci_pinned_toolchain — duplicate channel authority. Use rust-toolchain.toml only."
  exit 1
fi

toolchain_toml="$repo_root/rust-toolchain.toml"
if ! grep -Eq '^[[:space:]]*channel[[:space:]]*=[[:space:]]*"' "$toolchain_toml"; then
  echo "::error::rust-toolchain.toml must contain a quoted [toolchain].channel line"
  exit 1
fi

echo "Rust toolchain single-authority check OK (no ci_pinned_toolchain in rustup.dag; rust-toolchain.toml present)."
