#!/usr/bin/env bash
# Seed + gen-one + product emit only (1h BB cap). Other witnesses: invocation b3d5dce3.
set -euo pipefail
export TMPDIR="${TMPDIR:-/tmp}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
export RUSTC_WRAPPER="${RUSTC_WRAPPER:-}"
unset GUNBC_BIND_MEMORY_CGROUP_BYTES || true
unset GUNBC_MEMORY_ENVELOPE_REQUIRED || true
BOUND=$((22 * 1024 * 1024 * 1024))
LEAF=/sys/fs/cgroup/pr12.bound
mkdir -p "$LEAF"
echo '+memory' >/sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
printf '%s\n' "$BOUND" >"$LEAF/memory.max"
printf '%s\n' "$$" >"$LEAF/cgroup.procs"
REPO="$PWD"
cd "$REPO"
export GITHUB_SHA="$(git rev-parse HEAD)"
echo "GITHUB_SHA=$GITHUB_SHA LEAF_CGROUP=$(tr '\n' ' ' </proc/self/cgroup)"
cargo build --release -p v1-compiler --bin gunbc
GUNBC="$REPO/target/release/gunbc"
test -x "$GUNBC"
export GUNBC_NATIVE_PRODUCT_REUSE=fresh
mkdir -p "$REPO/target/pr12"
CLI_LOG="$REPO/target/pr12/v2-native-cli.log"
set +e
"$GUNBC" test //gunbc/instruments:v2-native-cli 2>&1 | tee "$CLI_LOG"
CLI=${PIPESTATUS[0]}
set -e
echo "TEST_RC label=//gunbc/instruments:v2-native-cli rc=$CLI"
GEN_ONE="$(sed -n 's/.*generation_one_executable=//p' "$CLI_LOG" | tail -n 1)"
echo "GEN_ONE=$GEN_ONE"
test -n "$GEN_ONE" && test -x "$GEN_ONE"
ERR="$REPO/target/pr12/source_root_eval.stderr"
set +e
"$GEN_ONE" emit --entry v2.compiler.source_root_eval --source-root dag --source-root src/v2 >/dev/null 2>"$ERR"
EMIT_RC=$?
set -e
echo "NATIVE_EMIT_ENTRY=v2.compiler.source_root_eval RC=$EMIT_RC"
echo "NATIVE_STDERR_BEGIN"
cat "$ERR"
echo "NATIVE_STDERR_END"
echo "SUMMARY v2-native-cli=$CLI source_root_eval_emit=$EMIT_RC"
exit 0
