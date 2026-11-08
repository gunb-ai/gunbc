#!/usr/bin/env bash
# Independent producer->grammar sweep for wc-probe-v12 (keeper-style audit, local bash only; no gunbc, no claim_executor).
set -u
die2() { echo "DIE: $*" >&2; return 2; }   # never exits during sweep
PFN=wc_verdict_probe_reports
source ./probe-fns-v12.sh
A=0; REJ=0; NEGOK=0; NEGBAD=0
run() { # kind accept|refuse  reason...
  local want="$1"; shift
  local line; line="$(printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: %s)\n' "$PFN" "$1")"
  local tmp; tmp="$(mktemp)"; printf '%s' "$line" > "$tmp"
  # probe_verdict sets PROBE_REASON from the typed line (extractor path == runtime path)
  PROBE_REASON=""
  probe_verdict "$tmp" "$PFN" >/dev/null 2>&1 || true
  rm -f "$tmp"
  local reason="$PROBE_REASON"
  probe_validate "$reason" >/dev/null 2>&1; local rc=$?
  if { [ "$want" = accept ] && [ $rc -eq 0 ]; } || { [ "$want" = refuse ] && [ $rc -ne 0 ]; }; then
    [ "$want" = accept ] && A=$((A+1)) || NEGOK=$((NEGOK+1))
  else
    [ "$want" = accept ] && REJ=$((REJ+1)) || NEGBAD=$((NEGBAD+1))
    echo "SURPRISE($want): [$1] -> rc=$rc reason=[$reason]"
  fi
}
I64=9223372036854775807
RSF=resolve_reason_observation_incomplete
# --- positive producer-emittable space ---
# ctx=OK: ctl fields over r=0..23|X (others at declared 0/X)
for v in 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 X; do
  run accept "WCVERIFY ctx=OK ctl_ok=r=$v ctl_out=r=X arm=R rs=- field=R rs=- binder=R rs=- body=$(probe_token 0 U T 0) rs=$RSF"
  run accept "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=$v arm=R rs=- field=R rs=- binder=R rs=- body=$(probe_token 0 U T 0) rs=$RSF"
done
# per-module X tokens: r sweep, a sweep, s sweep, al sweep, at other positions fixed
for pos in arm field binder body; do
  for v in 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 X; do
    toks=("$v"); for a in U ?; do for s in T F; do for al in 0 1 9 10 13 $I64; do toks+=("$(probe_token "$v" "$a" "$s" "$al")"); done; done; done
    toks+=("$(probe_token "$v" - F 0)")   # a=- only with al=0
    for t in "${toks[@]}"; do
      mk() { local o=(); for p in arm field binder body; do if [ "$p" = "$pos" ]; then o+=("$p=$t rs=$RSF"); else o+=("$p=R rs=-"); fi; done; printf 'WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X %s\n' "${o[*]}"; }
      run accept "$(mk "$t")"
    done
  done
done
# rs vocabulary sweep on one X token per position + RF row with absent rs
for n in "${CENSUS_CHAIN24[@]}" "${CENSUS_LANE37[@]}"; do
  t="$(probe_token 7 U T 9)"
  printf -v row 'WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=%s rs=%s field=R rs=- binder=R rs=- body=R rs=-' "$t" "$n"
  run accept "$row"
done
run accept "WCVERIFY ctx=RF arm=- rs=- field=- rs=- binder=- rs=- body=- rs=-"
# --- negatives: must REFUSE (fail-closed audit incl newline/braces/comma/paren in rs) ---
t="$(probe_token 3 U T 1)"
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=$t rs=- field=R rs=- binder=R rs=- body=R rs=-"                     # named token carrying rs=-
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs=ABCD field=R rs=- binder=R rs=- body=R rs=-"                     # uppercase rs
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs=$(printf 'a%.0s' {1..65}) field=R rs=- binder=R rs=- body=R rs=-" # 65 chars
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs= field=R rs=- binder=R rs=- body=R rs=-"                        # empty rs
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs=a,b field=R rs=- binder=R rs=- body=R rs=-"                      # comma in rs
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs=$(printf 'a\nb') field=R rs=- binder=R rs=- body=R rs=-"          # NEWLINE in rs
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs='a}b' field=R rs=- binder=R rs=- body=R rs=-"                    # brace in rs
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs='a)b' field=R rs=- binder=R rs=- body=R rs=-"                    # paren in rs
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs=- field=R rs=- binder=R rs=- body=$t"                            # X token missing rs
run refuse "WCVERIFY ctx=OK ctl_ok=r=24 ctl_out=r=X arm=R rs=- field=R rs=- binder=R rs=- body=$t rs=$RSF"                   # ctl_ok=24
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=24 arm=R rs=- field=R rs=- binder=R rs=- body=$t rs=$RSF"                   # ctl_out=24
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=X{r=24,a=U,s=T,al=0} rs=$RSF field=R rs=- binder=R rs=- body=R rs=-"  # r=24
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=X{r=03,a=U,s=T,al=0} rs=$RSF field=R rs=- binder=R rs=- body=R rs=-"  # leading zero
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=X{r=3,a=-,s=T,al=5} rs=$RSF field=R rs=- binder=R rs=- body=R rs=-"   # a=- al!=0
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs=- field=R rs=- binder=R rs=- body=R rs=- extra"                  # stray field
run refuse "WCVERIFY ctx=RF arm=- rs=- field=- rs=- binder=- rs=- body=$t rs=$RSF"                                            # RF with a named body
run refuse "WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs=- field=R rs=- binder=R rs=-"                                    # missing module
run refuse "WCVERIFY ctx=QQ ctl_ok=r=0 ctl_out=r=X arm=R rs=- field=R rs=- binder=R rs=- body=R rs=-"                        # bad ctx
echo "POS accepted=$A pos-unexpectedly-refused=$REJ | NEG refused-ok=$NEGOK neg-accepted=$NEGBAD"
[ "$REJ" = 0 ] && [ "$NEGBAD" = 0 ] && echo SWEEP_HELD || echo SWEEP_FAILED
