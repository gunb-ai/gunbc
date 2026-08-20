set -e
echo "AMBIENT RUSTC_WRAPPER before clearing: '${RUSTC_WRAPPER:-<unset>}'"
unset RUSTC_WRAPPER || true
export RUSTC_WRAPPER=

MERGE_FULL=10542cb3bb498ad408ec39e244aede102a36fd27
MAINTIP_FULL=29d2a66695afc4c9de1e3a2e7b7375d929c51d82
PRE8579_FULL=9fb884aa0062754a482b1b0e85b491e989826330

echo "MERGE_FULL=$MERGE_FULL"
echo "MAINTIP_FULL=$MAINTIP_FULL"
echo "PRE8579_FULL=$PRE8579_FULL"

run_mw() {
  BIN=$1
  OUT=$2
  set +e
  $BIN run --source-root dag --source-root src/v2 --entry dag/tools/generated_artifact_gate.dag --function main_wet > "$OUT" 2>&1
  echo "mw_exit($OUT)=$?"
  set -e
  grep -c "error:" "$OUT" || true
  grep "error:" "$OUT" | sort > "$OUT.sorted"
}

echo "=== STEP 1: build FINAL (post-merge, post-regen) artifact ==="
git fetch origin $MERGE_FULL
git checkout --force -B infer-arg-contract-fix $MERGE_FULL
GOT=$(git rev-parse HEAD)
if [ "$GOT" != "$MERGE_FULL" ]; then echo "MERGEFAIL got $GOT want $MERGE_FULL"; exit 99; fi
echo "MERGE_OK $GOT"

rm -rf /tmp/target-preregen
cargo build --release --target-dir /tmp/target-preregen -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -10

set +e
/tmp/target-preregen/release/claim_executor --required-regen --source-root dag --source-root src/v2 --regen-candidate-dir /tmp/merge-regen-candidate --regen-receipt /tmp/merge-regen-receipt.json > /tmp/merge_rr.log 2>&1
RR_EXIT=$?
echo "merge_rr_exit=$RR_EXIT"
set -e
tail -30 /tmp/merge_rr.log

ALGEBRA_CAND=$(find /tmp/merge-regen-candidate -iname "std_algebra.rs" | head -1)
INFER_CAND=$(find /tmp/merge-regen-candidate -iname "v1_compiler_infer.rs" | head -1)
echo "algebra candidate: $ALGEBRA_CAND"
echo "infer candidate: $INFER_CAND"

if [ -z "$ALGEBRA_CAND" ] || [ -z "$INFER_CAND" ]; then
  echo "MISSING CANDIDATE(S) -- ABORTING, must not hand-mirror silently"
  exit 97
fi
diff "$ALGEBRA_CAND" src/v1/stage0/src/std_algebra.rs || true
cp "$ALGEBRA_CAND" src/v1/stage0/src/std_algebra.rs
diff "$INFER_CAND" src/v1/stage0/src/v1_compiler_infer.rs || true
cp "$INFER_CAND" src/v1/stage0/src/v1_compiler_infer.rs
cargo fmt --all 2>&1 | tail -5 || true

git diff -- src/v1/stage0/src/std_algebra.rs src/v1/stage0/src/v1_compiler_infer.rs > /tmp/regen_mirrors.diff
wc -l /tmp/regen_mirrors.diff
echo "--- BEGIN REGEN_MIRRORS_DIFF ---"
cat /tmp/regen_mirrors.diff
echo "--- END REGEN_MIRRORS_DIFF ---"

rm -rf /tmp/target-final
cargo build --release --target-dir /tmp/target-final -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -15

echo "=== FINAL artifact main_wet (the AFTER measurement, same for both baseline comparisons) ==="
run_mw /tmp/target-final/release/gunbc /tmp/after_mw.log

echo "=== STEP 2: build MAIN-TIP baseline ($MAINTIP_FULL) -- contaminated by #8579's hardcode ==="
git fetch origin $MAINTIP_FULL
git checkout --force $MAINTIP_FULL
GOT=$(git rev-parse HEAD)
if [ "$GOT" != "$MAINTIP_FULL" ]; then echo "MAINTIPFAIL got $GOT want $MAINTIP_FULL"; exit 95; fi
rm -rf /tmp/target-maintip
cargo build --release --target-dir /tmp/target-maintip -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -10
run_mw /tmp/target-maintip/release/gunbc /tmp/maintip_mw.log

echo "=== STEP 3: build PRE-8579 baseline ($PRE8579_FULL) -- true, uncontaminated original-bug baseline ==="
git fetch origin $PRE8579_FULL
git checkout --force $PRE8579_FULL
GOT=$(git rev-parse HEAD)
if [ "$GOT" != "$PRE8579_FULL" ]; then echo "PRE8579FAIL got $GOT want $PRE8579_FULL"; exit 94; fi
rm -rf /tmp/target-pre8579
cargo build --release --target-dir /tmp/target-pre8579 -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -10
run_mw /tmp/target-pre8579/release/gunbc /tmp/pre8579_mw.log

