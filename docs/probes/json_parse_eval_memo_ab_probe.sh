#!/usr/bin/env bash
# SCAFFOLD — EvalCallMemo A/B for parse_json peak RSS (operator redirect eager-koi-458).
# dissolve-on: eval-memo byte-budget + two-hit admission lands on main, OR memo-off A/B shows
# memo is not the dominant term and the length-sweep receipt on json_unescape_from completes.
# Authority: extdeps.realization.eval_memo eval_memo_note; receipt
# src/v1/stage0/tests/json_parse_eval_memo_ab_receipt.rs.
# Witness: json_parse_eval_memo_ab_decode_sanity (correctness); A/B probe is #[ignore]d here.
#
# Varies only GUNBC_EVAL_MEMO (1 vs 0). Separate scoped processes per arm (VmHWM resets).
# MemoryMax cap required — do not run uncapped on shared srv1.
# `timeout N systemd-run --scope` does NOT kill the scoped process; poll/stop by scope name.
set -euo pipefail

MEMORY_MAX_GIB="${1:-4}"
DECODED_LEN="${JSON_PARSE_AB_DECODED_LEN:-$((50 * 1024))}"
REPO_ROOT="$(git rev-parse --show-toplevel)"
STAGE0="${REPO_ROOT}/src/v1/stage0"
LOG_DIR="${REPO_ROOT}/target/json_parse_eval_memo_ab"
mkdir -p "${LOG_DIR}"

if ! command -v systemd-run >/dev/null 2>&1; then
  echo "REFUSED: systemd-run not found (run on srv1)" >&2
  exit 1
fi

cd "${STAGE0}"

describe_termination() {
  local code=$1
  case "${code}" in
    0) echo "CompletedExit0" ;;
    127) echo "CommandNotFoundExit127 (probe never started — void reading)" ;;
    134) echo "SigAbortExit134" ;;
    137) echo "SigKillExit137 (likely OOM under MemoryMax)" ;;
    *) echo "Exit${code}" ;;
  esac
}

run_arm() {
  local memo_val=$1
  local scope_name="json-parse-eval-memo-ab-memo${memo_val}-$$"
  local log="${LOG_DIR}/memo_${memo_val}.log"
  local run_cmd="JSON_PARSE_AB_DECODED_LEN=${DECODED_LEN} GUNBC_EVAL_MEMO=${memo_val} cargo test -p v1-compiler --release --test json_parse_eval_memo_ab_receipt json_parse_eval_memo_ab_probe -- --ignored --nocapture"

  echo "=== arm GUNBC_EVAL_MEMO=${memo_val} MemoryMax=${MEMORY_MAX_GIB}G decoded_len=${DECODED_LEN} ==="
  echo "prediction: memo_on shows near-zero hits + large misses; high hit_rate refutes hypothesis"
  echo "log: ${log}"

  set +e
  systemd-run --user --scope \
    -p "MemoryMax=${MEMORY_MAX_GIB}G" \
    --unit="${scope_name}" \
    bash -lc "${run_cmd}" >"${log}" 2>&1
  local exit_code=$?
  set -e

  if systemctl --user is-active "${scope_name}.scope" >/dev/null 2>&1; then
    systemctl --user stop "${scope_name}.scope" || true
  fi

  local termination
  termination="$(describe_termination "${exit_code}")"
  echo "process_termination=${termination} (wait_exit=${exit_code})"
  grep -E '^eval_memo_ab_receipt:|^  ' "${log}" 2>/dev/null || tail -30 "${log}" || true
  echo
}

echo "EvalCallMemo A/B — compare parse_phase_vhwm_increase_bytes between arms, not setup-dominated process peak"
echo

run_arm 1
run_arm 0

echo "A/B complete — logs under ${LOG_DIR}/"
echo "Compare: parse_phase_vhwm_increase_bytes, memo_hits/misses/overflow, setup_dominates_discriminator"
