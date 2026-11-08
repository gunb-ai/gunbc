set -o pipefail
die2() { echo "REFUSED_SETUP: $*"; return 2; }
for d in FX RTD RS PGD R RUNDIR; do declare "$d=$(mktemp -d)"; done
source ./probe-kit-lib.sh >/dev/null 2>&1
ROW="$(cat ./probe-reason.txt)"
echo "len=${#ROW}"
probe_validate "$ROW"; echo "validate rc=$? kind=${PROBE_VALIDATE_KIND}"
B="$({ probe_read_line "$ROW"; })"; rb=$?
printf '%s\n' "$ROW" > ./reason-in.txt; P="$(python3 ./probe_read.py.v14 ./reason-in.txt)"; rp=$?
echo "--- BASH (rc=$rb) ---"; printf '%s\n' "$B"
echo "--- PYTHON (rc=$rp) ---"; printf '%s\n' "$P"
nb="$(printf '%s\n' "$B" | tr -s '[:space:]' ' ')"; np="$(printf '%s\n' "$P" | tr -s '[:space:]' ' ')"
[ "$nb" = "$np" ] && echo "READERS AGREE (normalized byte-equal)" || { echo "READERS DISAGREE"; diff <(printf '%s\n' "$B") <(printf '%s\n' "$P"); }
case "$ROW" in *"ctl_ok=r=0 ctl_out=r=X"*) echo "CONTROL GATE: HELD";; *) echo "CONTROL GATE: FAILED -> VOID";; esac
echo "DUTY: all modules r=X -> NAME DUTY not triggered (no r in 24..36)"
