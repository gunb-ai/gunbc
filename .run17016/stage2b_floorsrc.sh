#!/usr/bin/env bash
# STAGE 2b: same whole-module run against the FLOOR module (src tree, floor's own
# filesystem_effect_reach_wall.dag — its rows are the ones the census must classify).
. ./stage_common.sh
HEAD="$1"; T="head-$HEAD"; B="$T/target/release/gunbc"
test -x "$B" || { echo "NO_BINARY"; exit 9; }
echo "BINARY_SHA256=$(sha256sum "$B" | awk '{print $1}')"; echo "BUILT_FROM=$(git -C "$T" rev-parse HEAD) head-built, manifest-unmatched"
echo "GATE1MIN=$(awk '{print $1}' /proc/loadavg)"
"$B" test --entry "$T/src/v2/test/claim/filesystem_effect_reach_wall.dag" --source-root "$T/dag" --source-root "$T/src/v2" > floor_module.log 2>&1; RC=$?
echo "FLOOR_MODULE_RC=$RC"
grep -cE "^ok " floor_module.log | sed 's/^/FLOOR_OK_COUNT=/'
grep -E "census" floor_module.log | head -12
[ "$RC" -eq 0 ] || { echo "FLOOR_MODULE_FAILED"; tail -60 floor_module.log; }
exit $RC
