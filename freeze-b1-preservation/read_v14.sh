#!/usr/bin/env bash
# Mechanical v14 reading driver (POST_RUN step 3). Usage: read_v14.sh <claims-log>
# Inputs ONLY from the keeper's bounded read; verify shas before running. Frozen authority: packet 65b3b9d1...
set -o pipefail
LOG="$1"
[ -f "$LOG" ] || { echo "NO OBSERVATION (exit 2): log file missing"; exit 2; }
PFN=wc_verdict_probe_reports
mapfile -t ALL < <(grep -E '^[A-Z]+ ' "$LOG" || true)
if [ "${#ALL[@]}" != 1 ]; then echo "NO OBSERVATION (exit 2): join law — expected exactly one ALLCAPS line, got ${#ALL[@]}"; printf '%s\n' "${ALL[@]}"; exit 2; fi
L0="${ALL[0]}"
case "$L0" in "FAIL $PFN (returned ProcessExit::ExitFailure, code 3: "*) :;; *) echo "NO OBSERVATION (exit 2): the single ALLCAPS line is not the typed code-3 FAIL"; echo "$L0"; exit 2;; esac
die2() { echo "REFUSED_SETUP: $*"; return 2; }
for d in FX RTD RS PGD R RUNDIR; do declare "$d=$(mktemp -d)"; done
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/probe-kit-lib.sh" >/dev/null 2>&1 || true
PROBE_REASON=""
printf '%s' "$L0" > "$RS/typed.txt"
probe_verdict "$RS/typed.txt" "$PFN" >/dev/null 2>&1 || { echo "NO OBSERVATION (exit 2): probe_verdict failed to extract a typed token"; exit 2; }
if ! probe_validate "$PROBE_REASON" >/dev/null 2>&1; then
  K="$(probe_validate "$PROBE_REASON" 2>/dev/null)"; echo "NO OBSERVATION (exit 2): probe_validate refused (${K:-rc!=0})"; exit 2
fi
echo "== VALIDATED typed reason =="; echo "$PROBE_REASON"
echo "== BASH READER (frozen path) =="
B="$({ probe_read_line "$PROBE_REASON"; })"; rc_b=$?
printf '%s\n' "$B"
if [ "$rc_b" != 0 ]; then echo "BASH READER rc=$rc_b -> NO READING"; exit 2; fi
echo "== PYTHON READER (embedded bytes, sha-verified at prep) =="
printf '%s\n' "$PROBE_REASON" > "$RS/reason.txt"
P="$(python3 "$HERE/probe_read.py.v14" "$RS/reason.txt")"; rc_p=$?
printf '%s\n' "$P"
if [ "$rc_p" != 0 ]; then echo "PYTHON READER rc=$rc_p -> NO READING"; exit 2; fi
echo "== READER AGREEMENT (mechanical) =="
nb="$(printf '%s\n' "$B" | tr -s '[:space:]' ' ')"; np="$(printf '%s\n' "$P" | tr -s '[:space:]' ' ')"
if [ "$nb" = "$np" ]; then echo "READERS AGREE (normalized byte-equal)"; else echo "READERS DISAGREE -> report raw bytes; conclude NOTHING"; diff <(printf '%s\n' "$B") <(printf '%s\n' "$P") || true; exit 2; fi
echo "== CONTROL GATE =="
case "$PROBE_REASON" in *"ctl_ok=r=0 ctl_out=r=X"*) echo "CONTROL GATE: HELD (ctl_ok=r=0, ctl_out=r=X)";; *) echo "CONTROL GATE: FAILED -> VOID; nothing concluded"; exit 2;; esac
echo "== NAME DUTY (r=24..36): readers' named symbols vs the frozen table =="
mapfile -t CENSUS37 < <(sed -n 's/^CENSUS37=(\(.*\)) *#.*/\1/p' "$HERE/packet-wc-probe-v14.sh" | tr ' ' '\n' | sed '/^$/d')
[ "${#CENSUS37[@]}" = 37 ] || { echo "KIT DEFECT: table not 37"; exit 2; }
duty=0
for tok in $(printf '%s\n' "$PROBE_REASON" | grep -oE '[RAX]\{r=([0-9]+|X),[a-zA-Z0-9=,-]*\}' ); do
  r="${tok#*r=}"; r="${r%%,*}"
  case "$r" in X|0|1|2|3|4|5|6|7|8|9|1[0-9]|2[0-3]) continue;; esac
  duty=1; want="${CENSUS37[$r]:-TABLE-MISSING}"
  printf '%s\n' "$B" | grep -qF "$want" && printf '%s\n' "$P" | grep -qF "$want" \
    && echo "DUTY r=$r: BOTH readers name $want (matches frozen table)" \
    || { echo "DUTY r=$r: NAME CHECK FAILED (want $want) -> conclude NOTHING"; exit 2; }
done
[ "$duty" = 0 ] && echo "DUTY: no r in 24..36 observed; duty not triggered"
echo "== READING READY =="
