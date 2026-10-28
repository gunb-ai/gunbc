set -u
export CARGO_TARGET_DIR=$PWD/target/qb
CC=src/v1/stage0/src/cli_run/compile_clean.rs
B() { cargo build --release -p v1-compiler --bin gunbc --bin claim_executor --bin v1_src_dag_parse --bin claim_batch 2>&1 | grep -E "^error|Finished|could not compile" | head -40; }
cp docs/qb_cc_new.rs $CC
: > /tmp/changed.txt
B
ls target/qb/release/claim_executor || exit 4
echo "PARSE gate skipped (pre-existing corpus hang, reproduced on base a545526)"
for round in 1 2; do
  echo "=== REGEN ROUND $round $(date +%T)"
  rm -rf target/stage0-regen-candidate
  ./target/qb/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/regen$round.log 2>&1
  echo "regen exit=$?"; grep -E "FAIL|first_generation_equal|panick|refus" /tmp/regen$round.log | head -20 | cut -c1-300
  test -d target/stage0-regen-candidate/src || { echo "NO CANDIDATE"; tail -20 /tmp/regen$round.log; break; }
  for c in target/stage0-regen-candidate/src/*.rs; do grep -Hn 'unsupported cast from' "$c" | sed 's#target/stage0-regen-candidate/src/##' | cut -c1-300; done | sort > /tmp/panics$round.txt
  for c in src/v1/stage0/src/*.rs; do grep -Hn 'unsupported cast from' "$c" | sed 's#src/v1/stage0/src/##' | cut -c1-300; done | sort > /tmp/panics_installed.txt
  echo "PANICS candidate=$(wc -l < /tmp/panics$round.txt) installed=$(wc -l < /tmp/panics_installed.txt)"; comm -13 /tmp/panics_installed.txt /tmp/panics$round.txt | sed 's/^/NEWPANIC /' | head -30
  changed=0
  for c in target/stage0-regen-candidate/src/*.rs; do n=$(basename $c); if ! cmp -s "$c" "src/v1/stage0/src/$n"; then echo "CHANGED $n"; echo $n >> /tmp/changed.txt; cp "$c" "src/v1/stage0/src/$n"; changed=1; fi; done
  cp docs/qb_cc_new.rs $CC
  if [ $changed = 0 ]; then echo "FIXED POINT at round $round"; break; fi
  B
done
cp docs/qb_cc_new.rs $CC
echo "=== WITNESS"
./target/qb/release/claim_batch --source-root dag --source-root src/v2 --entry dag/test/claim/emitter_nested_refinement_cast_witness.dag --functions single_refinement_carrier_emits_no_unsupported_cast,nested_refinement_carrier_emits_no_unsupported_cast,intermediate_refinement_casts_emit_no_unsupported_cast,brand_refinement_casts_emit_no_unsupported_cast,unrelated_carriers_still_refuse_the_cast,alias_of_refinement_cast_emits_the_operand,generic_call_cast_emits_the_operand,empty_list_cast_emits_the_operand,nat_as_int_inclusion_emits_the_operand,numeric_conversion_still_uses_cast_syntax --hermetic --claim-run 2>&1 | grep -E "^(PASS|FAIL|UNOBSERVED)" | cut -c1-300
echo "=== TAR BEGIN"
sort -u /tmp/changed.txt | sed 's#^#src/v1/stage0/src/#' | tar czf - -T - | base64 -w0
echo
echo "=== TAR END"
