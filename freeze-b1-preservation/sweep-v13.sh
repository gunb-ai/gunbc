#!/usr/bin/env bash
# Independent producer->grammar sweep for wc-probe-v13 (keeper-style; local bash only; no gunbc, no claim_executor).
set -u
die2() { echo "DIE: $*" >&2; return 2; }
PFN=wc_verdict_probe_reports
FX=$(mktemp -d); RTD=$(mktemp -d); RS=$(mktemp -d); PGD=$(mktemp -d); R=$(mktemp -d); RUNDIR=$(mktemp -d)
export RANDBACK=0
source ./probe-fns-v13.sh
A=0; REJ=0; NEGOK=0; NEGBAD=0
run() { # accept|refuse  reason-line-suffix (the WCVERIFY... string)
  local want="$1"; shift
  local line; line="$(printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: %s)\n' "$PFN" "$1")"
  local tmp; tmp="$(mktemp)"; printf '%s' "$line" > "$tmp"
  PROBE_REASON=""; probe_verdict "$tmp" "$PFN" >/dev/null 2>&1 || true; rm -f "$tmp"
  probe_validate "$PROBE_REASON" >/dev/null 2>&1; local rc=$?
  if { [ "$want" = accept ] && [ $rc -eq 0 ]; } || { [ "$want" = refuse ] && [ $rc -ne 0 ]; }; then
    [ "$want" = accept ] && A=$((A+1)) || NEGOK=$((NEGOK+1))
  else
    [ "$want" = accept ] && REJ=$((REJ+1)) || NEGBAD=$((NEGBAD+1))
    echo "SURPRISE($want): rc=$rc reason=[$PROBE_REASON]"
  fi
}
I64=9223372036854775807
RSF=resolve_reason_observation_incomplete
mkrow() { # ctl1 ctl2 arm field binder body (tokens already carry their rs)
  printf 'WCVERIFY ctx=OK ctl_ok=r=%s ctl_out=r=%s arm=%s field=%s binder=%s body=%s' "$1" "$2" "$3" "$4" "$5" "$6"
}
# positives: ctl sweeps
for v in 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 X; do
  run accept "$(mkrow "$v" X 'R rs=-' 'R rs=-' 'R rs=-' "$(probe_token 0 U T 0 resolve_reason_unbound_symbol)")"
  run accept "$(mkrow 0 "$v" 'R rs=-' 'R rs=-' 'R rs=-' "$(probe_token 0 U T 0 resolve_reason_unbound_symbol)")"
done
# positives: per-module token space (other modules plain)
for pos in arm field binder body; do
  for v in 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 X; do
    toks=(); for a in U ?; do for s in T F; do for al in 0 1 9 10 13 $I64; do toks+=("$(probe_token "$v" "$a" "$s" "$al" "$( [ "$v" = X ] && echo "$RSF" || echo "${CENSUS_CHAIN24[$v]}")")"); done; done; done
    toks+=("$(probe_token "$v" - F 0 "$( [ "$v" = X ] && echo "$RSF" || echo "${CENSUS_CHAIN24[$v]}")")")
    for t in "${toks[@]}"; do
      o=(); for p in arm field binder body; do if [ "$p" = "$pos" ]; then o+=("$t"); else o+=("R rs=-"); fi; done
      run accept "$(mkrow 0 X "${o[0]}" "${o[1]}" "${o[2]}" "${o[3]}")"
    done
  done
done
# positives: rs vocabulary sweep (all 24 chain + 37 lane names on one X token, per position)
for n in "${CENSUS_LANE37[@]}" "${CENSUS_CHAIN24[@]}"; do
  for pos in arm field binder body; do
    case "$pos" in arm) idx=0;; field) idx=1;; binder) idx=2;; body) idx=3;; esac
    o=('R rs=-' 'R rs=-' 'R rs=-' 'R rs=-'); o[$idx]="$(probe_token X - T 0 "$n")"
    run accept "$(mkrow 0 X "${o[0]}" "${o[1]}" "${o[2]}" "${o[3]}")"
  done
done
run accept "WCVERIFY ctx=RF ctl_ok=r=0 ctl_out=r=X arm=- rs=- field=- rs=- binder=- rs=- body=- rs=-"
# negatives: rs abuse (must REFUSE)
t0="$(probe_token 3 U T 1 resolve_reason_lexical_reference_unkeyed)"
run refuse "$(mkrow 0 X "$t0 rs=-" 'R rs=-' 'R rs=-' 'R rs=-')"
run refuse "$(mkrow 0 X 'R rs=ABCD' 'R rs=-' 'R rs=-' 'R rs=-')"
run refuse "$(mkrow 0 X "R rs=$(printf 'a%.0s' {1..65})" 'R rs=-' 'R rs=-' 'R rs=-')"
run refuse "$(mkrow 0 X 'R rs=' 'R rs=-' 'R rs=-' 'R rs=-')"
run refuse "$(mkrow 0 X 'R rs=a,b' 'R rs=-' 'R rs=-' 'R rs=-')"
run refuse "$(mkrow 0 X "R rs=$(printf 'a\nb')" 'R rs=-' 'R rs=-' 'R rs=-')"
run refuse "$(mkrow 0 X 'R rs=a}b' 'R rs=-' 'R rs=-' 'R rs=-')"
run refuse "$(mkrow 0 X 'R rs=a{b' 'R rs=-' 'R rs=-' 'R rs=-')"
run refuse "$(mkrow 0 X 'R rs=a)b' 'R rs=-' 'R rs=-' 'R rs=-')"
run refuse "$(mkrow 0 X "${t0% rs=*}" 'R rs=-' 'R rs=-' 'R rs=-')"
run refuse "WCVERIFY ctx=OK ctl_ok=r=24 ctl_out=r=X arm=R rs=- field=R rs=- binder=R rs=- body=$t0"
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=24 arm=R rs=- field=R rs=- binder=R rs=- body=$t0"
run refuse "$(mkrow 0 X "X{r=24,a=U,s=T,al=0} rs=$RSF" 'R rs=-' 'R rs=-' 'R rs=-')"
run refuse "$(mkrow 0 X "X{r=03,a=U,s=T,al=0} rs=resolve_reason_unbound_symbol" 'R rs=-' 'R rs=-' 'R rs=-')"
run refuse "$(mkrow 0 X "X{r=3,a=-,s=T,al=5} rs=resolve_reason_lexical_reference_unkeyed" 'R rs=-' 'R rs=-' 'R rs=-')"
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs=- field=R rs=- binder=R rs=- body=$t0 extra"
run refuse "WCVERIFY ctx=RF ctl_ok=r=0 ctl_out=r=X arm=- rs=- field=- rs=- binder=- rs=- body=$t0"
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs=- field=R rs=- binder=R rs=-"
run refuse "WCVERIFY ctx=QQ ctl_ok=r=0 ctl_out=r=X arm=R rs=- field=R rs=- binder=R rs=- body=$t0"
run refuse "$(mkrow 0 X 'R rs=-' 'R rs=-' 'R rs=-' 'X{r=5,a=-,s=T,al=0} rs=-')"
echo "POS accepted=$A pos-refused-unexpectedly=$REJ | NEG refused-ok=$NEGOK neg-accepted=$NEGBAD"
[ "$REJ" = 0 ] && [ "$NEGBAD" = 0 ] && echo SWEEP_HELD || echo SWEEP_FAILED
