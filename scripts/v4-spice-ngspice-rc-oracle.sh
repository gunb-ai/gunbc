#!/usr/bin/env bash
# scripts/v4-spice-ngspice-rc-oracle.sh
#
# ngspice execution oracle — RC transient circuit (T-4.10 clear-win receipt).
#
# Verifies that the netlist emitted by spice_emit_ngspice(spice_rc_tran_deck) simulates
# to completion in ngspice batch mode (exit 0). This is the execution consumer required by
# INVARIANTS.md E-10: the emitted string must actually run, not just typecheck.
#
# Circuit: V1 (5 V DC) → R1 (1 kΩ) → C1 (1 µF) → GND
#          τ = RC = 1 ms; .tran 1us 10ms covers ~10 τ.
#
# Exit codes:
#   0  — ngspice simulated successfully
#   1  — ngspice failed (regression: emitted netlist does not simulate)
#   77 — ngspice binary not found (SKIP; install ngspice to run full oracle)
#
# Env:
#   NGSPICE     — ngspice binary (default: ngspice)
#   ORACLE_VERBOSE — set to 1 for full ngspice stdout

set -euo pipefail

ngspice_bin="${NGSPICE:-ngspice}"
verbose="${ORACLE_VERBOSE:-0}"

if ! command -v "$ngspice_bin" >/dev/null 2>&1; then
  echo "SKIP: ngspice not found (set NGSPICE= to override). Install ngspice to run full oracle." >&2
  exit 77
fi

# Canonical ngspice-ready netlist — must match spice_emit_ngspice(spice_rc_tran_deck).
# Line order is determined by fold_list over the Cons body in spice_rc_tran_deck.dag:
#   title, V1 n1 0 DC 5, R1 n1 n2 1000, C1 n2 0 1e-6, .tran 1us 10ms, .end
netlist="$(cat <<'NETLIST'
* rc tran witness
V1 n1 0 DC 5
R1 n1 n2 1000
C1 n2 0 1e-6
.tran 1us 10ms
.end
NETLIST
)"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

netlist_file="$tmpdir/rc_tran.spi"
printf '%s\n' "$netlist" > "$netlist_file"

echo "oracle: running ngspice on RC transient circuit ..."
if [[ "$verbose" == "1" ]]; then
  "$ngspice_bin" -b "$netlist_file"
  rc=$?
else
  ng_out="$("$ngspice_bin" -b "$netlist_file" 2>&1)" || true
  rc=${PIPESTATUS[0]:-$?}
  if [[ $rc -ne 0 ]]; then
    echo "ngspice output:" >&2
    echo "$ng_out" >&2
  fi
fi

if [[ $rc -eq 0 ]]; then
  echo "oracle: PASS — ngspice RC transient simulation exited 0."
  exit 0
else
  echo "oracle: FAIL — ngspice exited $rc (emitted netlist does not simulate)." >&2
  exit 1
fi
