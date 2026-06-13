#!/usr/bin/env bash
# Local profiler for v4-claim-witness-corpus-gate.sh — phase + per-invocation timing.
# Read-only instrumentation wrapper; does not change gate semantics.
set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

shard="${1:-}"
if [[ ! "$shard" =~ ^[ab]$ ]]; then
  echo "usage: $0 <a|b> [--spot-perturb-check]" >&2
  exit 2
fi
shift || true

bin="${V2_COMPILER:-target/release/gunbc}"
bin_batch="${CLAIM_BATCH:-target/release/claim_batch}"
perturb_mode="none"
for arg in "$@"; do
  case "$arg" in
    --spot-perturb-check) perturb_mode="spot" ;;
    --perturb-check) perturb_mode="full" ;;
  esac
done

if [[ ! -x "$bin" || ! -x "$bin_batch" ]]; then
  echo "error: build gunbc + claim_batch first" >&2
  exit 2
fi

# shellcheck source=/dev/null
source <(
  # Re-export gate helpers by sourcing the gate script's function definitions only.
  sed -n '/^dag_string_data()/,/^}/p;
         /^list_claim_run_row_members()/,/^}/p;
         /^project_list_member_row()/,/^}/p;
         /^run_row()/,/^}/p;
         /^classify_stdout()/,/^}/p;
         /^perturb_function_to_false()/,/^PY$/p;
         /^perturb_one_row()/,/^}/p;
         /^pick_spot_perturb_indices()/,/^}/p' \
    "$root/scripts/v4-claim-witness-corpus-gate.sh"
)

gate_model="src/v4/test/claim/workflow/claim_witness_corpus_ci_runner.dag"
rows_data="claim_witness_corpus_shard_${shard}_rows"
count_data="claim_witness_corpus_shard_${shard}_row_count"

now_s() { date +%s; }

timed_claim_batch() {
  local label="$1"
  shift
  local t0 t1 dt
  t0="$(now_s)"
  "$bin_batch" "$@" --claim-run
  t1="$(now_s)"
  dt=$((t1 - t0))
  echo "PROFILE claim_batch entry=${label} wall_s=${dt}" >&2
  echo "$dt"
}

timed_run_row() {
  local label="$1"
  local source_root="$2"
  local entry="$3"
  local function="$4"
  local t0 t1 dt
  t0="$(now_s)"
  run_row "$source_root" "$entry" "$function" >/dev/null 2>&1 || true
  t1="$(now_s)"
  dt=$((t1 - t0))
  echo "PROFILE gunbc_run label=${label} wall_s=${dt}" >&2
  echo "$dt"
}

gate_t0="$(now_s)"
phase_a_s=0
phase_fail_s=0
phase_perturb_s=0
batch_calls=0
batch_index_rebuild_est_s=0

expected_count="$(dag_string_data "$count_data")"
all_rows=()
pass_rows=()
fail_rows=()
while IFS= read -r member; do
  [[ -z "$member" ]] && continue
  row="$(project_list_member_row "$member")"
  all_rows+=("$row")
  IFS=$'\t' read -r _label _entry _function expect bind_anchor <<< "$row"
  if [[ "$expect" == pass ]]; then
    pass_rows+=("$row")
  else
    fail_rows+=("$row")
  fi
done < <(list_claim_run_row_members "$rows_data")

