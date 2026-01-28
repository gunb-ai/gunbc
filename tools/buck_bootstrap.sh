#!/usr/bin/env bash
set -euo pipefail

ROOT="$PWD"

resolve_cargo_cmd() {
  local toolchain="${BUCK_RUSTUP_TOOLCHAIN:-${RUSTUP_TOOLCHAIN:-}}"

  if [[ -n "$toolchain" ]]; then
    if command -v rustup >/dev/null 2>&1; then
      if rustup toolchain list | grep -q "^${toolchain}"; then
        echo "cargo +${toolchain}"
        return 0
      fi
    fi
    echo "rustup toolchain '${toolchain}' not installed. Run: rustup toolchain install ${toolchain}" >&2
    exit 2
  fi

  if command -v rustup >/dev/null 2>&1; then
    if rustup show active-toolchain >/dev/null 2>&1; then
      echo "cargo"
      return 0
    fi
    if rustup toolchain list | grep -q "^stable"; then
      echo "cargo +stable"
      return 0
    fi
    echo "no default rustup toolchain. Run: rustup default stable" >&2
    exit 2
  fi

  if command -v cargo >/dev/null 2>&1; then
    echo "cargo"
    return 0
  fi

  echo "cargo not found. Install Rust toolchain or set BUCK_RUSTUP_TOOLCHAIN." >&2
  exit 2
}

CARGO_HOME="${CARGO_HOME:-$ROOT/buck-out/cargo-home}"
RUSTUP_HOME="${RUSTUP_HOME:-$ROOT/buck-out/rustup-home}"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/buck-out/cargo-target}"
export CARGO_HOME RUSTUP_HOME CARGO_TARGET_DIR

mkdir -p "$CARGO_HOME" "$RUSTUP_HOME" "$CARGO_TARGET_DIR"

CARGO_CMD=$(resolve_cargo_cmd)
eval "$CARGO_CMD" run -p gunbc-deps -- --entry buck_bootstrap --mode upsert
