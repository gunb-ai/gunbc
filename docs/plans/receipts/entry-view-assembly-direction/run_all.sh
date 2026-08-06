#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: entry-view assembly memo direction closed (measured no-go;
# docs/plans/entry-view-assembly-direction-receipt.md). Frozen cohort A/B reproduction
# runner only — not a CI gate. Per-arm TSV/stderr artifacts are the committed receipt;
# this script re-derives them when re-run is needed.
# dissolve-on: delete with this receipt directory when floor-prep-tax-program P1
# follow-on row retires, or when gunbc bash-emit realizes measurement orchestration
# through host_effect_apply (#5828) and hand bash under docs/ is no longer authored.
#
# Full A/B receipt: 50-entry cohort + representative affected floor.
# Run on one host, back to back. Intended for ctrl-build --remote when local
# cgroup cannot complete a release v1-compiler link.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
RECEIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PATCH="$RECEIPT_DIR/base-arm-revert.patch"
COHORT="$RECEIPT_DIR/cohort.tsv"
AFTER_COMMIT="$(git -C "$ROOT" rev-parse HEAD)"
BASE_BIN="$RECEIPT_DIR/claim_batch.base"
AFTER_BIN="$RECEIPT_DIR/claim_batch.after"
FLOOR_BIN="$RECEIPT_DIR/claim_executor.after"
FLOOR_BASE_BIN="$RECEIPT_DIR/claim_executor.base"
# Post-merge representative slice-2 cell (29 changed paths, ~316 selected entries).
FLOOR_DIFF_BASE="${FLOOR_DIFF_BASE:-0d6ffc4db9759ca0000adaec3b067ea3aec8361d}"
# Post-#7836 fill-composition after-arm envelope (kB) for bar comparison.
POST_7836_PEAK_RSS_KB="${POST_7836_PEAK_RSS_KB:-5803012}"

cd "$ROOT"

build_args() {
  local -a args=(--source-root dag --source-root src/v2)
  while IFS=$'\t' read -r ord entry func _; do
    [[ "$ord" == "ordinal" ]] && continue
    args+=(--entry "$entry" --functions "$func")
  done < "$COHORT"
}

write_arm_tsv() {
  local label=$1
  local bin_sha=$2
  local stderr_file=$3
  local stdout_file=$4
  local exit_code=$5
  local out_tsv="$RECEIPT_DIR/${label}.tsv"

  local wall_ms peak_rss_kb pass fail
  peak_rss_kb=$(grep 'Maximum resident set size' "$stderr_file" | awk '{print $6}' | tail -1)
  pass=$(grep -c '^PASS ' "$stdout_file" || true)
  fail=$(grep -c '^FAIL ' "$stdout_file" || true)

  wall_ms="unknown"
  local time_wall
  time_wall=$(grep -E '^Elapsed \(wall clock\) time \(h:mm:ss or m:ss\):' "$stderr_file" | tail -1 | sed -E 's/.*: //')
  if [[ -n "$time_wall" ]]; then
    if [[ "$time_wall" =~ ^([0-9]+):([0-9]{2}):([0-9.]+)$ ]]; then
      wall_ms=$(awk -v h="${BASH_REMATCH[1]}" -v m="${BASH_REMATCH[2]}" -v s="${BASH_REMATCH[3]}" 'BEGIN{printf "%.0f", (h*3600+m*60+s)*1000}')
    elif [[ "$time_wall" =~ ^([0-9]+):([0-9.]+)$ ]]; then
      wall_ms=$(awk -v m="${BASH_REMATCH[1]}" -v s="${BASH_REMATCH[2]}" 'BEGIN{printf "%.0f", (m*60+s)*1000}')
    fi
  fi

  local assembly_ms="unknown"
  if grep -q '^\[assembly-split\]' "$stderr_file"; then
    assembly_ms=$(python3 - "$stderr_file" <<'PY' || echo unknown
import re, sys
text = open(sys.argv[1]).read().splitlines()
line = next((l for l in text if l.startswith("[assembly-split]")), "")
if not line:
    raise SystemExit(1)
vals = [float(x[:-2]) for x in re.findall(r"=([0-9.]+)ms", line)]
print(f"{sum(vals):.1f}")
PY
)
  fi

  if [[ -z "$peak_rss_kb" || "$peak_rss_kb" == "unknown" ]]; then
    if grep -q 'per-shard-peak-rss' "$stderr_file"; then
      peak_rss_kb=$(python3 - "$stderr_file" <<'PY'
import re, sys
m = re.search(r"per-shard-peak-rss — ([0-9.]+) GiB", open(sys.argv[1]).read())
if m:
    print(int(float(m.group(1)) * 1024 * 1024))
PY
)
    fi
  fi

  {
    echo "label	$label"
    echo "binary_sha256	$bin_sha"
    echo "exit	$exit_code"
    echo "wall_ms	$wall_ms"
    echo "exclusive_assembly_ms	$assembly_ms"
    echo "peak_rss_kb	${peak_rss_kb:-unknown}"
    echo "pass	$pass"
    echo "fail	$fail"
    grep -E '^\[(resolve-summary|resolve-split|assembly-split|entry-view-assembly|cost-partition)\]' "$stderr_file" || true
  } >"$out_tsv"
}

