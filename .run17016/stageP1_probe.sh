#!/usr/bin/env bash
# STAGE P1: the census is SEEN discharging on files. On a copy of the head tree, delete
# the contracted door_ledger_row declaration (module v2.workflow.compile_door_ledger)
# from the wall witness. The file-fed arms (filesystem_effect_census_over_real_resolver_rows_
# matches_the_contract + census_is_seen_discharging_when_a_contracted_row_is_removed) then
# run against a corpus with ONE contract member unproduced. The discharge leg must PASS
# there (the module runs green), and the census lines are printed; the real-rows leg would
# fail IF the join silently shrunk -- a module rc=0 on the copy is the joint evidence.
. ./stage_common.sh
HEAD="$1"; T="head-$HEAD"; B="$T/target/release/gunbc"
test -x "$B" || { echo "NO_BINARY"; exit 9; }
echo "BINARY_SHA256=$(sha256sum "$B" | awk '{print $1}')"; echo "BUILT_FROM=$(git -C "$T" rev-parse HEAD) head-built, manifest-unmatched"
rm -rf probe_p1 && mkdir -p probe_p1
cp -r "$T/dag" probe_p1/dag && cp -r "$T/src" probe_p1/src
F=probe_p1/dag/test/claim/filesystem_effect_reach_wall.dag
test -f "$F" || { echo "WITNESS_MISSING $F"; exit 8; }
echo "PRE_HIT=$(grep -c 'door_ledger_row' "$F")"
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
echo "POST_HIT=$(grep -c 'door_ledger_row' "$F")"
echo "GATE1MIN=$(awk '{print $1}' /proc/loadavg)"
"$B" test --entry probe_p1/dag/test/claim/filesystem_effect_reach_wall.dag --source-root probe_p1/dag --source-root probe_p1/src/v2 > probe_p1_run.log 2>&1; RC=$?
echo "P1_RC=$RC (expect 0: the discharge leg passes on the deleted row; a nonzero here is a finding, printed below)"
grep -E "census_is_seen_discharging|census_over_real_resolver" probe_p1_run.log | head -4
grep -E "census" probe_p1_run.log | head -12
grep -E "^ok |FAILED|FAIL " probe_p1_run.log | tail -8
[ "$RC" -eq 0 ] || { echo "P1_MODULE_FAILED (printed above)"; }
exit 0
