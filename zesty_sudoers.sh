#!/bin/bash
# scratch (NOT for commit): regenerate the two host sudoers artifacts after the rebase.
cd "$(dirname "$0")" || exit 1
G=/session-home/target-zesty/release/gunbc
for host in srv3 srv4; do
  echo "=== $host ==="
  "$G" run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/instruments/generated_artifact_gate.dag \
    --function main_wet_one --arg "path=provisioning/$host/gunbc-ghrunner.sudoers" \
    > "/session-home/logs/sudoers-$host.log" 2>&1
  echo "RUN_RC=$? $(tail -1 /session-home/logs/sudoers-$host.log | cut -c1-160)"
done
echo "--- git diff --stat ---"
git diff --stat -- provisioning/
git diff -- provisioning/ | grep -E "^[+-]" | grep -v "^[+-][+-]" | head -40
echo "SUDOERS DONE"
