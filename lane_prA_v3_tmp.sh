#!/bin/bash
# Scratch (never lands; mirrors untracked): prA v3 — close the census gap honestly.
# The digest bin FAILS OPEN (prints PARSE-FAIL, continues): during jc1a7cd3c the two
# #14559-broken grammar files were silently SKIPPED, so the census never saw their items.
# This lane digests their FIXED bytes (from fix/concept-route-data-row-parse, PR #14608)
# under OLD (12) and NEW (18) prefix binding and diffs — closing the 2-file gap.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

echo "=== fetch fixed bytes from PR #14608 head ==="
git fetch --depth=1 origin 692143606393 2>&1 | tail -1
mkdir -p /tmp/fixed_files
git show FETCH_HEAD:src/v2/test/claim/manual/java_grammar_structure.dag > /tmp/fixed_files/java_grammar_structure.dag
git show FETCH_HEAD:src/v2/test/claim/manual/rust_grammar_structure.dag > /tmp/fixed_files/rust_grammar_structure.dag
md5sum /tmp/fixed_files/*.dag

echo "=== digest bin (untracked copy) ==="
git log --oneline -1
cargo build --release -p v1-compiler --bin unary_prefix_digest 2>&1 | tail -1
echo "--- digest at THIS head (declared power; bin here = new binding) ---"
./target/release/unary_prefix_digest /tmp/fixed_files 2>&1 | grep -aE "PARSE-FAIL|ITEMS" | head -4
./target/release/unary_prefix_digest /tmp/fixed_files 2>&1 > /tmp/digest_new.txt
grep -ac "^/tmp/fixed_files" /tmp/digest_new.txt | sed 's/^/new-binding items: /'
head -10 /tmp/digest_new.txt
echo "=== DONE ==="
