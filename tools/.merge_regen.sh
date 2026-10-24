#!/bin/bash
set -u
B() { cargo build --release -p v1-compiler --bin gunbc --bin claim_executor > /tmp/b.log 2>&1; rc=$?; grep -E "^error|Finished" /tmp/b.log | head -5; grep -A12 "^error\[" /tmp/b.log | head -40; [ $rc -eq 0 ] || { echo "BUILD FAILED"; exit 3; }; }
B
echo "=== main_wet"
timeout 2400 ./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet 2>&1 | grep -v "^\[" | tail -15 | cut -c1-300
for round in 1 2 3 4 5; do
  echo "=== regen round $round"
  ./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 2>&1 | grep -v '^\[' | tail -4 | cut -c1-300
  n=0; C=target/stage0-regen-candidate/src
  [ -d $C ] || { echo "no candidate dir"; break; }
  for f in $(cd $C && find . -type f); do if ! cmp -s $C/$f src/v1/stage0/src/$f; then cp $C/$f src/v1/stage0/src/$f; n=$((n+1)); echo "installed $f"; fi; done
  echo "installed $n"; [ $n -eq 0 ] && break
  B
done
git diff --stat -- src/v1/stage0
echo "MIRRORPATCH-BEGIN"; git diff --binary -- src/v1/stage0 | gzip -9 | base64 -w0; echo; echo "MIRRORPATCH-END"