echo "############################################################"
echo "### COMPARISON A: FINAL AFTER  vs  PRE-8579 BEFORE (TRUE DELTA) ###"
echo "############################################################"
comm -23 /tmp/pre8579_mw.log.sorted /tmp/after_mw.log.sorted | tee /tmp/A_cleared.txt | wc -l
comm -12 /tmp/pre8579_mw.log.sorted /tmp/after_mw.log.sorted | tee /tmp/A_surviving.txt | wc -l
comm -13 /tmp/pre8579_mw.log.sorted /tmp/after_mw.log.sorted | tee /tmp/A_newly.txt | wc -l
echo "--- A sample CLEARED ---"; head -30 /tmp/A_cleared.txt
echo "--- A sample SURVIVING ---"; head -30 /tmp/A_surviving.txt
echo "--- A full NEWLY APPEARING ---"; cat /tmp/A_newly.txt

echo "############################################################"
echo "### COMPARISON B: FINAL AFTER  vs  MAIN-TIP BEFORE (CONTAMINATED, but what a reviewer sees against main) ###"
echo "############################################################"
comm -23 /tmp/maintip_mw.log.sorted /tmp/after_mw.log.sorted | tee /tmp/B_cleared.txt | wc -l
comm -12 /tmp/maintip_mw.log.sorted /tmp/after_mw.log.sorted | tee /tmp/B_surviving.txt | wc -l
comm -13 /tmp/maintip_mw.log.sorted /tmp/after_mw.log.sorted | tee /tmp/B_newly.txt | wc -l
echo "--- B sample CLEARED ---"; head -30 /tmp/B_cleared.txt
echo "--- B sample SURVIVING ---"; head -30 /tmp/B_surviving.txt
echo "--- B full NEWLY APPEARING ---"; cat /tmp/B_newly.txt

echo "############################################################"
echo "### MASKED-BY-8579 POPULATION: errors present pre-8579, silently absent at main-tip (already fixed by the hardcode before your PR touched anything) ###"
echo "############################################################"
comm -23 /tmp/pre8579_mw.log.sorted /tmp/maintip_mw.log.sorted | tee /tmp/masked_by_8579.txt | wc -l
cat /tmp/masked_by_8579.txt

echo "############################################################"
echo "### NEWLY-APPEARING (A, the true-delta comparison) SPLIT: (a) genuine wrong-argument vs (b) contract-unavailable coverage-gap ###"
echo "############################################################"
grep -F "method-arg-contract-unavailable" /tmp/A_newly.txt > /tmp/A_newly_b.txt || true
grep -vF "method-arg-contract-unavailable" /tmp/A_newly.txt > /tmp/A_newly_a.txt || true
echo "A newly-appearing (a) genuine wrong-argument count:"; wc -l < /tmp/A_newly_a.txt
cat /tmp/A_newly_a.txt
echo "A newly-appearing (b) contract-unavailable coverage-gap count:"; wc -l < /tmp/A_newly_b.txt
cat /tmp/A_newly_b.txt

echo "############################################################"
echo "### NEW REQUIREMENT: (b) sites split by PROVENANCE -- was this method in #8579's hardcoded list (skip/take/at/nth/index)? ###"
echo "############################################################"
git fetch origin $MERGE_FULL
git checkout --force $MERGE_FULL
HARDCODE_METHODS="skip take at nth index"
> /tmp/b_provenance_report.txt
while IFS= read -r line; do
  [ -z "$line" ] && continue
  LOC=$(echo "$line" | grep -oE '[A-Za-z0-9_./-]+\.dag:[0-9]+' | head -1)
  if [ -z "$LOC" ]; then
    echo "NO_LOCATION | $line" >> /tmp/b_provenance_report.txt
    continue
  fi
  F=$(echo "$LOC" | cut -d: -f1)
  L=$(echo "$LOC" | cut -d: -f2)
  if [ -f "$F" ]; then
    SRC_LINE=$(sed -n "${L}p" "$F")
  else
    SRC_LINE="<file not found: $F>"
  fi
  METHOD=$(echo "$SRC_LINE" | grep -oE '\.[a-zA-Z_][a-zA-Z0-9_]*\(' | tail -1 | sed 's/^\.//; s/(*$//')
  IS_HARDCODED="no"
  for hm in $HARDCODE_METHODS; do
    if [ "$METHOD" = "$hm" ]; then IS_HARDCODED="yes"; fi
  done
  echo "HARDCODED=$IS_HARDCODED METHOD=$METHOD LOC=$LOC | $line | SRC: $SRC_LINE" >> /tmp/b_provenance_report.txt
done < /tmp/A_newly_b.txt

echo "--- full provenance report for (b) sites ---"
cat /tmp/b_provenance_report.txt
echo "--- count HARDCODED=yes (methods whose ONLY prior coverage was #8579's hardcode) ---"
grep -c "^HARDCODED=yes" /tmp/b_provenance_report.txt || true
echo "--- count HARDCODED=no (methods never covered by #8579's hardcode either) ---"
grep -c "^HARDCODED=no" /tmp/b_provenance_report.txt || true
echo "--- count NO_LOCATION (could not extract file:line) ---"
grep -c "^NO_LOCATION" /tmp/b_provenance_report.txt || true

echo "DONE"
