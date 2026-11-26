#!/usr/bin/env bash
# STAGE P1: census SEEN discharging on files. Delete the contracted door_ledger_row from
# a COPY of the wall witness; rerun the module there; discharge arm must pass (rc 0).
. ./stage_common.sh
HEAD="$1"; T="head-$HEAD"; B="$PWD/$T/target/release/gunbc"
test -x "$B" || { echo NO_BINARY; exit 9; }
echo "BINARY_PROVENANCE=head-built manifest-unmatched sha256=$(sha256sum "$B" | awk "{print \$1}") built_from=$(git -C "$T" rev-parse HEAD)"
rm -rf probe_p1 && mkdir -p probe_p1
cp -r "$T/dag" probe_p1/dag && cp -r "$T/src" probe_p1/src
F=probe_p1/dag/test/claim/filesystem_effect_reach_wall.dag
test -f "$F" || { echo "WITNESS_MISSING $F"; exit 8; }
echo "PRE_HIT=$(grep -c door_ledger_row "$F")"
python3 - "$F" <<'PY'
import sys,re
p=sys.argv[1]; s=open(p).read()
m=re.search(r"data filesystem_contract_rows: List<DeclPhase1Row> = \[(.*?)\]", s, re.S)
if not m: print("P1_CONTRACT_ROWS_MISSING"); sys.exit(7)
block=m.group(1)
if "door_ledger_row" not in block: print("P1_MEMBER_ABSENT"); sys.exit(7)
nb=re.sub(r"\s*derived_row\(\"v2\.workflow\.compile_door_ledger\", \"door_ledger_row\", \[read_seam_path\(\)\]\),?", "", block)
s=s[:m.start(1)]+nb+s[m.end(1):]
open(p,"w").write(s); print("P1_ROW_DELETED")
PY
echo "POST_HIT=$(grep -c door_ledger_row "$F")"
echo "GATE1MIN=$(awk "{print \$1}" /proc/loadavg)"
BASE="$PWD"
cd "$T" || exit 9  # workspace_root rule: head tree is the checkout; corpus overridden by absolute probe roots
"$B" test "//dag/test/claim/filesystem_effect_reach_wall" --source-root "$BASE/probe_p1/dag" --source-root "$BASE/probe_p1/src/v1" --source-root "$BASE/probe_p1/src/v2" > ../probe_p1_run.log 2>&1; RC=$?
cd .. || exit 9
echo "P1_RC=$RC (expect 0: discharge passes on the deleted row; nonzero is a finding, printed)"
grep -qi panic probe_p1_run.log && { echo P1_PANIC; tail -n 20 probe_p1_run.log; exit 0; }
grep -E "census_is_seen_discharging|census_over_real_resolver" probe_p1_run.log | head -4
grep -iE "census" probe_p1_run.log | head -14
grep -E "^ok |FAILED|FAIL " probe_p1_run.log | tail -8
[ "$RC" -eq 0 ] || echo P1_MODULE_FAILED_PRINTED_ABOVE
exit 0
