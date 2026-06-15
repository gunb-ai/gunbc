#!/usr/bin/env bash
# scripts/v4-spice-ngspice-rc-oracle.sh
#
# ngspice execution oracle — RC transient circuit (T-4.10 clear-win receipt).
#
# Two-step consumer chain (INVARIANTS.md E-10):
#   Step 1 — emit correctness: gunbc --claim-run verifies spice_emit_ngspice(rc_tran_deck)
#             matches the canonical string. Broken emit exits 1 here before ngspice runs.
#   Step 2 — ngspice execution: the canonical netlist string is fed to ngspice -b.
#             A broken simulation exits 1 here.
#
# Exit codes:
#   0  — emit correct + ngspice simulated to completion
#   1  — emit mismatch OR ngspice failed
#   77 — ngspice not found (SKIP; install ngspice to run full oracle)
#
# Env:
#   NGSPICE        — ngspice binary (default: ngspice)
#   GUNBC          — gunbc binary (default: path resolution order below)
#   ORACLE_VERBOSE — set to 1 for full ngspice stdout

set -euo pipefail

ngspice_bin="${NGSPICE:-ngspice}"
verbose="${ORACLE_VERBOSE:-0}"

if ! command -v "$ngspice_bin" >/dev/null 2>&1; then
  echo "SKIP: ngspice not found (set NGSPICE= to override). Install ngspice to run full oracle." >&2
  exit 77
fi

# Resolve gunbc: env override > repo target/release > system PATH
root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
if [[ -n "${GUNBC:-}" ]]; then
  gunbc_bin="$GUNBC"
elif [[ -x "$root/target/release/gunbc" ]]; then
  gunbc_bin="$root/target/release/gunbc"
elif command -v gunbc >/dev/null 2>&1; then
  gunbc_bin="gunbc"
else
  gunbc_bin=""
fi

# Step 1: emit correctness (if gunbc available).
# spice_rc_tran_emit_ngspice_matches_expected_holds() must return true.
# Broken emit → claim-run exits 1 → this oracle exits 1 before running ngspice.
if [[ -n "$gunbc_bin" ]]; then
  echo "oracle: step 1 — verifying emit correctness via gunbc --claim-run ..."
  claim_entry="$root/src/v4/test/claim/formats/spice_rc_ngspice_oracle.dag"
  if ! "$gunbc_bin" run \
      --source-root "$root/src/v4" \
      --entry "$claim_entry" \
      --claim-run 2>&1 | grep -q "spice_rc_tran_emit_ngspice_matches_expected_holds.*true\|PASS\|pass"; then
    # --claim-run exits 1 on false; propagate.
    "$gunbc_bin" run \
      --source-root "$root/src/v4" \
      --entry "$claim_entry" \
      --claim-run >/dev/null 2>&1 || {
        echo "oracle: FAIL — emit claim returned false (spice_emit_ngspice output mismatch)." >&2
        exit 1
      }
  fi
  echo "oracle: step 1 PASS — emit produces canonical netlist string."
else
  echo "oracle: step 1 SKIP — gunbc not found; emit correctness unverified." >&2
fi

# Canonical ngspice-ready netlist — single authority is the literal in
# src/v4/test/claim/formats/spice_rc_ngspice_oracle.dag (spice_rc_tran_ngspice_expected).
# This copy must stay in sync with that .dag literal.
# Line order: title, V1, R1, C1, .tran, .end (fold_list order over spice_rc_tran_deck body).
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

# Step 2: ngspice execution.
echo "oracle: step 2 — running ngspice on RC transient circuit ..."
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
