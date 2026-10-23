#!/bin/bash
set -u
free -g | head -2; cat /sys/fs/cgroup/memory.max 2>/dev/null
date
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2
for round in 1 2; do
  echo "=== regen round $round $(date)"
  ./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 2>&1 | grep -v '^\[' | tail -15 | cut -c1-300
  git status --short | head -20
  cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2
done
echo "=== corpus compile $(date)"
./target/release/gunbc compile --output-dir /tmp/o --source-root dag --source-root src/v2 --target rust --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json > /tmp/corpus.log 2>&1
echo "exit $? $(date)"; wc -l /tmp/corpus.log
grep -c '^error\[' /tmp/corpus.log
echo "=== CENSUS by message"; grep 'CHAIN2CENSUS' /tmp/corpus.log | sed -E 's/.*(CHAIN2CENSUS [a-z]+ [A-Za-z]+).*/\1/' | sort | uniq -c
echo "=== CENSUS lines"; grep -B1 -A2 'CHAIN2CENSUS' /tmp/corpus.log | cut -c1-220 | head -4000
echo "=== other errors"; grep '^error\[' /tmp/corpus.log | grep -v CHAIN2CENSUS | wc -l
