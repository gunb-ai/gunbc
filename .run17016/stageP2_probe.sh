#!/usr/bin/env bash
# STAGE P2: the arrival-catch route, SEEN biting end to end. On a copy, blind ONE
# roster identity: delete the `uses fs: Filesystem;` clause from declaration
# mint_a_receipt_from_authored_standings in the forged-probe witness (a census-deletable
# fn row, NOT a census row of the wall witness). The corpus walk (the arriving producer)
# then cannot find that identity, so the blindness arm
# (every_authorized_restatement_identity_is_found_in_the_corpus) must go RED by naming
# it. Meanwhile the hold arm still holds for every OTHER identity (the guard bites one,
# not everything): wall_holds goes red only through the blinded row.
. ./stage_common.sh
HEAD="$1"; T="head-$HEAD"; B="$T/target/release/gunbc"
test -x "$B" || { echo "NO_BINARY"; exit 9; }
echo "BINARY_SHA256=$(sha256sum "$B" | awk '{print $1}')"; echo "BUILT_FROM=$(git -C "$T" rev-parse HEAD) head-built, manifest-unmatched"
rm -rf probe_p2 && mkdir -p probe_p2
cp -r "$T/dag" probe_p2/dag && cp -r "$T/src" probe_p2/src
F=probe_p2/dag/test/probe/census_app_acquisition_forged_probe.dag
test -f "$F" || { echo "PROBE_WITNESS_MISSING $F"; exit 8; }
PRE=$(grep -c 'uses fs: Filesystem' "$F"); echo "P2_PRE_HITS=$PRE"
grep -n "mint_a_receipt_from_authored_standings" "$F" | head -2
python3 - "$F" <<'PY'
import sys,re
p=sys.argv[1]; s=open(p).read()
m=re.search(r"(fn mint_a_receipt_from_authored_standings[^{]*\{)([^}]*)\}", s, re.S)
if not m: print("P2_ANCHOR_MISSING"); sys.exit(7)
body=m.group(2)
if "uses fs: Filesystem" not in body: print("P2_CLAUSE_NOT_IN_BODY"); sys.exit(7)
nb=body.replace("uses fs: Filesystem;","",1)
s=s[:m.start()]+m.group(1)+nb+"}"+s[m.end():]
open(p,"w").write(s); print("P2_BLINDED")
PY
POST=$(grep -c 'uses fs: Filesystem' "$F"); echo "P2_POST_HITS=$POST (expect PRE-1)"
echo "GATE1MIN=$(awk '{print $1}' /proc/loadavg)"
"$B" test --entry probe_p2/dag/test/claim/filesystem_effect_reach_wall.dag --source-root probe_p2/dag --source-root probe_p2/src/v2 > probe_p2_run.log 2>&1; RC=$?
echo "P2_RC=$RC (expect NONZERO: the blindness arm must go red)"
grep -E "every_authorized_restatement_identity_is_found_in_the_corpus" probe_p2_run.log | head -3
grep -E "demand_restatement_wall_holds_over_the_corpus_derived_arriving" probe_p2_run.log | head -3
if [ "$RC" -ne 0 ]; then echo "P2_VERDICT REFUSED as required";
else echo "P2_VERDICT VOID_STOP blinded corpus accepted"; tail -40 probe_p2_run.log; fi
[ "$RC" -ne 0 ]
