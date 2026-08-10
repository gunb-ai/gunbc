#!/usr/bin/env bash
# Default-off diagnostic receipt runner. Dissolve with the timing scaffold named by
# import_string_rewire_measurement_scaffold_note once an optimization is selected.
set -euo pipefail

root="$(cd "$(dirname "$0")/../../../.." && pwd)"
cohort="$root/docs/plans/receipts/entry-view-assembly-direction/cohort.tsv"
binary="$root/target/release/claim_batch"
run_dir="$(mktemp -d)"
verify_only=0
trap 'rm -rf "$run_dir"' EXIT

cd "$root"
if [[ "${1:-}" == "--verify-only" ]]; then
  verify_only=1
  shift
fi
if [[ "${1:-}" == "--binary" ]]; then
  binary="$(realpath "$2")"
  shift 2
else
  ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch
fi

if [[ "${1:-}" == "--export-binary" ]]; then
  artifact_dir="${BUILDBUDDY_ARTIFACT_DIR:-/root/workspace/artifacts/command-0}"
  mkdir -p "$artifact_dir"
  cp "$binary" "$artifact_dir/claim_batch"
  sha256sum "$artifact_dir/claim_batch"
  exit 0
fi

args=(--source-root dag --source-root src/v2)
while IFS=$'\t' read -r ordinal entry function; do
  if [[ "$ordinal" == "ordinal" ]]; then
    continue
  fi
  args+=(--entry "$entry" --functions "$function")
done <"$cohort"

echo "receipt_version=streaming-profile-ab-v1"
echo "source_head=$(git rev-parse HEAD)"
echo "binary_sha256=$(sha256sum "$binary" | awk '{print $1}')"
echo "cohort_sha256=$(sha256sum "$cohort" | awk '{print $1}')"
echo "run_order=baseline-r1,profile-r1,profile-r2,baseline-r2"
echo "equivalence_run=profile-verify (after timed A-B-B-A; excluded from tax comparison)"
echo "verify_only=$verify_only"

measure_command() {
  local time_file="$1"
  shift
  local max_run_seconds="${GUNBC_REWIRE_IMPORT_STR_MAX_RUN_SECONDS:-900}"
  local start_ns
  start_ns="$(date +%s%N)"
  local start_seconds="$SECONDS"
  "$@" &
  local measured_pid="$!"
  local peak_rss_kb=0
  local timed_out=0
  local timeout_process_state="not-timed-out"
  process_is_running() {
    if ! kill -0 "$measured_pid" 2>/dev/null; then
      return 1
    fi
    local process_state
    process_state="$(awk '{ print $3 }' "/proc/$measured_pid/stat" 2>/dev/null || true)"
    [[ "$process_state" != "Z" ]]
  }
  while process_is_running; do
    local current_peak_kb
    current_peak_kb="$(awk '/^VmHWM:/ { print $2 }' "/proc/$measured_pid/status" 2>/dev/null || true)"
    if [[ -n "$current_peak_kb" ]] && (( current_peak_kb > peak_rss_kb )); then
      peak_rss_kb="$current_peak_kb"
    fi
    if (( SECONDS - start_seconds >= max_run_seconds )); then
      timed_out=1
      timeout_process_state="$(awk '{ print $3 }' "/proc/$measured_pid/stat" 2>/dev/null || echo unreadable)"
      kill -TERM "$measured_pid" 2>/dev/null || true
      break
    fi
    sleep 0.05
  done
  local measured_exit
  if (( timed_out == 1 )); then
    local term_polls=0
    while process_is_running && (( term_polls < 200 )); do
      sleep 0.05
      term_polls=$((term_polls + 1))
    done
    if process_is_running; then
      kill -KILL "$measured_pid" 2>/dev/null || true
    fi
    wait "$measured_pid" 2>/dev/null || true
    measured_exit=124
  elif wait "$measured_pid"; then
    measured_exit=0
  else
    measured_exit="$?"
  fi
  local end_ns
  end_ns="$(date +%s%N)"
  awk -v start="$start_ns" -v end="$end_ns" 'BEGIN { printf "wall_seconds=%.3f\n", (end-start)/1000000000 }' >"$time_file"
  echo "peak_rss_kb=$peak_rss_kb" >>"$time_file"
  echo "timed_out=$timed_out" >>"$time_file"
  echo "timeout_process_state=$timeout_process_state" >>"$time_file"
  return "$measured_exit"
}

