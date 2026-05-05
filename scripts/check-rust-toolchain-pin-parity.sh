#!/usr/bin/env bash
# Fail if rust-toolchain.toml `channel` diverges from `ci_pinned_toolchain`
# in dsl/extdeps/rustup.dag (single bump surface for the documented CI pin).
#
# Usage: ./scripts/check-rust-toolchain-pin-parity.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

channel=""
while IFS= read -r line || [[ -n "$line" ]]; do
  if [[ "$line" =~ ^channel[[:space:]]*=[[:space:]]*\"([^\"]+)\" ]]; then
    channel="${BASH_REMATCH[1]}"
    break
  fi
done <rust-toolchain.toml

dag_pin=""
while IFS= read -r line || [[ -n "$line" ]]; do
  if [[ "$line" =~ ^data[[:space:]]+ci_pinned_toolchain:.*=[[:space:]]*\"([^\"]+)\" ]]; then
    dag_pin="${BASH_REMATCH[1]}"
    break
  fi
done <dsl/extdeps/rustup.dag

if [[ -z "$channel" ]]; then
  echo "error: could not parse channel from rust-toolchain.toml" >&2
  exit 1
fi
if [[ -z "$dag_pin" ]]; then
  echo "error: could not parse ci_pinned_toolchain from dsl/extdeps/rustup.dag" >&2
  exit 1
fi
if [[ "$channel" != "$dag_pin" ]]; then
  echo "error: rust-toolchain.toml channel ($channel) != ci_pinned_toolchain ($dag_pin) in dsl/extdeps/rustup.dag" >&2
  exit 1
fi

echo "check-rust-toolchain-pin-parity: ok"
