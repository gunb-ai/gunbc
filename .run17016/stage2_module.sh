#!/usr/bin/env bash
# STAGE 2: whole-module claim run on the wall witness, head-built binary.
. ./stage_common.sh
HEAD="$1"; T="head-$HEAD"; B="$T/target/release/gunbc"
test -x "$B" || { echo "NO_BINARY"; exit 9; }
echo "BINARY_SHA256=$(sha256sum "$B" | awk '{print $1}')"; echo "BUILT_FROM=$(git -C "$T" rev-parse HEAD) head-built, manifest-unmatched"
echo "GATE1MIN=$(awk '{print $1}' /proc/loadavg)"
"$B" test --entry "$T/src/v2/test/claim/filesystem_effect_reach_wall.dag" --source-root "$T/dag" --source-root "$T/src/v2" > module_run.log 2>&1; RC=$?
echo "MODULE_RC=$RC"
grep -E "^(ok|FAIL|PASS|error|failed|Test [0-9])" module_run.log | tail -40
grep -cE "^ok " module_run.log | sed 's/^/OK_COUNT=/'
grep -E "census" module_run.log | head -12
[ "$RC" -eq 0 ] || { echo "MODULE_FAILED"; tail -60 module_run.log; }
exit $RC
