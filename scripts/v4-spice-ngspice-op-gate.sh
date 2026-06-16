#!/usr/bin/env bash
# scripts/v4-spice-ngspice-op-gate.sh
#
# P4 SPICE ngspice oracle gate (SP-M1):
#   1. claim-run structural emit witness (spice_rc_ngspice_op_holds)
#   2. spice_ngspice_oracle: emit spice_rc_passive_deck → deck.cir → ngspice -b
#
# Env:
#   V2_COMPILER              — gunbc binary (default: target/release/gunbc)
#   SPICE_NGSPICE_ORACLE     — oracle binary (default: target/release/spice_ngspice_oracle)
#   NGSPICE_BIN              — ngspice executable override
#   V4_SPICE_NGSPICE_GATE_STRICT — if 1, exit non-zero when ngspice is missing

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
oracle_bin="${SPICE_NGSPICE_ORACLE:-target/release/spice_ngspice_oracle}"
claim_entry="src/v4/test/claim/formats/spice_rc_ngspice_op.dag"
claim_fn="spice_rc_ngspice_op_holds"
strict="${V4_SPICE_NGSPICE_GATE_STRICT:-${GITHUB_ACTIONS:+1}}"
strict="${strict:-0}"

if [[ ! -x "$bin" ]]; then
  cargo build -p v2-compiler --release --bin gunbc
fi

if [[ ! -x "$oracle_bin" ]]; then
  cargo build -p v2-compiler --release --bin spice_ngspice_oracle
fi

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc not found at $bin" >&2
  exit 2
fi

if [[ ! -x "$oracle_bin" ]]; then
  echo "error: spice_ngspice_oracle not found at $oracle_bin" >&2
  echo "build with: cargo build -p v2-compiler --release --bin spice_ngspice_oracle" >&2
  exit 2
fi

ngspice_bin="${NGSPICE_BIN:-ngspice}"
if ! command -v "$ngspice_bin" >/dev/null 2>&1; then
  echo "error: ngspice not found (set NGSPICE_BIN or install ngspice)" >&2
  if [[ "$strict" == "1" ]]; then
    echo "::error title=spice ngspice gate setup::ngspice missing (spice/setup/ngspice_missing)"
    exit 2
  fi
  echo "::notice title=spice ngspice gate::skipped — ngspice missing"
  exit 0
fi

echo "::group::spice ngspice gate: claim-run emit witness"
"$bin" run \
  --source-root src/v4 \
  --entry "$claim_entry" \
  --function "$claim_fn" \
  --claim-run
echo "::endgroup::"

echo "::group::spice ngspice gate: ngspice -b oracle"
NGSPICE_BIN="$ngspice_bin" "$oracle_bin"
echo "::endgroup::"

echo "::notice title=spice ngspice gate::emit witness + ngspice -b passed"
