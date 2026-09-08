#!/bin/bash
set -u
cargo build --release --bin claim_executor
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/regen.out 2>&1
echo REGEN_EXIT:$?
rg -n "v1_compiler_infer|FAIL|drift" /tmp/regen.out | head -40
CAND=target/stage0-regen-candidate/src/v1_compiler_infer.rs
if [ ! -f "$CAND" ]; then
  CAND=$(find target/stage0-regen-candidate -name 'v1_compiler_infer.rs' | head -1)
fi
echo CAND:"$CAND"
if [ -f "$CAND" ]; then
  diff -u src/v1/stage0/src/v1_compiler_infer.rs "$CAND" | head -c 200000
  echo
  echo DIFF_BYTES:$(diff -u src/v1/stage0/src/v1_compiler_infer.rs "$CAND" | wc -c)
fi
tail -20 /tmp/regen.out
