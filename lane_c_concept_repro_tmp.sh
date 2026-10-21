#!/bin/bash
# Lane adhoc-9102e2d6-92f scratch (never lands): REPRO for stern-bat-401's main break.
# concept_decl_facts refuses on src/v2/test/claim/manual/java_grammar_structure.dag at
# origin/main 3db8f9e731ff. Narrows: which bytes, which route, whose change.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

echo "=== 0. HEAD ==="
git log --oneline -1
git rev-parse origin/main 2>/dev/null | head -c 12; echo " <- origin/main as fetched"

echo "=== 1. BUILD ==="
cargo build --release -p v1-compiler --bin gunbc --bin unary_prefix_digest 2>&1 | tail -2

echo "=== 2. ISOLATED PARSE, three byte versions, same mirror parse_dag_file ==="
mkdir -p /tmp/java_versions
git show 7956cb45df8^:src/v2/test/claim/manual/java_grammar_structure.dag > /tmp/java_versions/java_before.dag
git show 7956cb45df8:src/v2/test/claim/manual/java_grammar_structure.dag > /tmp/java_versions/java_mine.dag
git show origin/main:src/v2/test/claim/manual/java_grammar_structure.dag > /tmp/java_versions/java_main.dag
md5sum /tmp/java_versions/*.dag
./target/release/unary_prefix_digest /tmp/java_versions 2>&1 | tail -20

echo "=== 3. REAL ROUTE at HEAD: whole_tree_extract_smoke (grounding.dag:377) ==="
./target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src" --source-root "$ROOT/src/v2" \
  --entry src/v2/lens/grounding.dag --function whole_tree_extract_smoke 2>&1 | tail -6
echo "rc=$?"

echo "=== 4. REAL ROUTE at HEAD: concept_decl_facts over witness_layer_roots ==="
./target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src" --source-root "$ROOT/src/v2" \
  --entry src/v2/std/concept_index.dag --function concept_decl_facts_live 2>&1 | tail -6
echo "rc=$?"

run_route() { # $1 = label
  ./target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src" --source-root "$ROOT/src/v2" \
    --entry src/v2/lens/grounding.dag --function whole_tree_extract_smoke > /tmp/route_$1.log 2>&1
  echo "route@$1 rc=$?"; tail -5 /tmp/route_$1.log
}

cp src/v1/stage0/src/bin/unary_prefix_digest.rs /tmp/unary_prefix_digest.rs

echo "=== 5. ROUTE AT origin/main 3db8f9e731ff (the reported break) ==="
git fetch --depth=1 origin 3db8f9e731ff 2>&1 | tail -1
git checkout -q FETCH_HEAD && git rev-parse --short HEAD | sed 's/^/checked out: /'
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
run_route main
cp /tmp/unary_prefix_digest.rs src/v1/stage0/src/bin/unary_prefix_digest.rs
cargo build --release -p v1-compiler --bin unary_prefix_digest 2>&1 | tail -1
./target/release/unary_prefix_digest src/v2/test/claim/manual 2>&1 | grep "^PARSE-FAIL" | head -10

echo "=== 6. ROUTE AT 7956cb45df8^ (BEFORE my merge -- the decisive control) ==="
git fetch --depth=1 origin 7956cb45df8^ 2>&1 | tail -1
git checkout -q FETCH_HEAD && git rev-parse --short HEAD | sed 's/^/checked out: /'
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
run_route premerge

git checkout -q - 2>&1 | tail -1
echo "=== DONE ==="
