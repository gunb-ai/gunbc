#!/bin/bash
# Lane adhoc-9102e2d6-92f scratch (never lands): C2 = CLEAN repro of stern-bat-401's main break.
# C1 (j11b130ab) was contaminated by my own untracked probe file in src/v2/test/claim/manual
# and a grafted clone (git show died, pre-merge fetch failed). This run:
#  - parses java_grammar_structure.dag + rust_grammar_structure.dag bytes through the HOST
#    parse route (parse_dag_file, what coproduct_reflection.rs uses) at 3db8 and at 27ea1fa (= 7956cb45df8^, pre-my-merge)
#  - runs the real concept route (whole_tree_extract_smoke) at both commits
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

# Save the digest bin source (my untracked file survives checkouts, but keep a copy anyway)
cp src/v1/stage0/src/bin/unary_prefix_digest.rs /tmp/unary_prefix_digest.rs

run_route() { # $1 = label
  ./target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src" --source-root "$ROOT/src/v2" \
    --entry src/v2/lens/grounding.dag --function whole_tree_extract_smoke > /tmp/route_$1.log 2>&1
  local rc=$?
  echo "route@$1 rc=$rc"
  grep -a "refused\|failed to parse\|expected item\|error" /tmp/route_$1.log | head -8
  grep -ac "java_grammar_structure" /tmp/route_$1.log | sed 's/^/route-mentions-java: /'
  return 0
}

run_concept_live() { # $1 = label -- the EXACT reported route: decls_parse_only_fail_closed
  ./target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src" --source-root "$ROOT/src/v2" \
    --entry src/v2/std/concept_index.dag --function concept_decl_facts_live > /tmp/concept_$1.log 2>&1
  local rc=$?
  echo "concept_live@$1 rc=$rc"
  grep -a "TypeError\|failed to parse\|fail-closed\|silent skip\|refused" /tmp/concept_$1.log | head -8
  wc -c < /tmp/concept_$1.log | sed 's/^/concept-log-bytes: /'
  return 0
}

parse_two() { # $1 = label; isolated host-parse of the two grammar files + probe-free dir
  mkdir -p /tmp/twofiles_$1
  cp src/v2/test/claim/manual/java_grammar_structure.dag /tmp/twofiles_$1/
  cp src/v2/test/claim/manual/rust_grammar_structure.dag /tmp/twofiles_$1/
  md5sum /tmp/twofiles_$1/*.dag
  ./target/release/unary_prefix_digest /tmp/twofiles_$1 2>&1 | grep -a "PARSE-FAIL\|files\|items" | head -8
}

echo "=== A. AT origin/main 3db8f9e731ff ==="
git fetch --depth=1 origin 3db8f9e731ff 2>&1 | tail -1
git checkout -q FETCH_HEAD && git rev-parse --short HEAD
cp /tmp/unary_prefix_digest.rs src/v1/stage0/src/bin/unary_prefix_digest.rs
cargo build --release -p v1-compiler --bin gunbc --bin unary_prefix_digest 2>&1 | tail -1
git status --porcelain | sed 's/^/dirty: /'
parse_two main
run_route main
rm -f src/v1/stage0/src/bin/unary_prefix_digest.rs

echo "=== B. AT 27ea1faa439 = 7956cb45df8^ (BEFORE my merge; decisive control) ==="
git fetch --depth=1 origin 27ea1faa439a5988dfd30ea1a997ac2ad8125648 2>&1 | tail -1
git checkout -q FETCH_HEAD && git rev-parse --short HEAD
cp /tmp/unary_prefix_digest.rs src/v1/stage0/src/bin/unary_prefix_digest.rs
cargo build --release -p v1-compiler --bin gunbc --bin unary_prefix_digest 2>&1 | tail -1
git status --porcelain | sed 's/^/dirty: /'
parse_two premerge
run_route premerge
rm -f src/v1/stage0/src/bin/unary_prefix_digest.rs

echo "=== C. file equality across the two commits ==="
md5sum /tmp/twofiles_main/*.dag /tmp/twofiles_premerge/*.dag

git checkout -q session/nimble-otter-791
echo "=== DONE ==="