run_cohort_arm() {
  local label=$1
  local bin_path=$2
  local bin_sha=$3
  local stderr_file="$RECEIPT_DIR/${label}.stderr.txt"
  local stdout_file="$RECEIPT_DIR/${label}.stdout.txt"
  local -a args=(--source-root dag --source-root src/v2)
  while IFS=$'\t' read -r ord entry func _; do
    [[ "$ord" == "ordinal" ]] && continue
    args+=(--entry "$entry" --functions "$func")
  done < "$COHORT"

  if [[ ! -x "$bin_path" ]]; then
    echo "missing binary: $bin_path" >&2
    exit 1
  fi

  echo "=== cohort $label at $(date -Is) ==="
  local start end
  start=$(date +%s)
  set +e
  if [[ -x /usr/bin/time ]]; then
    { /usr/bin/time -v "$bin_path" "${args[@]}"; } >"$stdout_file" 2>"$stderr_file"
  else
    "$bin_path" "${args[@]}" >"$stdout_file" 2>"$stderr_file"
  fi
  local exit_code=$?
  set -e
  end=$(date +%s)
  if ! grep -q 'Maximum resident set size' "$stderr_file" 2>/dev/null; then
    {
      echo "Elapsed (wall clock) time (h:mm:ss or m:ss): $(awk -v s=$((end - start)) 'BEGIN{printf "%d:%05.2f", int(s/60), s%60}')"
      echo "Maximum resident set size (kbytes): unknown"
    } >>"$stderr_file"
  fi
  write_arm_tsv "$label" "$bin_sha" "$stderr_file" "$stdout_file" "$exit_code"
  echo "=== cohort $label wall_ms=$(awk -F'\t' '/^wall_ms/ {print $2}' "$RECEIPT_DIR/${label}.tsv") assembly_ms=$(awk -F'\t' '/^exclusive_assembly_ms/ {print $2}' "$RECEIPT_DIR/${label}.tsv") peak_rss_kb=$(awk -F'\t' '/^peak_rss_kb/ {print $2}' "$RECEIPT_DIR/${label}.tsv") ==="
}

write_floor_tsv() {
  local label=$1
  local bin_sha=$2
  local stderr_file=$3
  local exit_code=$4
  local out_tsv="$RECEIPT_DIR/${label}.tsv"

  local wall_ms peak_rss_kb swap_events
  peak_rss_kb=$(grep 'Maximum resident set size' "$stderr_file" | awk '{print $6}' | tail -1)
  wall_ms="unknown"
  local time_wall
  time_wall=$(grep -E '^Elapsed \(wall clock\) time \(h:mm:ss or m:ss\):' "$stderr_file" | tail -1 | sed -E 's/.*: //')
  if [[ -n "$time_wall" ]]; then
    if [[ "$time_wall" =~ ^([0-9]+):([0-9]{2}):([0-9.]+)$ ]]; then
      wall_ms=$(awk -v h="${BASH_REMATCH[1]}" -v m="${BASH_REMATCH[2]}" -v s="${BASH_REMATCH[3]}" 'BEGIN{printf "%.0f", (h*3600+m*60+s)*1000}')
    elif [[ "$time_wall" =~ ^([0-9]+):([0-9.]+)$ ]]; then
      wall_ms=$(awk -v m="${BASH_REMATCH[1]}" -v s="${BASH_REMATCH[2]}" 'BEGIN{printf "%.0f", (m*60+s)*1000}')
    fi
  fi
  swap_events=$(grep -E 'swap|pgmajfault' "$stderr_file" | tail -5 || true)
  local memo_line
  memo_line=$(grep -E '^\[entry-view-assembly\]' "$stderr_file" | tail -1 || true)

  {
    echo "label	$label"
    echo "binary_sha256	$bin_sha"
    echo "exit	$exit_code"
    echo "floor_wall_ms	$wall_ms"
    echo "peak_rss_kb	${peak_rss_kb:-unknown}"
    echo "diff_base	$FLOOR_DIFF_BASE"
    [[ -n "$memo_line" ]] && echo "$memo_line"
    grep -E '^\[(floor-|selection|entry-view-assembly|resolve-summary)\]' "$stderr_file" || true
    if [[ -n "$swap_events" ]]; then
      echo "# swap_observations"
      echo "$swap_events"
    fi
  } >"$out_tsv"
}

