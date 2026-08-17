#!/usr/bin/env bash
# SCAFFOLD — CHARAT-0 parse_json survival slope comparison beside json_parse_scaling_probe
# (not a general scripts/ home; same transport class as docs/probes/run_frontier_probe_survey_per_module.sh).
# dissolve-on: delete when CHARAT-0 string-indexing acceptance enrolls a floor witness,
# STR-RC-0 scaffold retires, or large-regime measurement refutes and the branch reverts.
# Runtime-present: invokes json_parse_scaling_probe seed bin with JSON_PARSE_PROBE_MODE=survival
# per (carrier, size) in a fresh process; compares origin/main interpreter vs dispatch HEAD.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

PRE_DIR="/tmp/charat0-main-measure"
SIZES=(20000 40000 80000)
TIMEOUT_SEC=600

termination_label() {
  local ec="$1"
  case "$ec" in
    0) echo "completed" ;;
    2) echo "parse_failed" ;;
    137|134|9) echo "killed (OOM or signal)" ;;
    124) echo "timeout" ;;
    *) echo "failed" ;;
  esac
}

git fetch origin main --depth=1
MAIN_REF="origin/main"

echo "# charat0_scaling head=$(git rev-parse HEAD)"
echo "# charat0_scaling main=$(git rev-parse "$MAIN_REF")"

cargo build --release -p v1-compiler --bin json_parse_scaling_probe
POST_BIN="$ROOT/target/release/json_parse_scaling_probe"

rm -rf "$PRE_DIR"
git worktree remove --force "$PRE_DIR" 2>/dev/null || true
git worktree prune 2>/dev/null || true
git worktree add --detach "$PRE_DIR" "$MAIN_REF"
PROBE_SRC="$ROOT/src/v1/stage0/src/bin/json_parse_scaling_probe.rs"
cp "$PROBE_SRC" "$PRE_DIR/src/v1/stage0/src/bin/json_parse_scaling_probe.rs"
if ! grep -q json_parse_scaling_probe "$PRE_DIR/src/v1/stage0/Cargo.toml"; then
  cat >>"$PRE_DIR/src/v1/stage0/Cargo.toml" <<'EOF'

# SCAFFOLD — CHARAT-0 / STR-RC-0 measurement transport (not floor-enrolled).
# dissolve-on: enrolled witness or refuted hypothesis (see json_parse_scaling_probe.rs marker).
[[bin]]
name = "json_parse_scaling_probe"
path = "src/bin/json_parse_scaling_probe.rs"
EOF
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
  set +e
  timeout "$TIMEOUT_SEC" env \
    JSON_PARSE_PROBE_MODE=survival \
    JSON_PARSE_TARGET_BYTES="$size" \
    "$bin"
  local ec=$?
  set -e
  echo "# exit_code=$ec"
  echo "# termination=$(termination_label "$ec")"
  if [ "$ec" -ne 0 ]; then
    echo "probe_refused label=$label target_bytes=$size exit_code=$ec termination=$(termination_label "$ec")" >&2
    exit "$ec"
  fi
}

echo "label\ttarget_bytes\telapsed_ms\tmembers_found\toutcome"
for size in "${SIZES[@]}"; do
  run_one "pre-main" "$PRE_BIN" "$size"
  run_one "post-charat" "$POST_BIN" "$size"
done
