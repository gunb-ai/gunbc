#!/usr/bin/env bash
# reader-negative-suite-v16.sh — negative-path suite for read_v16.sh (END-phase probe reader).
# Run from the clean v16-only kit dir (read_v16.sh d75ca021 + packet-evidence-v16.sh + probe_read.py.v14 + probe-kit-lib.sh).
# Usage: reader-negative-suite-v16.sh <probe-log-with-typed-code3-line>
# Expected: cases 1-6 rc=2 with the stated refusal; case 7 rc=0. Prints PASS/FAIL per case.
set -u
BASE="${1:?usage: $0 <claims-probe.log>}"; HERE="$(pwd)"; NEG=$(mktemp -d); FAILS=0
runc(){ local want="$2" name="$3"; (cd "$HERE" && bash read_v16.sh A "$1" >"$NEG/o" 2>&1); local rc=$?
  if [ "$rc" = "$want" ]; then echo "PASS [$name] rc=$rc"; else echo "FAIL [$name] rc=$rc want=$want"; sed 's/^/    /' "$NEG/o" | head -4; FAILS=$((FAILS+1)); fi; }
[ -f "$BASE" ] || { echo "suite: base log missing"; exit 2; }
runc /nonexistent.log 2 missing
grep -v '^FAIL wc_verdict' "$BASE" > "$NEG/z";  runc "$NEG/z" 2 zero-lines
cat "$BASE" "$BASE" > "$NEG/d";                 runc "$NEG/d" 2 dup-lines
sed 's/code 3:/code 9:/' "$BASE" > "$NEG/c9";   runc "$NEG/c9" 2 not-typed-code3
sed 's/ctl_ok=r=0/ctl_ok=r=1/' "$BASE" > "$NEG/ctl"; runc "$NEG/ctl" 2 control-failed-VOID
sed 's/field=X{r=X,a=-,s=T,al=0}/field=X{r=X,a=-,s=T,al=0/' "$BASE" > "$NEG/g"; runc "$NEG/g" 2 grammar-garbled
runc "$BASE" 0 happy
rm -rf "$NEG"; echo "SUITE rc=$FAILS"; exit "$FAILS"
