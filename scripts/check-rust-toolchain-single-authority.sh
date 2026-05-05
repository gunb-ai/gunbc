#!/usr/bin/env bash
#
# P2 single-authority: the pinned rustc channel string lives only in
# `rust-toolchain.toml` `[toolchain].channel`. `dsl/extdeps/rustup.dag` must not
# reintroduce that value as a quoted literal, as a bare semver-like token (catches
# unquoted comment drift such as `// pin 1.93.0`), nor via the retired
# `ci_pinned_toolchain` symbol — see PR #1794 / INVARIANTS P2. Word channels (e.g.
# `stable`) are only checked in quoted form to avoid unrelated prose false positives.
#
# Also fail if `.github/workflows/ci.yml` sets an explicit `toolchain:` input on
# `actions-rust-lang/setup-rust-toolchain` (the action ignores rust-toolchain.toml
# when that input is present — same authority drift class).
#
# Dissolution: delete this script and its CI step if extdeps + workflow toolchain
# selection are generated or schema-checked so this shell guard is redundant.

set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)
rustup_dag="$repo_root/dsl/extdeps/rustup.dag"
toolchain_toml="$repo_root/rust-toolchain.toml"
ci_yml="$repo_root/.github/workflows/ci.yml"

if [ ! -r "$rustup_dag" ]; then
  echo "::error::missing $rustup_dag"
  exit 2
fi

if [ ! -r "$toolchain_toml" ]; then
  echo "::error::missing $toolchain_toml"
  exit 2
fi

if [ ! -r "$ci_yml" ]; then
  echo "::error::missing $ci_yml"
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

# Indented YAML key `toolchain:` under a `with:` block would make setup-rust-toolchain
# ignore rust-toolchain.toml — forbid it (comments must not fake this shape at BOL).
if grep -Eq '^[[:space:]]+toolchain[[:space:]]*:' "$ci_yml"; then
  echo "::error::.github/workflows/ci.yml contains an explicit \`toolchain:\` input — rust-toolchain.toml would be ignored. Remove it from setup-rust-toolchain steps."
  exit 1
fi

echo "Rust toolchain single-authority check OK (channel=${channel}; rustup.dag + ci.yml guard)."
