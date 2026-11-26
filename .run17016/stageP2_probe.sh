#!/usr/bin/env bash
# STAGE P2: arrival-catch seen biting. Blind one roster identity (uses fs: Filesystem in
# mint_a_receipt_from_authored_standings, forged-probe witness); the corpus walk cannot
# find it; the blindness arm must go RED (module rc nonzero) naming it.
. ./stage_common.sh
HEAD="$1"; T="head-$HEAD"; B="$PWD/$T/target/release/gunbc"
test -x "$B" || { echo NO_BINARY; exit 9; }
echo "BINARY_PROVENANCE=head-built manifest-unmatched sha256=$(sha256sum "$B" | awk "{print \$1}") built_from=$(git -C "$T" rev-parse HEAD)"
rm -rf probe_p2 && mkdir -p probe_p2
cp -r "$T/dag" probe_p2/dag && cp -r "$T/src" probe_p2/src
F=probe_p2/dag/test/probe/census_app_acquisition_forged_probe.dag
test -f "$F" || { echo "PROBE_WITNESS_MISSING $F"; exit 8; }
echo "P2_PRE_HITS=$(grep -c "uses fs: Filesystem" "$F")"
grep -n mint_a_receipt_from_authored_standings "$F" | head -2
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
echo "P2_POST_HITS=$(grep -c "uses fs: Filesystem" "$F")"
echo "GATE1MIN=$(awk "{print \$1}" /proc/loadavg)"
BASE="$PWD"
cd "$T" || exit 9  # workspace_root rule: head tree is the checkout; corpus overridden by absolute probe roots
"$B" test "//dag/test/claim/filesystem_effect_reach_wall" --source-root "$BASE/probe_p2/dag" --source-root "$BASE/probe_p2/src/v1" --source-root "$BASE/probe_p2/src/v2" > ../probe_p2_run.log 2>&1; RC=$?
cd .. || exit 9
echo "P2_RC=$RC (expect NONZERO: blindness arm must go red)"
grep -qi panic probe_p2_run.log && { echo P2_PANIC_NOT_REFUSAL; tail -n 20 probe_p2_run.log; exit 1; }
grep -E "every_authorized_restatement_identity_is_found_in_the_corpus" probe_p2_run.log | head -3
grep -E "demand_restatement_wall_holds_over_the_corpus_derived_arriving" probe_p2_run.log | head -3
if [ "$RC" -ne 0 ]; then echo "P2_VERDICT REFUSED as required"; else echo "P2_VERDICT VOID_STOP blinded corpus accepted"; tail -40 probe_p2_run.log; fi
[ "$RC" -ne 0 ]
