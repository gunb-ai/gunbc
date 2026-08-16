#!/usr/bin/env bash
# STR-RC-0 clean-process survival experiment: pre (String) vs post (Rc<str>).
# One fresh process per (carrier, size) pair; cgroup memory.peak/events when available.
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

# 8 GiB cgroup cap (honest memory bound; not ulimit -v virtual address space).
MEM_LIMIT_BYTES=$((8 * 1024 * 1024 * 1024))
TIMEOUT_SEC=900
SURVIVAL_SIZES=(100000 200000 507000)

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

cgroup_create() {
  local id="$1"
  local path=""
  if [ -w /sys/fs/cgroup ] && [ -f /sys/fs/cgroup/cgroup.controllers ]; then
    path="/sys/fs/cgroup/str-rc0-${id}-$$"
    if mkdir -p "$path" 2>/dev/null; then
      if ! { echo "$MEM_LIMIT_BYTES"; } >"$path/memory.max" 2>/dev/null; then
        rmdir "$path" 2>/dev/null || true
        path=""
      fi
    else
      path=""
    fi
  fi
  echo "$path"
}

cgroup_read_peak_kb() {
  local path="$1"
  if [ -n "$path" ] && [ -r "$path/memory.peak" ]; then
    local peak
    peak="$(cat "$path/memory.peak" 2>/dev/null || echo "")"
    if [ -n "$peak" ] && [ "$peak" != "max" ]; then
      echo $((peak / 1024))
      return
    fi
  fi
  echo "unavailable"
}

cgroup_read_oom_kills() {
  local path="$1"
  if [ -n "$path" ] && [ -r "$path/memory.events" ]; then
    awk '/^oom_kill / {print $2; exit}' "$path/memory.events" 2>/dev/null || echo "unavailable"
    return
  fi
  echo "unavailable"
}

cgroup_cleanup() {
  local path="$1"
  if [ -n "$path" ] && [ -d "$path" ]; then
    rmdir "$path" 2>/dev/null || true
  fi
}

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

# Fresh process: one (carrier, size, eval_memo) invocation.
run_survival_process() {
  local label="$1"
  local bin="$2"
  local target_bytes="$3"
  local eval_memo="$4"
  local outfile="$5"

  local cgroup_id="${label}-${target_bytes}"
  local cgroup_path
  cgroup_path="$(cgroup_create "$cgroup_id")"

  echo "=== Running $label survival target=${target_bytes} GUNBC_EVAL_MEMO=${eval_memo} (fresh process) ==="
  {
    echo "# experiment=survival"
    echo "# label=$label"
    echo "# target_bytes=$target_bytes"
    echo "# GUNBC_EVAL_MEMO=$eval_memo"
    echo "# cgroup_path=${cgroup_path:-unavailable}"
    echo "# cgroup_memory_max_bytes=$MEM_LIMIT_BYTES"
    echo "# host=$(uname -a)"
    echo "# date=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "# timeout_sec=$TIMEOUT_SEC"
    echo "# command: JSON_PARSE_PROBE_MODE=survival JSON_PARSE_TARGET_BYTES=$target_bytes GUNBC_EVAL_MEMO=$eval_memo $bin"
    echo "#"

    set +e
    if [ -n "$cgroup_path" ]; then
      (
        echo $$ >"$cgroup_path/cgroup.procs" 2>/dev/null || true
        exec timeout "$TIMEOUT_SEC" env \
          GUNBC_EVAL_MEMO="$eval_memo" \
          JSON_PARSE_PROBE_MODE=survival \
          JSON_PARSE_TARGET_BYTES="$target_bytes" \
          "$bin"
      )
      ec=$?
    else
      timeout "$TIMEOUT_SEC" env \
        GUNBC_EVAL_MEMO="$eval_memo" \
        JSON_PARSE_PROBE_MODE=survival \
        JSON_PARSE_TARGET_BYTES="$target_bytes" \
        "$bin"
      ec=$?
    fi
    set -e

    echo "# exit_code=$ec"
    echo "# termination=$(termination_label "$ec")"
    echo "# cgroup_memory_peak_kb=$(cgroup_read_peak_kb "$cgroup_path")"
    echo "# cgroup_oom_kill_count=$(cgroup_read_oom_kills "$cgroup_path")"
    cgroup_cleanup "$cgroup_path"
  } | tee "$outfile"
  return 0
}

