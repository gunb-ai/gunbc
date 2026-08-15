#!/usr/bin/env bash
# SCAFFOLD — srv1 capped-run wrapper for json_unescape_from cost measurement (not a general scripts/ home).
# dissolve-on: json_unescape_from index-only + linear-accumulator fix lands and
# src/v1/stage0/tests/json_unescape_cost_receipt.rs deletes with the measurement-complete fix PR.
# Authority: extdeps.languages.json.parse json_unescape_cost_receipt_seed_growth_mark (subject module)
# and src/v1/stage0/tests/json_unescape_cost_receipt.rs (ignored benchmark receipt).
# Witness: json_unescape_modeled_copy_terms_scale_quadratically (gating); length-sweep / escape-density /
# term-separation probes are #[ignore]d and run via this script on srv1 under MemoryMax.
#
# Operator brief: capped runs only — systemd-run --user --scope -p MemoryMax=NG.
# Poll and kill by scope name; `timeout` does not tear down a scope.
#
# Usage (on srv1, from repo root):
#   docs/probes/json_unescape_cost_probe.sh [MEMORY_MAX_GIB]
#
# Example:
#   docs/probes/json_unescape_cost_probe.sh 4
#
# Requires a release build:
#   cd src/v1/stage0 && cargo build --release -p v1-compiler

set -euo pipefail

MEMORY_MAX_GIB="${1:-4}"
SCOPE_NAME="json-unescape-cost-probe-$$"
REPO_ROOT="$(git rev-parse --show-toplevel)"
STAGE0="${REPO_ROOT}/src/v1/stage0"
LOG_DIR="${REPO_ROOT}/target/json_unescape_cost_probe"
mkdir -p "${LOG_DIR}"

if ! command -v systemd-run >/dev/null 2>&1; then
  echo "REFUSED: systemd-run not found (run on srv1)" >&2
  exit 1
fi

cd "${STAGE0}"

RUN_CMD="cargo test -p v1-compiler --release --test json_unescape_cost_receipt -- --ignored --nocapture"

echo "starting scoped run: scope=${SCOPE_NAME} MemoryMax=${MEMORY_MAX_GIB}G"
echo "log: ${LOG_DIR}/probe.log"

systemd-run --user --scope \
  -p "MemoryMax=${MEMORY_MAX_GIB}G" \
  --unit="${SCOPE_NAME}" \
  bash -lc "${RUN_CMD}" >"${LOG_DIR}/probe.log" 2>&1 &
RUN_PID=$!

cleanup() {
  if systemctl --user is-active "${SCOPE_NAME}.scope" >/dev/null 2>&1; then
    systemctl --user stop "${SCOPE_NAME}.scope" || true
  fi
}
trap cleanup EXIT

if ! wait "${RUN_PID}"; then
  echo "probe finished non-zero — see ${LOG_DIR}/probe.log"
  tail -40 "${LOG_DIR}/probe.log" || true
  exit 1
fi

echo "probe complete — ${LOG_DIR}/probe.log"
grep -E '^(===|  |decoded_len|density=|production)' "${LOG_DIR}/probe.log" || cat "${LOG_DIR}/probe.log"
