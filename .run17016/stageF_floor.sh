#!/usr/bin/env bash
# STAGE F: the required-floor claim_batch, three roots, with the MAIN-BUILT floor
# binary (cutover main 4036f188602, 'main-built, manifest-unmatched').
. ./stage_common.sh
MB=4036f188602; B="$HOME/scratch/bold-crane-719/bin/gunbc-main-$MB"; T="main-$MB"
test -x "$B" || { echo "NO_FLOOR_BINARY"; exit 9; }
echo "BINARY_SHA256=$(sha256sum "$B" | awk '{print $1}')"; echo "BUILT_FROM main 4036f188602 main-built, manifest-unmatched"
test -f "$T/src/v2/test/claim/filesystem_effect_reach_wall.dag" || { echo "FLOOR_MODULE_MISSING"; exit 8; }
echo "GATE1MIN=$(awk '{print $1}' /proc/loadavg)"
"$B" claim_batch --source-root "$T/dag" --source-root "$T/src/v1" --source-root "$T/src/v2" --entry src/v2/test/claim/filesystem_effect_reach_wall --functions all > floor_batch.log 2>&1; RC=$?
echo "FLOOR_BATCH_RC=$RC"
grep -E "^(ok|FAIL|PASS|error|failed)" floor_batch.log | tail -30
grep -cE "^ok " floor_batch.log | sed 's/^/FLOOR_BATCH_OK=/'
[ "$RC" -eq 0 ] || { echo "FLOOR_BATCH_FAILED"; tail -60 floor_batch.log; }
exit $RC
