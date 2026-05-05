#!/usr/bin/env bash
# Fail closed if rust-toolchain.toml [toolchain].channel drifts from
# dsl/extdeps/rustup.dag `ci_pinned_toolchain` while both remain hand-authored
# (ROADMAP P2 / rust_toolchain_toml_codegen_from_rustup_dag).
set -euo pipefail
repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"

toml_channel=$(
  grep -E '^[[:space:]]*channel[[:space:]]*=' rust-toolchain.toml |
    head -1 |
    sed -E 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/'
)
dag_channel=$(
  grep -E '^data ci_pinned_toolchain' dsl/extdeps/rustup.dag |
    head -1 |
    sed -E 's/^data ci_pinned_toolchain:[^=]*=[[:space:]]*"([^"]+)".*/\1/'
)

if [ -z "${toml_channel}" ] || [ -z "${dag_channel}" ]; then
  echo "::error::could not parse channel from rust-toolchain.toml and/or ci_pinned_toolchain from dsl/extdeps/rustup.dag"
  exit 1
fi

if [ "${toml_channel}" != "${dag_channel}" ]; then
  echo "::error::Rust toolchain pin mismatch: rust-toolchain.toml channel='${toml_channel}' vs dsl/extdeps/rustup.dag ci_pinned_toolchain='${dag_channel}'. Align both until rust_toolchain_toml_codegen_from_rustup_dag dissolves the duplicate (ROADMAP.md P2)."
  exit 1
fi

echo "Rust toolchain pin alignment OK (${toml_channel})."
