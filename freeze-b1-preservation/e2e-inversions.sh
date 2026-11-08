#!/bin/bash
# Inversion proof: five MUTATIONS of the stub, each must trip a NAMED law. Expectations frozen here BEFORE running.
# dup      -> probe outcome-join law: exit 2, "OUTCOME-JOIN LAW" (2 outcome lines vs roster of 1)
# foreign  -> claim join law: exit 2, "foreign name"
# missing  -> claim join law: exit 2, "roster join FAILED"
# rcmis    -> claim rc/set law: exit 2, "rc/set inconsistency"   (rc 1, zero FAIL rows)
# unobs    -> OBSERVED-BUT-UNEXPECTED: exit 1 with EXIT1-REASON row (UNOBSERVED recorded, not interpreted)
source /dev/stdin <<'ENVX'
MUTS="dup foreign missing rcmis unobs"
declare -A WANT=( [dup]=2 [foreign]=2 [missing]=2 [rcmis]=2 [unobs]=1 )
declare -A MARK=( [dup]='OUTCOME-JOIN LAW' [foreign]='foreign name' [missing]='roster join FAILED' [rcmis]='rc/set inconsistency' [unobs]='EXIT1-REASON=' )
ENVX
FAILS=0
for M in $MUTS; do
  export MUT=$M
  bash /tmp/packet/e2e-v15.sh >/tmp/e2e-inv.$M.log 2>&1; RC=$?
  want=${WANT[$M]}; mark=${MARK[$M]}
  if [ "$RC" = "$want" ] && grep -aqF "$mark" /tmp/e2e-inv.$M.log; then
    echo "INVERSION-HELD $M rc=$want mark='$mark'"
  else
    echo "INVERSION-FAILED $M rc=$RC want=$want mark='$mark' $(grep -acF "$mark" /tmp/e2e-inv.$M.log) hits"; FAILS=$((FAILS+1))
  fi
done
unset MUT
echo "INVERSIONS FAILS=$FAILS"
[ "$FAILS" = 0 ]
