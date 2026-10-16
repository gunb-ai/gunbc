#!/usr/bin/env bash
set -euo pipefail
export GUNBC_GAP_ACCOUNT=1
export GUNBC_BIND_MEMORY_CGROUP_BYTES="${GUNBC_BIND_MEMORY_CGROUP_BYTES:-17179869184}"
export GITHUB_ACTIONS=1
ROOT="$(pwd)"
echo "START $(date -u +%T) ROOT=$ROOT"
cd "$ROOT/src/v1/stage0"
cargo build --offline --bin gunbc
echo "BUILT $(date -u +%T)"
cd "$ROOT"
BIN="$ROOT/target/debug/gunbc"
if [ ! -x "$BIN" ]; then
  BIN=$(find "$ROOT" -name gunbc -type f | head -1)
fi
ls -l "$BIN"
for d in 1 2 3 4 5 6; do
  echo "===== AFTER depth $d ====="
  set +e
  "$BIN" compile --source-root dag --entry "dag/test/gap_depth_${d}.dag" --output-dir "/tmp/oa${d}"
  echo "AFTER_DEPTH_STATUS $d $?"
  set -e
done
cat > dag/test/gap_right.dag <<'END'
module test.gap_right
import std.types { Int }
fn gar_app<T, R>(f: fn(T) -> R, x: T) -> R { f(x) }
fn probe() -> Int { gar_app(f: fn(v) { v }, x: 1) }
END
echo "===== RIGHT_BOUND TINY ====="
set +e
"$BIN" compile --source-root dag --entry dag/test/gap_right.dag --output-dir /tmp/or
echo "RIGHT_STATUS $?"
set -e
cat > dag/test/gap_inner_red.dag <<'END'
module test.gap_inner_red
import std.types { Bool, Int }
fn gar_id<T>(x: T) -> T { x }
fn gar_app<T, R>(f: fn(T) -> R, x: T) -> R { f(x) }
fn probe() -> Bool { gar_app(f: fn(v) { gar_id(x: v) }, x: 1) }
END
echo "===== INNER RED TINY ====="
set +e
"$BIN" compile --source-root dag --entry dag/test/gap_inner_red.dag --output-dir /tmp/ored
echo "INNER_RED_STATUS $?"
set -e
echo "DONE $(date -u +%T)"