run_one() {
  local label="$1"
  local profile="$2"
  local verify="${3:-0}"
  local stdout_file="$run_dir/$label.stdout"
  local stderr_file="$run_dir/$label.stderr"
  local time_file="$run_dir/$label.time"

  set +e
  if [[ "$verify" == "1" ]]; then
    measure_command "$time_file" env \
      GUNBC_REWIRE_IMPORT_STR_PROFILE=1 GUNBC_REWIRE_IMPORT_STR_VERIFY=1 \
      "$binary" "${args[@]}" >"$stdout_file" 2>"$stderr_file"
  elif [[ "$profile" == "1" ]]; then
    measure_command "$time_file" env GUNBC_REWIRE_IMPORT_STR_PROFILE=1 \
      "$binary" "${args[@]}" \
      >"$stdout_file" 2>"$stderr_file"
  else
    measure_command "$time_file" env -u GUNBC_REWIRE_IMPORT_STR_PROFILE \
      "$binary" "${args[@]}" \
      >"$stdout_file" 2>"$stderr_file"
  fi
  local exit_code="$?"
  set -e

  local outcomes_sha256
  outcomes_sha256="$(sort "$stdout_file" | sha256sum | awk '{print $1}')"
  local subjects_sha256
  subjects_sha256="$({
    sed -n -E 's/^\[witness\] ([^:]+):.* subject=([^ ]+).*/\1\t\2/p' "$stderr_file"
  } | sort | sha256sum | awk '{print $1}')"
  local closures_sha256
  closures_sha256="$({
    sed -n -E 's/^\[resolve\] ([^:]+):.*\(([0-9]+ modules, [0-9]+ resolved items in closure)\)$/\1\t\2/p' "$stderr_file"
  } | sort | sha256sum | awk '{print $1}')"

  echo "run=$label"
  echo "profile_enabled=$profile"
  echo "complete_module_equivalence_check=$verify"
  echo "exit=$exit_code"
  echo "pass=$(grep -c '^PASS ' "$stdout_file" || true)"
  echo "fail=$(grep -c '^FAIL ' "$stdout_file" || true)"
  echo "outcomes_sha256=$outcomes_sha256"
  echo "subjects_sha256=$subjects_sha256"
  echo "closures_sha256=$closures_sha256"
  cat "$time_file"
  grep -E '^\[(resolve-summary|rewire-import-str-split)\]' "$stderr_file" || true
  local cost_partition
  cost_partition="$(sed -n 's/^\[cost-partition\] //p' "$stderr_file" | tail -1)"
  if [[ -n "$cost_partition" ]]; then
    jq -r '
      "parent_span_nanos=\(.parent_span_nanos)",
      "assembly_rewire_import_str_nanos=\(.exclusive.assembly_rewire_import_str)",
      "type_name_index_nanos=\(.inclusive.assembly_rewire_type_name_index.nanos)",
      "export_name_index_nanos=\(.inclusive.assembly_rewire_export_name_index.nanos)",
      "module_preparation_nanos=\(.inclusive.assembly_rewire_module_preparation.nanos)",
      "binding_application_nanos=\(.inclusive.assembly_rewire_binding_application.nanos)",
      "partition_state=\(.verdict.state)",
      "partition_residual_nanos=\(.verdict.residual_nanos)"
    ' <<<"$cost_partition"
  else
    echo "cost_partition=unavailable"
  fi
  echo "end_run=$label"
}

if [[ "$verify_only" == "0" ]]; then
  run_one baseline-r1 0
  run_one profile-r1 1
  run_one profile-r2 1
  run_one baseline-r2 0
fi
run_one profile-verify 1 1