run_floor_arm() {
  local label=$1
  local bin_path=$2
  local bin_sha=$3
  local stderr_file="$RECEIPT_DIR/${label}.stderr.txt"

  if [[ ! -x "$bin_path" ]]; then
    echo "missing binary: $bin_path" >&2
    exit 1
  fi

  echo "=== floor $label at $(date -Is) diff_base=$FLOOR_DIFF_BASE ==="
  local start end
  start=$(date +%s)
  set +e
  if [[ -x /usr/bin/time ]]; then
    {
      /usr/bin/time -v env \
        GITHUB_EVENT_NAME=pull_request \
        GUNBC_CI_DIFF_BASE="$FLOOR_DIFF_BASE" \
        "$bin_path" \
        --source-root dag \
        --source-root src/v2 \
        --plan-entry src/v2/workflow/ci_floor_plan.dag \
        --plan-function gunbc_ci_floor_plan
    } >"$RECEIPT_DIR/${label}.stdout.txt" 2>"$stderr_file"
  else
    env \
      GITHUB_EVENT_NAME=pull_request \
      GUNBC_CI_DIFF_BASE="$FLOOR_DIFF_BASE" \
      "$bin_path" \
      --source-root dag \
      --source-root src/v2 \
      --plan-entry src/v2/workflow/ci_floor_plan.dag \
      --plan-function gunbc_ci_floor_plan \
      >"$RECEIPT_DIR/${label}.stdout.txt" 2>"$stderr_file"
  fi
  local exit_code=$?
  set -e
  end=$(date +%s)
  if ! grep -q 'Maximum resident set size' "$stderr_file" 2>/dev/null; then
    {
      echo "Elapsed (wall clock) time (h:mm:ss or m:ss): $(awk -v s=$((end - start)) 'BEGIN{printf "%d:%05.2f", int(s/60), s%60}')"
      echo "Maximum resident set size (kbytes): unknown"
    } >>"$stderr_file"
  fi
  write_floor_tsv "$label" "$bin_sha" "$stderr_file" "$exit_code"
  echo "=== floor $label wall_ms=$(awk -F'\t' '/^floor_wall_ms/ {print $2}' "$RECEIPT_DIR/${label}.tsv") peak_rss_kb=$(awk -F'\t' '/^peak_rss_kb/ {print $2}' "$RECEIPT_DIR/${label}.tsv") ==="
}

ensure_after_sources() {
  if git apply -R --check "$PATCH" 2>/dev/null; then
    git apply -R "$PATCH"
  fi
}

build_arm_binaries() {
  echo "=== building after-arm binaries ==="
  ensure_after_sources
  cargo build --release -p v1-compiler --bin claim_batch --bin claim_executor
  cp target/release/claim_batch "$AFTER_BIN"
  cp target/release/claim_executor "$FLOOR_BIN"
  AFTER_BATCH_SHA=$(sha256sum "$AFTER_BIN" | awk '{print $1}')
  AFTER_FLOOR_SHA=$(sha256sum "$FLOOR_BIN" | awk '{print $1}')

  echo "=== building base-arm binaries (patch applied) ==="
  git apply "$PATCH"
  cargo build --release -p v1-compiler --bin claim_batch --bin claim_executor
  cp target/release/claim_batch "$BASE_BIN"
  cp target/release/claim_executor "$FLOOR_BASE_BIN"
  BASE_BATCH_SHA=$(sha256sum "$BASE_BIN" | awk '{print $1}')
  BASE_FLOOR_SHA=$(sha256sum "$FLOOR_BASE_BIN" | awk '{print $1}')
  git apply -R "$PATCH"
  cargo build --release -p v1-compiler --bin claim_batch --bin claim_executor >/dev/null

  cat >"$RECEIPT_DIR/binary_sha256.txt" <<EOF
after_claim_batch $AFTER_BATCH_SHA
base_claim_batch $BASE_BATCH_SHA
after_claim_executor $AFTER_FLOOR_SHA
base_claim_executor $BASE_FLOOR_SHA
EOF
}

