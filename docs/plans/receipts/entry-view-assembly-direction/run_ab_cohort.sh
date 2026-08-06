#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: entry-view assembly memo direction closed (measured no-go;
# docs/plans/entry-view-assembly-direction-receipt.md). Superseded by run_all.sh for
# the committed receipt; retained as a narrower cohort-only reproduction entrypoint.
# dissolve-on: delete with run_all.sh when this receipt directory retires.
#
# Interleaved A/B cohort: base-r1 / after-r1 / base-r2 / after-r2
# One tree (after_commit), one variable (base-arm-revert.patch applied or not).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
RECEIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PATCH="$RECEIPT_DIR/base-arm-revert.patch"
COHORT="$RECEIPT_DIR/cohort.tsv"
AFTER_COMMIT="$(git -C "$ROOT" rev-parse HEAD)"
BASE_BIN="$RECEIPT_DIR/claim_batch.base"
AFTER_BIN="$RECEIPT_DIR/claim_batch.after"

cd "$ROOT"

build_args() {
  local -a args=(--source-root dag --source-root src/v2)
  while IFS=$'\t' read -r ord entry func _; do
    [[ "$ord" == "ordinal" ]] && continue
    args+=(--entry "$entry" --functions "$func")
  done < "$COHORT"
  printf '%s\n' "${args[@]}"
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

  local time_wall
  time_wall=$(grep -E '^Elapsed \(wall clock\) time \(h:mm:ss or m:ss\):' "$stderr_file" | tail -1 | sed -E 's/.*: //')
  wall_ms="unknown"
  if [[ -n "$time_wall" ]]; then
    if [[ "$time_wall" =~ ^([0-9]+):([0-9]{2}):([0-9.]+)$ ]]; then
      wall_ms=$(awk -v h="${BASH_REMATCH[1]}" -v m="${BASH_REMATCH[2]}" -v s="${BASH_REMATCH[3]}" 'BEGIN{printf "%.0f", (h*3600+m*60+s)*1000}')
    elif [[ "$time_wall" =~ ^([0-9]+):([0-9.]+)$ ]]; then
      wall_ms=$(awk -v m="${BASH_REMATCH[1]}" -v s="${BASH_REMATCH[2]}" 'BEGIN{printf "%.0f", (m*60+s)*1000}')
    fi
  fi

  {
    echo "label	$label"
    echo "binary_sha256	$bin_sha"
    echo "exit	$exit_code"
    echo "wall_ms	$wall_ms"
    echo "peak_rss_kb	${peak_rss_kb:-unknown}"
    echo "pass	$pass"
    echo "fail	$fail"
    grep -E '^\[(resolve-summary|resolve-split|assembly-split|entry-view-assembly|cost-partition)\]' "$stderr_file" || true
  } >"$out_tsv"
}

run_with_binary() {
  local label=$1
  local bin_path=$2
  local bin_sha=$3
  local stderr_file="$RECEIPT_DIR/${label}.stderr.txt"
  mapfile -t ARGS < <(build_args)

  echo "=== $label binary=$bin_sha at $(date -Is) ==="
  set +e
  { /usr/bin/time -v "$bin_path" "${ARGS[@]}"; } >"$RECEIPT_DIR/${label}.stdout.txt" 2>"$stderr_file"
  local exit_code=$?
  set -e
  write_arm_tsv "$label" "$bin_sha" "$stderr_file" "$RECEIPT_DIR/${label}.stdout.txt" "$exit_code"
  echo "=== $label done exit=$exit_code wall_ms=$(awk -F'\t' '/^wall_ms/ {print $2}' "$RECEIPT_DIR/${label}.tsv") peak_rss_kb=$(awk -F'\t' '/^peak_rss_kb/ {print $2}' "$RECEIPT_DIR/${label}.tsv") ==="
}

ensure_clean_tree() {
  if git apply -R --check "$PATCH" 2>/dev/null; then
    git apply -R "$PATCH"
  fi
}

: >"$RECEIPT_DIR/binary_sha256.txt"
ensure_clean_tree

# Build base (post-#7836, pre entry-view memos)
git apply "$PATCH"
CTRL_BUILD_MODE=local cargo build --release -p v1-compiler --bin claim_batch
cp target/release/claim_batch "$BASE_BIN"
BASE_SHA=$(sha256sum "$BASE_BIN" | awk '{print $1}')
echo "base $BASE_SHA" >>"$RECEIPT_DIR/binary_sha256.txt"

# Build after (current HEAD mechanism)
git apply -R "$PATCH"
CTRL_BUILD_MODE=local cargo build --release -p v1-compiler --bin claim_batch
cp target/release/claim_batch "$AFTER_BIN"
AFTER_SHA=$(sha256sum "$AFTER_BIN" | awk '{print $1}')
echo "after $AFTER_SHA" >>"$RECEIPT_DIR/binary_sha256.txt"

# Interleaved runs on one host, back to back
run_with_binary base-r1 "$BASE_BIN" "$BASE_SHA"
run_with_binary after-r1 "$AFTER_BIN" "$AFTER_SHA"
run_with_binary base-r2 "$BASE_BIN" "$BASE_SHA"
run_with_binary after-r2 "$AFTER_BIN" "$AFTER_SHA"

cat >"$RECEIPT_DIR/subject.tsv" <<EOF
field	value
after_commit	${AFTER_COMMIT}
base_arm	same tree as after_commit with base-arm-revert.patch applied (entry-view assembly memos only; post-#7836 fill composition retained)
base_parent_commit	26c36ec47a86f5e7aa7b3dc72e6fbbf626543bd2
base_binary_sha256	${BASE_SHA}
after_binary_sha256	${AFTER_SHA}
binary	target/release/claim_batch, cargo build --release -p v1-compiler --bin claim_batch
host_arch	$(uname -m)
host_cpus	$(nproc)
host_mem_total_gib	$(awk '/MemTotal/ {printf "%.0f", $2/1024/1024}' /proc/meminfo)
host_cgroup	root (no per-run cgroup; cgroup peak not separable from the host on this box)
memory_instrument	/usr/bin/time -v Maximum resident set size (kbytes); kernel-tracked peak
run_order	base-r1, after-r1, base-r2, after-r2 (interleaved, sequential, one run at a time)
invocation	single process, single MultiEntryIndex: claim_batch --source-root dag --source-root src/v2 (--entry P --functions F) x50
selection	none; the roster is the fixed cohort in cohort.tsv, identical in both arms
EOF

echo "A/B cohort complete. Binaries:" && cat "$RECEIPT_DIR/binary_sha256.txt"