# Phase A: mirror batch_green_pass but time each claim_batch invocation.
if [[ "${#pass_rows[@]}" -gt 0 ]]; then
  phase_a_start="$(now_s)"
  declare -A entry_fns=()
  declare -a entry_order=()
  while IFS=$'\t' read -r _label entry function _expect _bind; do
  done < <(printf '%s\n' "${pass_rows[@]}" | cut -f2,3 | while IFS=$'\t' read -r entry function; do
    if [[ -z "${entry_fns[$entry]+x}" ]]; then
      entry_order+=("$entry")
    fi
    entry_fns[$entry]+="${function},"
  done)
  # Rebuild entry_order properly (bash subshell issue above) — inline:
  entry_order=()
  declare -A entry_fns=()
  local_entry=""
  local_fn=""
  while IFS=$'\t' read -r entry function; do
    [[ -z "$entry" ]] && continue
    if [[ -z "${entry_fns[$entry]+x}" ]]; then
      entry_order+=("$entry")
    fi
    entry_fns[$entry]+="${function},"
  done < <(printf '%s\n' "${pass_rows[@]}" | cut -f2,3)

  for e in "${entry_order[@]}"; do
    fns="${entry_fns[$e]%,}"
    dt="$(timed_claim_batch "$e" --source-root src/v4 --entry "$e" --functions "$fns")"
    phase_a_s=$((phase_a_s + dt))
    batch_calls=$((batch_calls + 1))
    # Each separate claim_batch process rebuilds index; log line appears ~0.01-0.5s before resolve.
    batch_index_rebuild_est_s=$((batch_index_rebuild_est_s + 1))
  done
  phase_a_wall=$(( $(now_s) - phase_a_start ))
fi

# Phase B: ExpectFail rows
for row in "${fail_rows[@]}"; do
  IFS=$'\t' read -r label entry function _expect bind_anchor <<< "$row"
  dt="$(timed_run_row "$label" "src/v4" "$entry" "$function")"
  phase_fail_s=$((phase_fail_s + dt))
done

# Phase C: spot/full perturb
if [[ "$perturb_mode" == spot || "$perturb_mode" == full" ]]; then
  phase_perturb_start="$(now_s)"
  if [[ "$perturb_mode" == full ]]; then
    for row in "${pass_rows[@]}"; do
      IFS=$'\t' read -r label entry function _expect _bind <<< "$row"
      t0="$(now_s)"
      tmp="$(mktemp -d)"
      mkdir -p "$tmp"
      cp -a src/v4 "$tmp/src"
      perturbed_entry="$tmp/src/${entry#src/v4/}"
      perturb_function_to_false "$perturbed_entry" "$function"
      run_row "$tmp/src" "$perturbed_entry" "$function" >/dev/null 2>&1 || true
      rm -rf "$tmp"
      phase_perturb_s=$((phase_perturb_s + $(now_s) - t0))
    done
  else
    spot_indices=()
    pick_spot_perturb_indices "${#pass_rows[@]}" spot_indices
    for idx in "${spot_indices[@]}"; do
      row="${pass_rows[$idx]}"
      IFS=$'\t' read -r label entry function _expect _bind <<< "$row"
      t0="$(now_s)"
      tmp="$(mktemp -d)"
      mkdir -p "$tmp"
      cp -a src/v4 "$tmp/src"
      perturbed_entry="$tmp/src/${entry#src/v4/}"
      perturb_function_to_false "$perturbed_entry" "$function"
      run_row "$tmp/src" "$perturbed_entry" "$function" >/dev/null 2>&1 || true
      rm -rf "$tmp"
      phase_perturb_s=$((phase_perturb_s + $(now_s) - t0))
    done
  fi
fi

gate_wall=$(( $(now_s) - gate_t0 ))

cat <<EOF
=== claim witness corpus profile shard=${shard} perturb=${perturb_mode} ===
rows_total=${#all_rows[@]} pass=${#pass_rows[@]} fail=${#fail_rows[@]} modeled=${expected_count}
gate_wall_s=${gate_wall}
phase_a_green_pass_s=${phase_a_s} (claim_batch_calls=${batch_calls})
phase_b_expect_fail_s=${phase_fail_s}
phase_c_perturb_s=${phase_perturb_s}
overhead_unaccounted_s=$(( gate_wall - phase_a_s - phase_fail_s - phase_perturb_s ))
note: each claim_batch call rebuilds module index (N=${batch_calls} separate processes)
EOF
