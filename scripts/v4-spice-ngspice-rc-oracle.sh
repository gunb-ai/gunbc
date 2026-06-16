#!/usr/bin/env bash
# scripts/v4-spice-ngspice-rc-oracle.sh
#
# ngspice execution oracle — RC transient circuit (T-4.10 clear-win receipt).
#
# Two-step consumer chain (INVARIANTS.md E-10):
#
#   Step 1 — emit correctness: gunbc --claim-run on spice_rc_ngspice_oracle.dag.
#     spice_rc_ngspice_op_holds() returns Bool; --claim-run exit code is the
#     authority (exit 0 = true, exit 1 = false). A broken spice_emit_ngspice
#     exits 1 here before ngspice ever runs.
#
#   Step 2 — ngspice execution: reads spice_rc_tran_deck_ngspice_golden from the
#     fixture .dag (single string authority). Step 1 must pass before ngspice runs.
#
# Exit codes:
#   0  — emit correct + ngspice simulated to completion
#   1  — emit mismatch OR ngspice failed
#   2  — setup missing (gunbc/ngspice) when strict mode is on
#   77 — ngspice or gunbc not found (SKIP; non-strict local dev)
#
# Env:
#   NGSPICE        — ngspice binary (default: ngspice)
#   GUNBC          — gunbc binary (default: path resolution order below)
#   ORACLE_VERBOSE — set to 1 for full ngspice stdout
#   V4_SPICE_NGSPICE_RC_ORACLE_STRICT — if 1, exit non-zero when ngspice/gunbc missing

set -euo pipefail

ngspice_bin="${NGSPICE:-ngspice}"
verbose="${ORACLE_VERBOSE:-0}"
strict="${V4_SPICE_NGSPICE_RC_ORACLE_STRICT:-${GITHUB_ACTIONS:+1}}"
strict="${strict:-0}"

if ! command -v "$ngspice_bin" >/dev/null 2>&1; then
  echo "error: ngspice not found (set NGSPICE= to override)." >&2
  if [[ "$strict" == "1" ]]; then
    echo "::error title=spice ngspice rc oracle setup::ngspice missing"
    exit 2
  fi
  echo "SKIP: ngspice not found (set NGSPICE= to override)." >&2
  exit 77
fi

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

# Resolve gunbc: env override > repo target/release (build if needed) > system PATH
if [[ -n "${GUNBC:-}" ]]; then
  gunbc_bin="$GUNBC"
elif [[ -x "$root/target/release/gunbc" ]]; then
  gunbc_bin="$root/target/release/gunbc"
else
  if [[ ! -x "$root/target/release/gunbc" ]]; then
    (cd "$root" && cargo build -p v2-compiler --release --bin gunbc)
  fi
  if [[ -x "$root/target/release/gunbc" ]]; then
    gunbc_bin="$root/target/release/gunbc"
  elif command -v gunbc >/dev/null 2>&1; then
    gunbc_bin="gunbc"
  else
    echo "error: gunbc not found; cannot verify emit correctness." >&2
    if [[ "$strict" == "1" ]]; then
      echo "::error title=spice ngspice rc oracle setup::gunbc missing"
      exit 2
    fi
    echo "SKIP: gunbc not found; cannot verify emit correctness." >&2
    exit 77
  fi
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
echo "oracle: step 1 PASS — spice_emit_ngspice output matches spice_rc_tran_deck_ngspice_golden."

fixture_dag="$root/src/v4/test/fixture/spice_rc_tran_deck.dag"

# Step 2: ngspice execution — netlist bytes from the single golden authority in fixture.
# 🟡 scaffold — feature:gunbc-golden-bytes-subcommand — regex over .dag source; dissolve-on-arrival:
# replace with gunbc-evaluated golden literal export (avoids parallel parser fragility).
netlist="$(python3 - "$fixture_dag" <<'PY'
import re
import sys
from pathlib import Path

src = Path(sys.argv[1]).read_text()
match = re.search(
    r'data spice_rc_tran_deck_ngspice_golden: String = "((?:\\.|[^"\\])*)"',
    src,
    re.DOTALL,
)
if match is None:
    sys.stderr.write("oracle: FAIL — spice_rc_tran_deck_ngspice_golden not found in fixture\n")
    sys.exit(1)
print(bytes(match.group(1), "utf-8").decode("unicode_escape"), end="")
PY
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
