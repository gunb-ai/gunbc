#!/usr/bin/env bash
# v17 inversion battery: 11 log-mutants (v16 suite, through e2e-v17.sh) + argvshape + 3 SOURCE mutants (M1a, M1b, M2).
set -u
cd /tmp/packet
# harness copy with a source-mutation hook inserted after the script.sh build
awk '{print} /^done < "\$G\/packet-evidence-v17.sh"/{print "if [ -n \"${SRCMUT:-}\" ]; then perl -0pi -e \"$SRCMUT\" \"$E/script.sh\" || { echo SRCMUT-APPLY-FAIL; exit 9; }; fi"}' e2e-v17.sh > e2e-v17m.sh
bash -n e2e-v17m.sh || { echo "harness hook broken"; exit 9; }
MUTS="dup foreign missing rcmis nonbare rc2allpass unobs contradiction ctl_off ctl_off2 argvbad"
declare -A WANT=( [dup]=2 [foreign]=2 [missing]=2 [rcmis]=2 [nonbare]=2 [rc2allpass]=2 [unobs]=2 [contradiction]=2 [ctl_off]=2 [ctl_off2]=2 [argvbad]=2 )
declare -A MARK=( [dup]='OUTCOME-JOIN LAW' [foreign]='foreign name' [missing]='roster join FAILED' [rcmis]='rc/set inconsistency' [nonbare]='NON-BARE outcome line' [rc2allpass]='rc/set CONTRADICTION: rc=2' [unobs]='NON-BARE outcome line' [contradiction]='roster join FAILED' [ctl_off]='CONTROL TOKEN OFF' [ctl_off2]='CONTROL TOKEN OFF' [argvbad]='no single' )
FAILS=0
for M in $MUTS; do
  export MUT=$M; unset SRCMUT
  bash /tmp/packet/e2e-v17m.sh >/tmp/e2e-inv17.$M.log 2>&1; RC=$?
  want=${WANT[$M]}; mark=${MARK[$M]}
  if [ "$RC" = "$want" ] && grep -aqF "$mark" /tmp/e2e-inv17.$M.log; then echo "INVERSION-HELD $M rc=$want"; else echo "INVERSION-FAILED $M rc=$RC want=$want mark='$mark'"; FAILS=$((FAILS+1)); fi
done
# argvshape (real legs)
export MUT=none; unset SRCMUT
bash /tmp/packet/e2e-v17m.sh >/tmp/e2e-inv17.argvshape.log 2>&1; RC=$?
DRV="$(ls -td /tmp/e2e17/out.*/drv.log 2>/dev/null | head -1)"
if [ "$RC" = 0 ] && [ -n "$DRV" ] && grep -aq '^claim_entry_argv' "$DRV" \
   && ! grep -a '^claim_entry_argv' "$DRV" | sed 's/.*--functions //' | grep -aq ' ' \
   && [ "$(grep -ac '^claim_entry_argv' "$DRV")" = 4 ]; then
  echo "INVERSION-HELD argvshape rc=0 (4 comma argv lines, no space in --functions)"
else echo "INVERSION-FAILED argvshape rc=$RC"; FAILS=$((FAILS=FAILS+1)); fi
# SOURCE mutants: each must make a fixture RED (packet exit 2 on the fixture's own message)
runsrc(){ local name="$1" wantmark="$2"; export MUT=none SRCMUT="$3"
  bash /tmp/packet/e2e-v17m.sh >/tmp/e2e-inv17.$name.log 2>&1; local rc=$?
  if [ "$rc" = 2 ] && grep -aqF "$wantmark" /tmp/e2e-inv17.$name.log; then echo "INVERSION-HELD $name rc=2 mark='$wantmark'"; else echo "INVERSION-FAILED $name rc=$rc want=2 mark='$wantmark' hits=$(grep -acF "$wantmark" /tmp/e2e-inv17.$name.log)"; FAILS=$((FAILS+1)); fi; unset SRCMUT; }
# M2: classify accepts a reasoned FAIL as FAIL -> the typed fixture (expects exit 2) must go red
runsrc classify_typed "a TYPED EXIT (ProcessExit) row satisfied" 's/\[ "\$l" = "FAIL \$2" \]/[ "\${l%% (*}" = "FAIL \$2" ]/; s/\|\| die2 "REFUSED_SETUP: verdict fixture misfire"/|| echo "FINDING: verdict unit fixtures inverted (silenced for this inversion)"/'
# M1(a): rc 0 + FAIL rows back to FINDING/exit 0 -> the rc0fail fixture must go red
runsrc rc0_finding "rc=0 with a bare FAIL row" 's/0\) \[ "\$nfail" = 0 \] \|\| die2 "claim entry \$TAG leg \$LEG: rc\/set CONTRADICTION: rc=0/0) [ "$nfail" = 0 ] || echo "FINDING: claim entry $TAG leg $LEG rc=0/'
# M1(b): accept rc 2 all-bare with FAILs -> the rc2fail fixture must go red
runsrc rc2_accept "rc=2 with bare FAIL rows" 's/\*\) die2 "claim entry \$TAG leg \$LEG: rc\/set CONTRADICTION: rc=\$RC/\*) ! { [ "\$RC" = 2 ] && [ "\$nfail" != 0 ]; } && die2 "claim entry \$TAG leg \$LEG: rc\/set CONTRADICTION: rc=\$RC/'
echo "INVERSIONS-v17 FAILS=$FAILS"
[ "$FAILS" = 0 ]
