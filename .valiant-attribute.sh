#!/usr/bin/env bash
# ATTRIBUTION (tidy-ant-251 condition 2): the remaining std-core errors, BY SITE and BY THE RENDER
# ARM that produced each. Bootstrap main's mirror set, regen once from the live corpus, adopt the
# candidate IN FULL (this is the merge driver's step-2 'install every generated file it names'),
# then build v1-stage0-std-core from the candidate bodies and print each error with its source
# context, plus the whole emitted std_algebra.rs so the render arm can be identified locally.
set -uo pipefail
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536
cd "$(dirname "$0")"

echo "=== BOOTSTRAP main mirrors; build regenerator ==="
git checkout ae563700cbe -- src/v1/stage0/src/
cargo build --release -p v1-compiler --bin claim_executor > /tmp/ab1.log 2>&1
echo "build_errors=$(grep -cE '^error' /tmp/ab1.log)"

echo "=== REGEN (live corpus, my 5 source edits in the tree) ==="
rm -rf target/candAtt
./target/release/claim_executor --required-regen --regen-candidate-dir target/candAtt \
  --regen-receipt target/receiptAtt.json --source-root dag --source-root src/v2 > /tmp/att.out 2>&1
echo "regen_exit=$?"; grep -cE "effect summary incomplete" /tmp/att.out | sed 's/^/effect_diags=/'
grep -E "^required-regen" /tmp/att.out | tail -2

echo "=== ADOPT THE CANDIDATE IN FULL (step 2 says install every file it names) ==="
N=$(ls target/candAtt/src/*.rs | wc -l); echo "candidate_files=$N"
cp target/candAtt/src/*.rs src/v1/stage0/src/

echo "=== BUILD v1-stage0-std-core FROM THE CANDIDATE BODIES (the target of the 26) ==="
cargo build --release -p v1-stage0-std-core > /tmp/att-sc.log 2>&1
echo "STDCORE_ERRORS=$(grep -cE '^error' /tmp/att-sc.log)"
echo "--- error sites (file:line:col, message) ---"
grep -E "^error(\[|:)" /tmp/att-sc.log | sed 's/^[[0-9;]*//g' | sort | uniq -c
echo "--- per-error: the --> line plus the 'expected/found' lines ---"
awk '/^error\[E0/{p=1;buf=""} p{buf=buf $0 "\n"} /-->|expected|found|^error\[E0/{if(p&&/-->|expected|found/)print buf} /^$/{if(p){p=0}}' /tmp/att-sc.log | sed 's/^[[0-9;]*//g' | grep -E "-->|expected|found" | sed 's/^[[0-9;]*//g' > /tmp/att-sites.txt
cat /tmp/att-sites.txt | sed 's/^[[0-9;]*//g' | head -60

echo "=== CANDIDATE std_algebra.rs, every error line +/- 6, plus its fn header ==="
for L in $(grep -oE 'std_algebra\.rs:[0-9]+' /tmp/att-sites.txt | cut -d: -f2 | sort -un); do
  echo "##### site line $L"
  awk -v L=$L 'NR>=L-6 && NR<=L+3 {printf "%4d| %s\n", NR, $0}' target/candAtt/src/std_algebra.rs
  awk -v L=$L 'NR<=L && /^pub fn |^fn /{h=NR": "$0} NR==L{print "      in: " h}' target/candAtt/src/std_algebra.rs
done

echo "=== also std_types.rs / std_computation.rs error lines if any ==="
for f in std_types std_computation; do
  for L in $(grep -oE "$f\.rs:[0-9]+" /tmp/att-sites.txt | cut -d: -f2 | sort -un); do
    echo "##### $f:$L"
    awk -v L=$L -v F=target/candAtt/src/$f.rs 'NR>=L-5 && NR<=L+2 {printf "%4d| %s\n", NR, $0}' $F
  done
done

echo "=== VALIANT-MIRROR-BEGIN ==="
for f in std_algebra.rs; do
  echo "VALIANT-FILE $f $(wc -c < target/candAtt/src/$f)"
  base64 -w 120 target/candAtt/src/$f
done
echo "=== VALIANT-MIRROR-END ==="
