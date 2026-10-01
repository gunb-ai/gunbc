export GUNBC_MEMORY_BUDGET_BYTES=30064771072
MODE=$1
if [ "$MODE" = base ]; then git fetch -q --depth=1 origin c48221ef64829f27db5248baf6450c173bb0b4cb && git checkout -q c48221ef64829f27db5248baf6450c173bb0b4cb -- src dag && git clean -fdq -- src dag || echo BASEFAIL; fi
FILES="src/v2/test/claim/parse/choice_plan_test.dag src/v2/test/claim/body_lowering/single_arm_match_test.dag src/v2/test/claim/body_lowering/match_as_field_name_test.dag src/v2/test/claim/body_lowering/declaration_structure_preserved_test.dag src/v2/test/claim/body_lowering/data_initializer_fn_value_test.dag"
for f in $FILES; do grep -q "^import v2.std.live_tree" $f || sed -i '0,/^import /s//import v2.std.live_tree { LiveTreeDisposition, SubstrateInputsOnly }\nimport /' $f; done
for f in $FILES; do grep -v "file: \"$f\"" src/v2/workflow/floor_unimported_bare_provider_debt_roster.dag > /tmp/r && cp /tmp/r src/v2/workflow/floor_unimported_bare_provider_debt_roster.dag; done
grep -c "choice_plan_test" src/v2/workflow/floor_unimported_bare_provider_debt_roster.dag
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
for f in $FILES; do
  fns=$(grep -h '^test fn' $f | sed -E 's/^test fn ([A-Za-z0-9_]+).*/\1/' | paste -sd,)
  timeout 1500 ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $f --functions $fns > /tmp/o.txt 2>&1; echo "=== $MODE $f rc=$?"
  grep -E "^(PASS|FAIL) |ENTRY REFUSAL|resolve failed|error:" /tmp/o.txt | cut -c1-300 | sed "s/^/$MODE /"
done
