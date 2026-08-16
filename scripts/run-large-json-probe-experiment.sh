#!/usr/bin/env bash
# STR-RC-0 large-regime experiment: pre (String) vs post (Rc<str>).
# Run from a CLEAN committed tree (ctrl-build overlays local diffs onto fetch base).
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$ROOT" ]; then
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi
cd "$ROOT"

POST_DIR="$ROOT"
PRE_DIR="/tmp/str-rc0-pre-build"
RESULTS_DIR="$ROOT/docs/probes/results"
mkdir -p "$RESULTS_DIR"

MEM_LIMIT_KB=8000000
TIMEOUT_SEC=900

echo "=== Dispatch metadata (quote this in receipts, not the fetch line alone) ==="
echo "# dispatch_head=$(git rev-parse HEAD)"
echo "# dispatch_pushed=$(git rev-parse @{u} 2>/dev/null || echo unpushed)"
echo "# dispatch_dirty=$(git status --porcelain | wc -l | tr -d ' ') files"
git status --porcelain | sed 's/^/# dirty: /' || true

verify_str_carrier() {
  local label="$1"
  local file="$2"
  local expect="$3"
  local count
  count="$(grep -c 'Str(Rc<str>)' "$file" || true)"
  echo "# verify_${label}_str_rc_count=$count (expect $expect)"
  if [ "$count" -ne "$expect" ]; then
    echo "FATAL: $label interpreter has Str(Rc<str>) count=$count, expected $expect — discard run" >&2
    exit 1
  fi
}

# --- Build POST (current branch) ---
echo "=== Building POST (Rc<str>) probe ==="
verify_str_carrier post "$POST_DIR/src/v1/stage0/src/v1_interpreter.rs" 1
cargo build --release -p v1-compiler --bin json_parse_scaling_probe
POST_BIN="$ROOT/target/release/json_parse_scaling_probe"

# --- Build PRE from isolated clean worktree (no overlay from POST tree) ---
echo "=== Preparing PRE (String) worktree ==="
rm -rf "$PRE_DIR"
if ! git rev-parse --verify origin/main >/dev/null 2>&1; then
  git fetch --depth=1 origin main
fi
PRE_REF="origin/main"
echo "# pre_ref=$PRE_REF pre_sha=$(git rev-parse "$PRE_REF")"
git worktree add --detach "$PRE_DIR" "$PRE_REF"
verify_str_carrier pre "$PRE_DIR/src/v1/stage0/src/v1_interpreter.rs" 0

PROBE_SRC="$ROOT/src/v1/stage0/src/bin/json_parse_scaling_probe.rs"
PROBE_DST="$PRE_DIR/src/v1/stage0/src/bin/json_parse_scaling_probe.rs"
mkdir -p "$(dirname "$PROBE_DST")"
cp "$PROBE_SRC" "$PROBE_DST"

# Adapt str_value for Value::Str(String) on pre-migration tree.
sed -i 's/Value::Str(std::rc::Rc::from(s.as_ref()))/Value::Str(s.as_ref().to_string())/' "$PROBE_DST"

if ! grep -q json_parse_scaling_probe "$PRE_DIR/src/v1/stage0/Cargo.toml"; then
  cat >>"$PRE_DIR/src/v1/stage0/Cargo.toml" <<'EOF'

# SCAFFOLD (STR-RC-0 pre-measurement transport; not floor-enrolled).
[[bin]]
name = "json_parse_scaling_probe"
path = "src/bin/json_parse_scaling_probe.rs"
EOF
fi

echo "=== Building PRE (String) probe ==="
(
  cd "$PRE_DIR"
  CARGO_TARGET_DIR="$PRE_DIR/target" cargo build --release -p v1-compiler --bin json_parse_scaling_probe
)
PRE_BIN="$PRE_DIR/target/release/json_parse_scaling_probe"

run_probe_bounded() {
  local label="$1"
  local bin="$2"
  local outfile="$3"
  echo "=== Running $label (large mode, ulimit -v $MEM_LIMIT_KB, timeout ${TIMEOUT_SEC}s) ==="
  {
    echo "# label=$label"
    echo "# host=$(uname -a)"
    echo "# date=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "# mem_limit_kb=$MEM_LIMIT_KB timeout_sec=$TIMEOUT_SEC"
    echo "# command: JSON_PARSE_PROBE_MODE=large $bin"
    echo "#"
    set +e
    (
      ulimit -v "$MEM_LIMIT_KB"
      JSON_PARSE_PROBE_MODE=large timeout "$TIMEOUT_SEC" "$bin" &
      pid=$!
      peak_kb=0
      while kill -0 "$pid" 2>/dev/null; do
        if [ -r "/proc/$pid/status" ]; then
          hwm="$(awk '/^VmHWM:/ {print $2}' "/proc/$pid/status" 2>/dev/null || true)"
          if [ -n "$hwm" ] && [ "$hwm" -gt "$peak_kb" ]; then
            peak_kb=$hwm
          fi
        fi
        sleep 0.2
      done
      wait "$pid"
      ec=$?
      echo "# peak_rss_kb=$peak_kb"
      exit "$ec"
    )
    ec=$?
    set -e
    echo "# exit_code=$ec"
    case "$ec" in
      0) echo "# outcome=completed" ;;
      137|134|9) echo "# outcome=killed (OOM or signal)" ;;
      124) echo "# outcome=timeout" ;;
      *) echo "# outcome=failed" ;;
    esac
  } | tee "$outfile"
}

TS="$(date -u +%Y%m%dT%H%M%SZ)"
run_probe_bounded "pre-string" "$PRE_BIN" "$RESULTS_DIR/large-pre-string-${TS}.txt"
run_probe_bounded "post-rc-str" "$POST_BIN" "$RESULTS_DIR/large-post-rc-str-${TS}.txt"

# Summary TSV for PR body / receipts.
SUMMARY="$RESULTS_DIR/large-regime-summary-${TS}.tsv"
{
  echo -e "label\ttarget_bytes\tactual_bytes\toutcome\telapsed_ms\texit_code\tpeak_rss_kb"
  for f in "$RESULTS_DIR/large-pre-string-${TS}.txt" "$RESULTS_DIR/large-post-rc-str-${TS}.txt"; do
    label="$(grep '^# label=' "$f" | cut -d= -f2)"
    exit_code="$(grep '^# exit_code=' "$f" | cut -d= -f2)"
    peak="$(grep '^# peak_rss_kb=' "$f" | cut -d= -f2)"
    grep -E '^[0-9]+\t' "$f" | while IFS=$'\t' read -r target members actual outcome elapsed; do
      echo -e "${label}\t${target}\t${actual}\t${outcome}\t${elapsed}\t${exit_code}\t${peak}"
    done
  done
} >"$SUMMARY"

echo "=== Results ==="
ls -la "$RESULTS_DIR"/large-*-"${TS}".*
cat "$SUMMARY"
