#!/bin/bash
# Lane adhoc-9102e2d6-92f scratch (never lands): D = BISECT the #14559 construct that breaks
# parse_dag_file on java/rust_grammar_structure.dag (stern-bat-401's main break).
# C2 receipts (j017076d2): route@premerge rc=2 (parsed, returned true) vs route@head rc=1
# TypeError "concept_decl_facts: failed to parse `src/v2/test/claim/manual/java_grammar_structure.dag`";
# parse-standalone: pre-merge files OK, post-#14559 files PARSE-FAIL.
# Prime suspect: the two-line `data` row (initializer on a continuation line). dag.dag's
# identical row (src/v2/extdeps/languages/dag.dag:4235) is ONE line and parses fine.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
JAVA=src/v2/test/claim/manual/java_grammar_structure.dag
RUST=src/v2/test/claim/manual/rust_grammar_structure.dag

run_route() { # $1 = label
  ./target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src" --source-root "$ROOT/src/v2" \
    --entry src/v2/lens/grounding.dag --function whole_tree_extract_smoke > /tmp/route_$1.log 2>&1
  local rc=$?
  echo "route@$1 rc=$rc"
  grep -a "TypeError\|failed to parse\|returned \`true\`" /tmp/route_$1.log | head -3
  return 0
}

probe() { # $1..$n = dirs to parse-standalone
  ./target/release/unary_prefix_digest "$@" 2>&1 | grep -a "PARSE-FAIL\|ITEMS" | head -6
}

echo "=== 0. SETUP (head + probe bins) ==="
git rev-parse --short HEAD
cp src/v1/stage0/src/bin/unary_prefix_digest.rs /tmp/digest.rs 2>/dev/null || true
cargo build --release -p v1-compiler --bin gunbc --bin unary_prefix_digest 2>&1 | tail -1
cp /tmp/digest.rs src/v1/stage0/src/bin/unary_prefix_digest.rs 2>/dev/null || true
cargo build --release -p v1-compiler --bin unary_prefix_digest 2>&1 | tail -1

cp $JAVA /tmp/java_orig.dag
cp $RUST /tmp/rust_orig.dag

echo "=== 1. REPRO AT CURRENT origin/main (same grammar bytes as reported 3db8: git log shows the file untouched since 7956cb45df8) ==="
git fetch --depth=1 origin 3db8f9e731ffc2079a316c958b760abd78d086ae 2>&1 | tail -1
git checkout -q FETCH_HEAD 2>/dev/null && echo "checked out $(git rev-parse --short HEAD)"
git log -1 --format='commit: %H %s' | head -1
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
run_route exactmain
git checkout -q session/nimble-otter-791
cp /tmp/java_orig.dag $JAVA
cp /tmp/rust_orig.dag $RUST
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1

echo "=== 2. CONTROL at head (broken): expect route rc=1 ==="
mkdir -p /tmp/v0 && cp $JAVA $RUST /tmp/v0/
probe /tmp/v0
run_route head_control

echo "=== 3. V-E PRIME SUSPECT: single-line the data row (initializer same line) ==="
python3 - <<'PY'
for p in ["src/v2/test/claim/manual/java_grammar_structure.dag",
          "src/v2/test/claim/manual/rust_grammar_structure.dag"]:
    s = open(p).read()
    import re
    s = re.sub(r"(data \w+: Map<Symbol, Bool> =)\n  (lex_rule_set_token_class_map\(rules: \w+\(\)\))", r"\1 \2", s)
    open(p, "w").write(s)
PY
grep -n "token_class_membership: Map" $JAVA $RUST | sed 's/^/line: /'
mkdir -p /tmp/vE && cp $JAVA $RUST /tmp/vE/
probe /tmp/vE
run_route vE_oneline
cp /tmp/java_orig.dag $JAVA; cp /tmp/rust_orig.dag $RUST

echo "=== 4. V-A: drop my 3 comment lines only ==="
grep -v "^// Built ONCE\|^// rule set is a pure\|^// duplication (gunbc" /tmp/java_orig.dag > $JAVA
mkdir -p /tmp/vA && cp $JAVA /tmp/vA/
probe /tmp/vA
run_route vA_comments_out
cp /tmp/java_orig.dag $JAVA

echo "=== 5. V-B: data row deleted, member call restored (java only) ==="
python3 - <<'PY'
p = "src/v2/test/claim/manual/java_grammar_structure.dag"
s = open(p).read()
s = s.replace("""data java_grammar_structure_token_class_membership: Map<Symbol, Bool> =
  lex_rule_set_token_class_map(rules: java_lex_rules())
""", "")
s = s.replace("map_contains_key(m: java_grammar_structure_token_class_membership, key: ^java_token_kw_void)",
              "lex_rule_set_token_class_member(rules: java_lex_rules(), token_class: ^java_token_kw_void)")
open(p, "w").write(s)
PY
mkdir -p /tmp/vB && cp $JAVA /tmp/vB/
probe /tmp/vB
run_route vB_java_reverted
cp /tmp/java_orig.dag $JAVA

echo "=== restore + final control ==="
cp /tmp/java_orig.dag $JAVA
cp /tmp/rust_orig.dag $RUST
mkdir -p /tmp/vF && cp $JAVA $RUST /tmp/vF/
probe /tmp/vF
run_route final_broken_control
git status --porcelain | sed 's/^/dirty: /'
echo "=== DONE ==="
