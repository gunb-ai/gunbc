#!/bin/bash
# inversion-check.sh [--control] <script: K1.sh|K2.sh|-> <old> <new> <case>...
# Builds a reduced harness (run-all.sh's preamble, the J2 setup helpers, the K helpers, and ONLY the named `#case` blocks),
# copies the packet into a temp dir, applies ONE literal mutation (old -> new, which must occur exactly once) to the named
# script, and runs the cases against it. Prints each case's PASS/FAIL line; exit 0 iff EVERY case FAILED (the mutation was
# caught). With --control the mutation is skipped and exit 0 iff every case PASSED. Local only; nothing outside temp dirs.
set -u; H="$(cd "$(dirname "$0")" && pwd)"; P0="$(cd "$H/.." && pwd)"; CTRL=0; [ "${1:-}" = --control ] && { CTRL=1; shift; }
S=$1; OLD=$2; NEW=$3; shift 3; PM="$(mktemp -d)"; trap 'rm -rf -- "$PM"' EXIT
cp "$P0"/K1.sh "$P0"/K2.sh "$P0"/ld449-r2-j2.sh "$PM"/ || exit 3
if [ $CTRL = 0 ] && [ "$S" != - ]; then python3 - "$PM/$S" "$OLD" "$NEW" <<'PY' || exit 3
import sys; p,a,b=sys.argv[1:4]; s=open(p).read(); n=s.count(a)
if n!=1: print("MUTATION NOT APPLIED: old text occurs %d times"%n); sys.exit(1)
open(p,'w').write(s.replace(a,b))
PY
fi
R="$PM/reduced.sh"; A="$H/run-all.sh"
{ awk -v pm="$PM" '/^# ---- frame \+ join-law/{exit} index($0, "P=\"$(cd ")==1 {print "P=" pm "; BASE=\"$(mktemp -d)\"; FAILS=0"; next} {print}' "$A"
  awk '/^# ---- J2: fake root/{f=1} f{print} f && /^run2\(\) \{/{exit}' "$A"
  awk '/^# ---- K1\/K2 \(root acts/{f=1} /^#case /{exit} f{print}' "$A"
  for c in "$@"; do awk -v c="#case $c" '$0==c{f=1; print; next} f && (/^#case / || /^#endcases/){exit} f{print}' "$A"; done
  echo 'echo "REDUCED FAILS=$FAILS"; exit $FAILS'; } > "$R"
grep -q "^P=$PM; " "$R" || { echo "reduced harness: P not redirected"; exit 3; }
o="$(bash "$R" 2>&1)"; printf '%s\n' "$o" | grep -E ' (PASS|FAIL) \(|MUTATION|REDUCED'
np=$(printf '%s\n' "$o" | grep -c ' PASS ('); nf=$(printf '%s\n' "$o" | grep -c ' FAIL (')
if [ $CTRL = 1 ]; then [ $nf = 0 ] && [ $np = $# ]; else [ $np = 0 ] && [ $nf = $# ]; fi
