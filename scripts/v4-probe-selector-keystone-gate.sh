#!/usr/bin/env bash
# ProbeSelector keystone CI gate (PS-0 host-health / availability conditioning).
#
# `.dag`-driven Bool-witness gate transport. Row authority lives in the gate model
# (`--gate-entry` → `--rows-fn probe_selector_ci_runner_rows_tsv`); this script invokes
# the `ci-claim-gate` host, which evaluates the rows via the v2 interpreter and runs the
# GREEN pass plus, on --perturb-check, a per-row witness-body→`false` perturb that must go
# RED. Host-collapse of the prior shell transport: the awk/grep roster projection and the
# inline-python per-row perturb are now absorbed by ci-claim-gate (identical witness roster
# and identical function-body→false perturb mechanism), mirroring scripts/v4-lens-ci-gate.sh.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${CI_CLAIM_GATE:-target/release/ci-claim-gate}"
perturb=()

case "${1:-}" in
  --perturb-check) perturb=(--perturb-check) ;;
  "") ;;
  *)
    echo "usage: $0 [--perturb-check]" >&2
    exit 2
    ;;
esac

if [[ ! -x "$bin" ]]; then
  echo "error: ci-claim-gate not found at $bin (build: cargo build -p ci_claim_gate --release)" >&2
  exit 2
fi

exec "$bin" \
  --source-root src/v4 \
  --gate-entry src/v4/test/claim/workflow/probe_selector_ci_runner.dag \
  --rows-fn probe_selector_ci_runner_rows_tsv \
  --notice-title "probe-selector keystone" \
  "${perturb[@]}"
