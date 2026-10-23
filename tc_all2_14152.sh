#!/bin/bash
# ONE dispatch, two measurements, round 2.
#  [1] TC  : same-text-refused FIELD AXIS (node_type_equals vs node_type_shape, per field).
#  [2] SPEC: SPECIMEN dump inside the emitting branch named by the backtrace
#            (direct_call_structured_application_mismatch_diags -> 
#             text_crossing_or_type_mismatch_error) for the 02_parse same-text pair:
#            both nodes' fields + which of the 3 sub-predicates fired.
# Throwaway seed patch; mirrors restored by cp from seedgreen/ (NEVER git checkout).
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
restore_mine() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
restore_mine
echo "TC: seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16) (want 5e9c37585fe088ac)"

echo "########## [1] TC FIELD AXIS ##########"
cp src/v1/tests/src/lib.rs /tmp/lib.rs.bak.tc
cp tc_probe_body.rs src/v1/tests/src/tc_probe_test.rs
grep -q 'mod tc_probe_test;' src/v1/tests/src/lib.rs || echo 'mod tc_probe_test;' >> src/v1/tests/src/lib.rs
cargo test -p v1-compiler-tests --lib tc_ -- --nocapture 2>&1 | grep -E "^TC |^error|test result|panicked" | head -60
cp /tmp/lib.rs.bak.tc src/v1/tests/src/lib.rs
rm -f src/v1/tests/src/tc_probe_test.rs
echo "TC: restored lib.rs"

echo "########## [2] SPEC SPECIMEN DUMP ##########"
restore_mine
python3 - <<'PY'
p="src/v1/stage0/src/v1_compiler_infer.rs"
s=open(p,encoding='utf-8').read()
anchor="""Rc::new(vec![text_crossing_or_type_mismatch_error(formal.clone(), actual.clone(), type_env.clone(), actual_expr.span.clone(), module_name.clone())])"""
assert s.count(anchor)==1, "anchor count %d" % s.count(anchor)
inject = r'''if std::env::var("GUNBC_EMIT_TRACE").is_ok() {
                            let __sl = crate::v1_compiler_infer_types::node_type_shape(formal.clone(), source_indices.clone());
                            let __sr = crate::v1_compiler_infer_types::node_type_shape(actual.clone(), source_indices.clone());
                            if actual_expr.span.file.contains("02_parse") && __sl == __sr {
                                let __fd = match formal.declaration.clone() { Some(d) => format!("{}::{}::{:?}", d.module_path, d.decl_name, d.field), None => "NONE".to_string() };
                                let __ad = match actual.declaration.clone() { Some(d) => format!("{}::{}::{:?}", d.module_path, d.decl_name, d.field), None => "NONE".to_string() };
                                let __eq = crate::v1_compiler_infer_types::node_type_equals(formal.clone(), actual.clone(), source_indices.clone(), std::rc::Rc::new(crate::v1_compiler_infer_types::TextJudgment::TextNotAsked { reason: crate::v1_compiler_infer_types::TextNotAskedReason::TextNotAskedForSyntheticWitnessNodes }));
                                eprintln!("SPEC span={}:{}-{} module={}\n  shape={}\n  FORMAL name={:?} conn={:?} children={:?} decl={} inferred={}\n  ACTUAL name={:?} conn={:?} children={:?} decl={} inferred={}\n  equals={}\n  LIT={} STRUCT={} RECLIT={}\n  actual_expr_data={:?}",
                                    actual_expr.span.file, actual_expr.span.start, actual_expr.span.end, module_name, __sl,
                                    formal.name.clone(), formal.connective.clone(), formal.children.iter().map(|c| c.name.clone()).collect::<Vec<String>>(), __fd, formal.inferred.is_some(),
                                    actual.name.clone(), actual.connective.clone(), actual.children.iter().map(|c| c.name.clone()).collect::<Vec<String>>(), __ad, actual.inferred.is_some(),
                                    __eq,
                                    literal_introduction_type_mismatch(formal.clone(), actual_expr.clone(), scope.clone()),
                                    structured_application_site_type_mismatch_with_peeled(formal_subst.clone(), formal.clone(), carried_structural_type_name(formal.clone(), source_indices.clone(), 0), actual_expr.clone(), scope.clone()),
                                    direct_call_structured_record_literal_resolved_type_mismatch(formal.clone(), app.formal.clone().substitution_basis.clone(), actual_expr.clone(), type_env.clone(), module_name.clone(), source_indices.clone()),
                                    (*actual_expr.expr_data.clone()).clone());
                            }
                        }
                        ''' + anchor
s=s.replace(anchor, inject)
open(p,'w',encoding='utf-8').write(s)
print("SPEC: instrumented emitting branch")
PY
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -3
GUNBC_EMIT_TRACE=1 timeout 2400 target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/spec_regen.log 2>&1
echo "rc=$? bound-as=$(grep -c 'bound as' /tmp/spec_regen.log) SPEC=$(grep -c '^SPEC span' /tmp/spec_regen.log)"
grep -A9 "^SPEC span" /tmp/spec_regen.log | head -40
restore_mine
echo "SPEC: restored; seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo ALL_DONE
