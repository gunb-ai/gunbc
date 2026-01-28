#!/usr/bin/env bash
set -euo pipefail

ROOT="$PWD"
VENDOR_DIR="$ROOT/vendor"

if [[ ! -d "$VENDOR_DIR" ]]; then
  echo "vendor/ not found. Run: tools/buck_bootstrap.sh" >&2
  exit 2
fi

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

GEN_DIR="$ROOT/buck-out/gen"
mkdir -p "$GEN_DIR"

CARGO_HOME="$ROOT/buck-out/cargo-home"
RUSTUP_HOME="$ROOT/buck-out/rustup-home"
CARGO_TARGET_DIR="$ROOT/buck-out/cargo-target"
mkdir -p "$CARGO_HOME" "$RUSTUP_HOME" "$CARGO_TARGET_DIR"

cat > "$CARGO_HOME/config.toml" <<CONFIG
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "$VENDOR_DIR"
CONFIG

export CARGO_HOME
export RUSTUP_HOME
export CARGO_TARGET_DIR
export CARGO_NET_OFFLINE=true

CARGO_CMD=$(resolve_cargo_cmd)
eval "$CARGO_CMD" run -p gunbc-deps -- --entry buck_test --mode check
eval "$CARGO_CMD" run -p gunbc-testgen -- --out "$GEN_DIR/generated_tests.rs"
GUNBC_GENERATED_TESTS_DIR="$GEN_DIR" eval "$CARGO_CMD" test --workspace --offline --locked
