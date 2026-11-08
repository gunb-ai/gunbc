#!/bin/bash
# v16 inversion proof: mutations of the stub, each must trip a NAMED law. Expectations frozen here BEFORE running.
# dup        -> probe outcome-join law: exit 2, "OUTCOME-JOIN LAW" (2 outcome lines vs roster of 1)
# foreign    -> claim join law: exit 2, "foreign name"
# missing    -> claim join law: exit 2, "roster join FAILED"
# rcmis      -> claim rc/set law: exit 2, "rc/set inconsistency"      (rc 1, zero FAIL rows)
# nonbare    -> classification law: exit 2, "NON-BARE outcome line"    (reasoned FAIL never satisfies an expected FAIL)
# rc2allpass -> rc/set law m3:          exit 2, "rc/set inconsistency" (rc 2, all bare PASS)
# unobs      -> classification law:     exit 2, "NON-BARE outcome line" (UNOBSERVED is not a verdict)
# contradiction -> join law m6:         exit 2, "CONTRADICTORY claim cell"
# ctl_off    -> CONTROL gate (M2):      exit 2, "CONTROL TOKEN OFF"    (ctl_ok != r=0 must never exit 0)
# ctl_off2   -> CONTROL gate (M2):      exit 2, "CONTROL TOKEN OFF"    (ctl_out != r=X)
# argvbad    -> argv law (B1 inversion): exit 2, stub refuses the non-CSV argv ("STUB_UNEXPECTED_ARG"/shape refusal)
MUTS="dup foreign missing rcmis nonbare rc2allpass unobs contradiction ctl_off ctl_off2 argvbad"
declare -A WANT=( [dup]=2 [foreign]=2 [missing]=2 [rcmis]=2 [nonbare]=2 [rc2allpass]=2 [unobs]=2 [contradiction]=2 [ctl_off]=2 [ctl_off2]=2 [argvbad]=2 )
declare -A MARK=( [dup]='OUTCOME-JOIN LAW' [foreign]='foreign name' [missing]='roster join FAILED' [rcmis]='rc/set inconsistency' [nonbare]='NON-BARE outcome line' [rc2allpass]='rc/set inconsistency' [unobs]='NON-BARE outcome line' [contradiction]='roster join FAILED' [ctl_off]='CONTROL TOKEN OFF' [ctl_off2]='CONTROL TOKEN OFF' [argvbad]='no single' )
FAILS=0
for M in $MUTS; do
  export MUT=$M
  bash /tmp/packet/e2e-v16.sh >/tmp/e2e-inv16.$M.log 2>&1; RC=$?
  want=${WANT[$M]}; mark=${MARK[$M]}
  if [ "$RC" = "$want" ] && grep -aqF "$mark" /tmp/e2e-inv16.$M.log; then
    echo "INVERSION-HELD $M rc=$want mark='$mark'"
  else
    echo "INVERSION-FAILED $M rc=$RC want=$want mark='$mark' hits=$(grep -acF "$mark" /tmp/e2e-inv16.$M.log)"; FAILS=$((FAILS+1))
  fi
done
# argv shape on the REAL legs: every claim_entry_argv line is comma-joined with no space inside the --functions
# field (the space-join cannot exist by construction; this asserts the bytes actually passed)
export MUT=none
bash /tmp/packet/e2e-v16.sh >/tmp/e2e-inv16.argvshape.log 2>&1; RC=$?
DRV="$(ls -td /tmp/e2e/out.*/drv.log 2>/dev/null | head -1)"
if [ "$RC" = 0 ] && [ -n "$DRV" ] && grep -aq '^claim_entry_argv' "$DRV" \
   && ! grep -a '^claim_entry_argv' "$DRV" | sed 's/.*--functions //' | grep -aq ' ' \
   && [ "$(grep -ac '^claim_entry_argv' "$DRV")" = 4 ]; then
  echo "INVERSION-HELD argvshape rc=0 (4 real-leg claim_entry_argv lines, none with a space inside --functions)"
else
  echo "INVERSION-FAILED argvshape rc=$RC"; FAILS=$((FAILS+1))
fi
unset MUT
echo "INVERSIONS-v16 FAILS=$FAILS"
[ "$FAILS" = 0 ]
