#!/usr/bin/env bash
# Mechanical v13 reading driver (POST_RUN step 3). Usage: read_v13.sh <claims-log> <outcome-line-file>
# Inputs come ONLY from the keeper's bounded read (verify shas before running).
set -uo pipefail
LOG="$1"; LINE="$1"
[ -f "$LOG" ] || { echo "NO OBSERVATION (exit 2): log file missing"; exit 2; }
# Join law: exactly one '^[A-Z]+ ' line, equal to 'FAIL wc_verdict_probe_reports (...)' with code 3
mapfile -t ALL < <(grep -E '^[A-Z]+ ' "$LOG" || true)
if [ "${#ALL[@]}" != 1 ]; then echo "NO OBSERVATION (exit 2): join law — expected exactly one ALLCAPS line, got ${#ALL[@]}"; printf '%s\n' "${ALL[@]}"; exit 2; fi
L0="${ALL[0]}"
case "$L0" in "FAIL wc_verdict_probe_reports (returned ProcessExit::ExitFailure, code 3: "*) :;; *) echo "NO OBSERVATION (exit 2): the single ALLCAPS line is not the typed code-3 FAIL"; echo "$L0"; exit 2;; esac
# rc cross-check is done against the keeper-reported claim_batch rc (see driver stderr/receipt):
#   rc must be 1 for a FAIL-only roster. The caller records it in the report; here we gate the READING.
# Typed reason extraction + validation via the frozen packet functions (extracted region, die2-stubbed)
die2() { echo "REFUSED_SETUP: $*"; return 2; }
PFN=wc_verdict_probe_reports
for d in FX RTD RS PGD R RUNDIR; do declare "$d=$(mktemp -d)"; done
source ./probe-fns-v13.sh >/dev/null 2>&1 || true
PROBE_REASON=""
printf '%s' "$L0" > "$RS/typed.txt"
probe_verdict "$RS/typed.txt" "$PFN" >/dev/null 2>&1 || { echo "NO OBSERVATION (exit 2): probe_verdict failed to extract a typed token"; exit 2; }
if probe_validate "$PROBE_REASON" >/dev/null 2>&1; then :; else
  K="$(probe_validate "$PROBE_REASON" 2>/dev/null)"; echo "NO OBSERVATION (exit 2): probe_validate refused (${K:-rc!=0})"; exit 2
fi
echo "== VALIDATED typed reason =="; echo "$PROBE_REASON"
echo "== SHELL READER =="
probe_read_line "$PROBE_REASON"
echo "== PYTHON READER =="
printf '%s\n' "$PROBE_REASON" > "$RS/reason.txt"; python3 ./probe_read.py.v13 "$RS/reason.txt"
echo "== GATE SUMMARY (mechanical) =="
case "$PROBE_REASON" in
  *"ctl_ok=r=0 ctl_out=r=X"*) echo "CONTROL GATE: HELD (ctl_ok=r=0, ctl_out=r=X)";;
  *) echo "CONTROL GATE: FAILED -> VOID; nothing concluded";;
esac
if printf '%s\n' "$(probe_read_line "$PROBE_REASON" 2>/dev/null)" | grep -q 'CONSISTENCY VERDICT'; then
  echo "CONSISTENCY GATE: MISMATCH -> claims/falsifier/table VOID; report raw data only"
else
  echo "CONSISTENCY GATE: consistent (or control-void)"
fi