run_memo_receipt_process() {
  local label="$1"
  local bin="$2"
  local target_bytes="$3"
  local outfile="$4"

  echo "=== Running $label memo_receipt target=${target_bytes} (fresh process, memo default) ==="
  {
    echo "# experiment=memo_receipt"
    echo "# label=$label"
    echo "# target_bytes=$target_bytes"
    echo "# GUNBC_EVAL_MEMO=default"
    echo "# date=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "# command: JSON_PARSE_PROBE_MODE=memo_receipt JSON_PARSE_TARGET_BYTES=$target_bytes $bin"
    echo "#"
    set +e
    timeout "$TIMEOUT_SEC" env \
      JSON_PARSE_PROBE_MODE=memo_receipt \
      JSON_PARSE_TARGET_BYTES="$target_bytes" \
      "$bin"
    ec=$?
    set -e
    echo "# exit_code=$ec"
    echo "# termination=$(termination_label "$ec")"
  } | tee "$outfile"
  return 0
}

TS="$(date -u +%Y%m%dT%H%M%SZ)"

# PRODUCTION SURVIVAL: 6 fresh processes, memo at production default.
for size in "${SURVIVAL_SIZES[@]}"; do
  run_survival_process "pre-string" "$PRE_BIN" "$size" "default" \
    "$RESULTS_DIR/survival-pre-string-${size}-${TS}.txt"
  run_survival_process "post-rc-str" "$POST_BIN" "$size" "default" \
    "$RESULTS_DIR/survival-post-rc-str-${size}-${TS}.txt"
done

# MECHANISM: memo OFF at 100KB and 200KB for both carriers (diagnostic only).
for size in 100000 200000; do
  run_survival_process "pre-string-memo0" "$PRE_BIN" "$size" 0 \
    "$RESULTS_DIR/mechanism-pre-string-memo0-${size}-${TS}.txt"
  run_survival_process "post-rc-str-memo0" "$POST_BIN" "$size" 0 \
    "$RESULTS_DIR/mechanism-post-rc-str-memo0-${size}-${TS}.txt"
done

# MEMO RECEIPT: post only at 100KB (cold / first repeat / subsequent hits).
run_memo_receipt_process "post-rc-str" "$POST_BIN" 100000 \
  "$RESULTS_DIR/memo-receipt-post-rc-str-100000-${TS}.txt"

SUMMARY="$RESULTS_DIR/clean-process-summary-${TS}.tsv"
{
  echo -e "experiment\tlabel\tGUNBC_EVAL_MEMO\ttarget_bytes\tactual_bytes\toutcome\tmembers_found\telapsed_ms\texit_code\ttermination\tcgroup_memory_peak_kb\tcgroup_oom_kill_count"
  for f in "$RESULTS_DIR"/*-"${TS}".txt; do
    [ -f "$f" ] || continue
    experiment="$(grep '^# experiment=' "$f" | cut -d= -f2)"
    label="$(grep '^# label=' "$f" | cut -d= -f2)"
    eval_memo="$(grep '^# GUNBC_EVAL_MEMO=' "$f" | cut -d= -f2)"
    exit_code="$(grep '^# exit_code=' "$f" | cut -d= -f2)"
    termination="$(grep '^# termination=' "$f" | cut -d= -f2)"
    peak="$(grep '^# cgroup_memory_peak_kb=' "$f" | cut -d= -f2)"
    oom="$(grep '^# cgroup_oom_kill_count=' "$f" | cut -d= -f2)"
    if [ "$experiment" = "memo_receipt" ]; then
      grep -E '^[0-9]+\t' "$f" | while IFS=$'\t' read -r target members actual cold first avg outcome; do
        echo -e "${experiment}\t${label}\tdefault\t${target}\t${actual}\t${outcome}\tcold=${cold}\tfirst=${first}\tavg_hit=${avg}\t${exit_code}\t${termination}\t${peak}\t${oom}"
      done
    else
      grep -E '^[0-9]+\t' "$f" | while IFS=$'\t' read -r target members actual outcome members_found elapsed; do
        echo -e "${experiment}\t${label}\t${eval_memo}\t${target}\t${actual}\t${outcome}\t${members_found}\t${elapsed}\t${exit_code}\t${termination}\t${peak}\t${oom}"
      done
    fi
  done
} >"$SUMMARY"

echo "=== Results ==="
ls -la "$RESULTS_DIR"/*-"${TS}".*
echo ""
cat "$SUMMARY"
