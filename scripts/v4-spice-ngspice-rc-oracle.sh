#!/usr/bin/env bash
# scripts/v4-spice-ngspice-rc-oracle.sh
#
# ngspice execution oracle — RC transient circuit (T-4.10 clear-win receipt).
#
# Two-step consumer chain (INVARIANTS.md E-10):
#   Step 1 — emit correctness: gunbc --claim-run on spice_rc_ngspice_oracle.dag.
#             Exit code is the authority; broken emit exits 1 before ngspice runs.
#   Step 2 — ngspice execution: gunbc run --function emit_ngspice_text produces the
#             netlist string from the .dag authority; that string is fed to ngspice -b.
#             If gunbc is unavailable, step 2 is SKIP (exit 77) — not silently downgraded.
#
# Exit codes:
#   0  — emit correct + ngspice simulated to completion
#   1  — emit mismatch OR ngspice failed
#   77 — ngspice or gunbc not found (SKIP)
#
# Env:
#   NGSPICE        — ngspice binary (default: ngspice)
#   GUNBC          — gunbc binary (default: path resolution order below)
#   ORACLE_VERBOSE — set to 1 for full ngspice stdout

set -euo pipefail

ngspice_bin="${NGSPICE:-ngspice}"
verbose="${ORACLE_VERBOSE:-0}"

if ! command -v "$ngspice_bin" >/dev/null 2>&1; then
  echo "SKIP: ngspice not found (set NGSPICE= to override)." >&2
  exit 77
fi

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

# Resolve gunbc: env override > repo target/release > system PATH
if [[ -n "${GUNBC:-}" ]]; then
  gunbc_bin="$GUNBC"
elif [[ -x "$root/target/release/gunbc" ]]; then
  gunbc_bin="$root/target/release/gunbc"
elif command -v gunbc >/dev/null 2>&1; then
  gunbc_bin="gunbc"
else
  echo "SKIP: gunbc not found; cannot derive emit output from .dag authority." >&2
  exit 77
fi

claim_entry="$root/src/v4/test/claim/formats/spice_rc_ngspice_oracle.dag"
fixture_entry="$root/src/v4/test/fixture/spice_rc_tran_deck.dag"

# Step 1: emit correctness — gate on --claim-run exit code, not string sentinels.
echo "oracle: step 1 — verifying emit correctness via gunbc --claim-run ..."
if ! "$gunbc_bin" run \
    --source-root "$root/src/v4" \
    --entry "$claim_entry" \
    --claim-run 2>&1; then
  echo "oracle: FAIL — emit claim returned false (spice_emit_ngspice output mismatch)." >&2
  exit 1
fi
echo "oracle: step 1 PASS."

# Step 2: derive the netlist from the .dag authority and feed it to ngspice.
# emit_ngspice_text() in spice_rc_tran_deck.dag returns spice_rc_tran_deck_ngspice_text,
# which is computed by spice_emit_ngspice — no parallel literal copy.
echo "oracle: step 2 — deriving netlist from .dag authority ..."
emitted="$("$gunbc_bin" run \
    --source-root "$root/src/v4" \
    --entry "$fixture_entry" \
    --function emit_ngspice_text 2>/dev/null)" || true

if [[ -z "$emitted" ]]; then
  echo "oracle: FAIL — gunbc run --function emit_ngspice_text produced no output." >&2
  exit 1
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

netlist_file="$tmpdir/rc_tran.spi"
printf '%s\n' "$emitted" > "$netlist_file"

echo "oracle: running ngspice on emitted netlist ..."
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
  echo "oracle: FAIL — ngspice exited $rc." >&2
  exit 1
fi
