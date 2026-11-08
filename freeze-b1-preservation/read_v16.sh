#!/usr/bin/env bash
# Mechanical v15 reading driver. Usage: read_v15.sh <tag> <claims-log> [<tag> <claims-log> ...]
# e.g. read_v15.sh A claims-A-probe.log B claims-B-probe.log
# Inputs ONLY from the keeper's bounded read; verify the keeper's shas before running.
# Frozen authority: packet packet-evidence-v15.sh (sha posted on PR #15999); PRE_REGISTRATION_v15.md governs.
# NAME DUTY (v15, widened per PRE_REG R5): EVERY decimal r in 0..36 on any module must be named by both
# readers from the CENSUS37 table embedded in the packet script; a mismatch voids THAT module's reading.
set -o pipefail
[ "$#" -ge 2 ] && [ $(( $# % 2 )) = 0 ] || { echo "usage: read_v15.sh <tag> <log> [...]"; exit 2; }
PFN=wc_verdict_probe_reports
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/probe-kit-lib.sh" >/dev/null 2>&1 || true
mapfile -t CENSUS37 < <(sed -n 's/^CENSUS37=(\(.*\)) *#.*/\1/p' "$HERE/packet-evidence-v15.sh" | tr ' ' '\n' | sed '/^$/d')
[ "${#CENSUS37[@]}" = 37 ] || { echo "KIT DEFECT: CENSUS37 from packet-evidence-v15.sh not 37 unique rows"; exit 2; }
OVERALL=0
while [ "$#" -ge 2 ]; do
  TAG="$1"; LOG="$2"; shift 2
  echo "===== LEG $TAG ($LOG) ====="
  [ -f "$LOG" ] || { echo "NO OBSERVATION (exit 2): log file missing"; OVERALL=2; continue; }
  mapfile -t ALL < <(grep -E '^[A-Z]+ ' "$LOG" || true)
  if [ "${#ALL[@]}" != 1 ]; then echo "NO OBSERVATION (exit 2): join law — expected exactly one ALLCAPS line, got ${#ALL[@]}"; printf '%s\n' "${ALL[@]}"; OVERALL=2; continue; fi
  L0="${ALL[0]}"
  case "$L0" in "FAIL $PFN (returned ProcessExit::ExitFailure, code 3: "*) :;; *) echo "NO OBSERVATION (exit 2): the single ALLCAPS line is not the typed code-3 FAIL"; echo "$L0"; OVERALL=2; continue;; esac
  for d in FX RTD RS PGD R RUNDIR; do declare "$d=$(mktemp -d)"; done
  PROBE_REASON=""
  printf '%s' "$L0" > "$RS/typed.txt"
  # v14 kit semantics: probe_verdict -> probe_classify sets PROBE_REASON in the CURRENT shell, but it STRIPS the
  # final character (the last closing brace) from the reason; probe_validate takes the reason as $1.
  # Both are called WITHOUT $( ... ) so the assignment survives.
  probe_verdict "$RS/typed.txt" "$PFN" >/dev/null 2>&1
  [ -n "$PROBE_REASON" ] || { echo "NO OBSERVATION (exit 2): probe_verdict set no PROBE_REASON"; OVERALL=2; continue; }
  probe_validate "$PROBE_REASON" >/dev/null 2>&1 || { echo "NO OBSERVATION (exit 2): probe_validate refused (kind=${PROBE_VALIDATE_KIND:-?})"; OVERALL=2; continue; }
  case "$PROBE_REASON" in *"}") :;; *) PROBE_REASON="$PROBE_REASON}";; esac   # restore the classify-stripped final brace for per-module extraction
  echo "== VALIDATED typed reason =="; echo "$PROBE_REASON"
  echo "== BASH READER (frozen path) =="
  B="$({ probe_read_line "$PROBE_REASON"; })"; rc_b=$?
  printf '%s\n' "$B"
  if [ "$rc_b" != 0 ]; then echo "BASH READER rc=$rc_b -> NO READING"; OVERALL=2; continue; fi
  echo "== PYTHON READER (embedded bytes) =="
  printf '%s\n' "$PROBE_REASON" > "$RS/reason.txt"
  P="$(python3 "$HERE/probe_read.py.v14" "$RS/reason.txt")"; rc_p=$?
  printf '%s\n' "$P"
  if [ "$rc_p" != 0 ]; then echo "PYTHON READER rc=$rc_p -> NO READING"; OVERALL=2; continue; fi
  nb="$(printf '%s\n' "$B" | tr -s '[:space:]' ' ')"; np="$(printf '%s\n' "$P" | tr -s '[:space:]' ' ')"
  if [ "$nb" = "$np" ]; then echo "READERS AGREE (normalized byte-equal)"; else echo "READERS DISAGREE -> report raw bytes; conclude NOTHING"; diff <(printf '%s\n' "$B") <(printf '%s\n' "$P") || true; OVERALL=2; continue; fi
  case "$PROBE_REASON" in *"ctl_ok=r=0 ctl_out=r=X"*) echo "CONTROL GATE: HELD (ctl_ok=r=0, ctl_out=r=X)";; *) echo "CONTROL GATE: FAILED -> VOID; nothing concluded"; OVERALL=2; continue;; esac
  echo "== NAME DUTY (v14-frozen window r=24..36: both readers must name CENSUS37[r]; other decimal r are table-named by the kit, informational) =="
  duty=0
  for mod in arm field binder body; do
    # pure-bash extraction (no sed quoting traps): token after "mod=", r value after '{' up to first ','
    _rest="${PROBE_REASON#*"$mod"=}"
    _tok="${_rest%%\}*}"
    _r="${_tok#*\{}"; _r="${_r%%\}*}"; _r="${_r%%,*}"; _r="${_r#r=}"
    r="$_r"; tok="$_tok"
    case "$r" in ''|X) echo "  $mod: r=$r (no name duty; X is never a verdict)";;
      *) want="${CENSUS37[$r]:-TABLE-MISSING}"
         if [ "$r" -ge 24 ] 2>/dev/null; then
           duty=1
           if printf '%s\n' "$B" | grep -qF "$want"; then bnamed=yes; else bnamed=no; fi
           if printf '%s\n' "$P" | grep -qF "$want"; then pnamed=yes; else pnamed=no; fi
           echo "  $mod: r=$r -> table=$want bash_named=$bnamed python_named=$pnamed"
           [ "$bnamed" = yes ] && [ "$pnamed" = yes ] || { echo "  NAME DUTY VIOLATION on $mod -> that module's reading is VOID"; OVERALL=2; }
         else
           echo "  $mod: r=$r -> CENSUS37 row is $want (informational only; outside the NAME DUTY window 24..36, the reader carries no naming duty here)"
         fi;;
    esac
  done
  [ "$duty" = 1 ] || echo "  (no in-37 r values on this leg)"
  for d in FX RTD RS PGD R RUNDIR; do eval "[ -n \"\${$d:-}\" ] && rm -rf -- \"$d\""; done
done
echo "READ_V15 OVERALL rc=$OVERALL"
exit "$OVERALL"
