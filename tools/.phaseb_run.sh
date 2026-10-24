#!/bin/bash
set -u
B() { cargo build --release -p v1-compiler --bin gunbc --bin claim_executor > /tmp/b.log 2>&1; rc=$?; grep -E "^error|Finished" /tmp/b.log | head -8; grep -B3 -A12 "^error\[" /tmp/b.log | head -60; [ $rc -eq 0 ] || { echo "BUILD FAILED rc=$rc"; exit 3; }; }
regen() {
  for round in 1 2 3 4; do
    echo "=== regen round $round $(date)"
    ./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 2>&1 | grep -v '^\[' | tail -4 | cut -c1-300
    n=0; C=target/stage0-regen-candidate/src
    [ -d $C ] || { echo "no candidate dir"; break; }
    for f in $(cd $C && find . -type f); do
      if ! cmp -s $C/$f src/v1/stage0/src/$f; then cp $C/$f src/v1/stage0/src/$f; n=$((n+1)); echo "installed $f"; fi
    done
    echo "installed $n"; [ $n -eq 0 ] && break
    B
  done
}
wit() {
  for w in nominal_host_text_argument_wall_witness nominal_cast_wall_witness; do
    for fn in $(grep -o '^test fn [a-z_0-9]*' dag/test/claim/$w.dag | awk '{print $3}'); do
      r=$(timeout 900 ./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/$w.dag --function $fn 2>&1 | grep -o 'returned `[a-z]*`' | head -1)
      echo "$1 $w $fn :: ${r:-NO-RESULT}"
    done
  done
}
B; regen
git diff --stat -- src/v1/stage0 | tail -3
wit HEAD
# mutation: the chain (ii) refusal builder returns nothing
python3 - <<'P'
p='src/v1/04_infer.dag'; s=open(p).read()
a=s.index('fn nominal_value_at_host_text_diags'); b=s.index('  if value_type_is_declared_nominal(t: t, scope: scope) {',a)
s=s[:b]+'  if false && value_type_is_declared_nominal(t: t, scope: scope) {'+s[b+len('  if value_type_is_declared_nominal(t: t, scope: scope) {'):]
open(p,'w').write(s)
P
regen
wit MUTANT