: >"$RECEIPT_DIR/run.log"

build_arm_binaries
ls -la "$BASE_BIN" "$AFTER_BIN" "$FLOOR_BIN" "$FLOOR_BASE_BIN"

# Interleaved cohort: base-r1 / after-r1 / base-r2 / after-r2
run_cohort_arm base-r1 "$BASE_BIN" "$(awk '/^base_claim_batch/ {print $2}' "$RECEIPT_DIR/binary_sha256.txt")"
run_cohort_arm after-r1 "$AFTER_BIN" "$(awk '/^after_claim_batch/ {print $2}' "$RECEIPT_DIR/binary_sha256.txt")"
run_cohort_arm base-r2 "$BASE_BIN" "$(awk '/^base_claim_batch/ {print $2}' "$RECEIPT_DIR/binary_sha256.txt")"
run_cohort_arm after-r2 "$AFTER_BIN" "$(awk '/^after_claim_batch/ {print $2}' "$RECEIPT_DIR/binary_sha256.txt")"

if [[ "${COHORT_ONLY:-}" == "1" ]]; then
  python3 "$RECEIPT_DIR/derive_summary.py"
  echo "=== RECEIPT_SUMMARY_BEGIN ==="
  cat "$RECEIPT_DIR/summary.json"
  echo "=== RECEIPT_SUMMARY_END ==="
  for arm in base-r1 after-r1 base-r2 after-r2; do
    echo "=== ARM_TSV_BEGIN $arm ==="
    cat "$RECEIPT_DIR/${arm}.tsv"
    echo "=== ARM_TSV_END $arm ==="
  done
  echo "=== cohort-only measurement complete (floor skipped) ==="
  exit 0
fi

# Interleaved floor: same order
run_floor_arm floor-base-r1 "$FLOOR_BASE_BIN" "$(awk '/^base_claim_executor/ {print $2}' "$RECEIPT_DIR/binary_sha256.txt")"
run_floor_arm floor-after-r1 "$FLOOR_BIN" "$(awk '/^after_claim_executor/ {print $2}' "$RECEIPT_DIR/binary_sha256.txt")"
run_floor_arm floor-base-r2 "$FLOOR_BASE_BIN" "$(awk '/^base_claim_executor/ {print $2}' "$RECEIPT_DIR/binary_sha256.txt")"
run_floor_arm floor-after-r2 "$FLOOR_BIN" "$(awk '/^after_claim_executor/ {print $2}' "$RECEIPT_DIR/binary_sha256.txt")"

python3 "$RECEIPT_DIR/derive_summary.py"
echo "=== RECEIPT_SUMMARY_BEGIN ==="
cat "$RECEIPT_DIR/summary.json"
echo "=== RECEIPT_SUMMARY_END ==="

cat >"$RECEIPT_DIR/subject.tsv" <<EOF
field	value
after_commit	${AFTER_COMMIT}
base_arm	same tree as after_commit with base-arm-revert.patch applied (entry-view assembly memos only; post-#7836 fill composition retained)
base_parent_commit	PR B commits reversed on cli_run.rs and claim_batch.rs only (surgical revert, not 26c36ec)
base_binary_sha256	$(awk '/^base_claim_batch/ {print $2}' "$RECEIPT_DIR/binary_sha256.txt")
after_binary_sha256	$(awk '/^after_claim_batch/ {print $2}' "$RECEIPT_DIR/binary_sha256.txt")
binary	target/release/claim_batch and claim_executor; cargo build --release -p v1-compiler
host_arch	$(uname -m)
host_cpus	$(nproc)
host_mem_total_gib	$(awk '/MemTotal/ {printf "%.0f", $2/1024/1024}' /proc/meminfo)
host_cgroup	memory.max=$(cat /sys/fs/cgroup/memory.max 2>/dev/null || echo unknown)
memory_instrument	/usr/bin/time -v Maximum resident set size (kbytes)
run_order	cohort base-r1, after-r1, base-r2, after-r2; then floor base-r1, after-r1, base-r2, after-r2
cohort_invocation	single process, single MultiEntryIndex: claim_batch x50 fixed roster (cohort.tsv)
floor_invocation	claim_executor gunbc_ci_floor_plan with GITHUB_EVENT_NAME=pull_request GUNBC_CI_DIFF_BASE=${FLOOR_DIFF_BASE}
floor_diff_base	${FLOOR_DIFF_BASE}
post_7836_peak_rss_kb_envelope	${POST_7836_PEAK_RSS_KB}
EOF

echo "=== measurement complete ==="
