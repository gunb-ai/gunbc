#!/usr/bin/env bash
# CHARAT-0: compare parse_json survival slope post vs origin/main at small sizes.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

POST_DIR="$ROOT"
PRE_DIR="/tmp/charat0-main-measure"
SIZES=(40000 80000)
TIMEOUT_SEC=600

git fetch origin main --depth=1
MAIN_REF="origin/main"

echo "# charat0_scaling head=$(git rev-parse HEAD)"
echo "# charat0_scaling main=$(git rev-parse "$MAIN_REF")"

cargo build --release -p v1-compiler --bin json_parse_scaling_probe
POST_BIN="$ROOT/target/release/json_parse_scaling_probe"

rm -rf "$PRE_DIR"
git worktree add --detach "$PRE_DIR" "$MAIN_REF"
PROBE_SRC="$ROOT/src/v1/stage0/src/bin/json_parse_scaling_probe.rs"
cp "$PROBE_SRC" "$PRE_DIR/src/v1/stage0/src/bin/json_parse_scaling_probe.rs"
if ! grep -q json_parse_scaling_probe "$PRE_DIR/src/v1/stage0/Cargo.toml"; then
  sed -i '/name = "claim_executor"/,/path = "src\/bin\/claim_executor.rs"/a\
\
# CHARAT-0 measurement scaffold (not floor-enrolled).\
[[bin]]\
name = "json_parse_scaling_probe"\
path = "src/bin/json_parse_scaling_probe.rs"' "$PRE_DIR/src/v1/stage0/Cargo.toml"
fi
(
  cd "$PRE_DIR"
  CARGO_TARGET_DIR="$PRE_DIR/target" cargo build --release -p v1-compiler --bin json_parse_scaling_probe
)
PRE_BIN="$PRE_DIR/target/release/json_parse_scaling_probe"

run_one() {
  local label="$1"
  local bin="$2"
  local size="$3"
  echo "=== $label target_bytes=$size (fresh process) ==="
  timeout "$TIMEOUT_SEC" env JSON_PARSE_PROBE_MODE=survival JSON_PARSE_TARGET_BYTES="$size" "$bin" || echo "exit=$?"
}

echo "label\ttarget_bytes\telapsed_ms\tmembers_found\toutcome"
for size in "${SIZES[@]}"; do
  run_one "pre-main" "$PRE_BIN" "$size"
  run_one "post-charat" "$POST_BIN" "$size"
done
