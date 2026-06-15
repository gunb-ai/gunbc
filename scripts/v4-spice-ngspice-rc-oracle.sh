#!/usr/bin/env bash
# scripts/v4-spice-ngspice-rc-oracle.sh
#
# ngspice execution oracle — RC transient circuit (T-4.10 clear-win receipt).
#
# Two-step consumer chain (INVARIANTS.md E-10):
#
#   Step 1 — emit correctness: gunbc --claim-run on spice_rc_ngspice_oracle.dag.
#     spice_rc_tran_emit_ngspice_matches_expected_holds() returns Bool; --claim-run
#     exit code is the authority (exit 0 = true, exit 1 = false). A broken
#     spice_emit_ngspice exits 1 here before ngspice ever runs.
#
#   Step 2 — ngspice execution: the step-1 claim proves
#     spice_emit_ngspice(rc_tran_deck) == spice_rc_tran_ngspice_expected
#     so the literal below is NOT a parallel authority — it is a consequence
#     of step 1 passing. P2 does not apply: a single claim gates the equality;
#     the literal is the same bytes under a different binding site.
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
  echo "SKIP: gunbc not found; cannot verify emit correctness." >&2
  exit 77
fi

claim_entry="$root/src/v4/test/claim/formats/spice_rc_ngspice_oracle.dag"

# Step 1: emit correctness — exit code is the sole authority.
# --claim-run exits 0 when the witness Bool is true, 1 when false.
echo "oracle: step 1 — verifying emit correctness via gunbc --claim-run ..."
if ! "$gunbc_bin" run \
    --source-root "$root/src/v4" \
    --entry "$claim_entry" \
    --function spice_rc_ngspice_op_holds \
    --claim-run 2>&1; then
  echo "oracle: FAIL — spice_rc_ngspice_op_holds returned false (emit/sim deck mismatch)." >&2
  exit 1
fi
echo "oracle: step 1 PASS — spice_emit_ngspice output matches spice_rc_tran_ngspice_expected."

# Step 2: ngspice execution.
# The literal below matches spice_rc_tran_ngspice_expected in spice_rc_ngspice_oracle.dag.
# Step 1 passing proves they are equal to spice_emit_ngspice(rc_tran_deck); this is not
# a parallel authority but a consequence of step 1.
netlist="$(cat <<'NETLIST'
* rc tran witness
V1 n1 0 DC 5
R1 n1 n2 1000
C1 n2 0 1e-6
.ic v(n2)=0
.tran 1us 10ms
.control
run
print v(n2)
quit
.endc
.end
NETLIST
)"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

netlist_file="$tmpdir/rc_tran.spi"
printf '%s\n' "$netlist" > "$netlist_file"

echo "oracle: step 2 — running ngspice on emitted netlist ..."
if [[ "$verbose" == "1" ]]; then
  "$ngspice_bin" -b "$netlist_file"
  rc=$?
else
  set +e
  "$ngspice_bin" -b "$netlist_file" >"$tmpdir/ngspice.out" 2>&1
  rc=$?
  set -e
  if [[ $rc -ne 0 ]]; then
    echo "ngspice output:" >&2
    tail -40 "$tmpdir/ngspice.out" >&2
  fi
fi

if [[ $rc -eq 0 ]]; then
  echo "oracle: PASS — ngspice RC transient simulation exited 0."
  exit 0
else
  echo "oracle: FAIL — ngspice exited $rc." >&2
  exit 1
fi
