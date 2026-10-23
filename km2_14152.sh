#!/bin/bash
# KERNEL-EMPTY-LEAF ROOT CAUSE, round 2 (parent msg_d7ee8f8a ruling 3). Round 1's anchor
# missed because the emitted seed's refusal block has irregular indentation; the anchor is
# now the node_type_equals CALL inside unify_generics (verified count==1 locally).
# Question: 16 census rows print the FIRST binding as Map(Primitive(),Primitive()) at
# <kernel:Map>:0-3. node_type_shape prints a leaf as Primitive(authored_name_at(n)), and
# v1.compiler.core authored_name_at answers "" when BOTH ident_span and name are empty --
# so a blank print does NOT mean the node carries no name. Candidates with opposite fixes:
#  (a) leaf has .name (K/V from container_param_name) but ident_span empty -> builder
#      dropped the span; give it the declared parameter name; then (c) already handles it.
#  (b) leaf is v1.compiler.types missing_kernel_container_profile_type (name:"",
#      ident_span:none, inferred CompilerError, expr_data ExprError) -> an ERROR-shaped
#      node carried as a type; a real defect that must REFUSE, and a nameless-leaf
#      acceptance rule would wrongly accept it.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
restore_mine() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
restore_mine
echo "seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs|cut -c1-16) (want 36 / 5e9c37585fe088ac)"
python3 - <<'PY'
p="src/v1/stage0/src/v1_compiler_infer.rs"
s=open(p,encoding='utf-8').read()
anchor="""if crate::v1_compiler_infer_types::node_type_equals(
                                        bound.clone(),"""
assert s.count(anchor)==1, "anchor=%d" % s.count(anchor)
probe = r'''if std::env::var("GUNBC_EMIT_TRACE").is_ok() {
                                        fn __leaves9(n: &std::rc::Rc<crate::v1_compiler_infer_types::Node>, out: &mut Vec<String>) {
                                            if n.children.is_empty() {
                                                out.push(format!("name={:?} ident_span={:?} inferred={:?} expr_data={:?} decl={} conn={:?} authored={:?}",
                                                    n.name.clone(),
                                                    n.ident_span.clone(),
                                                    n.inferred.clone(),
                                                    (*n.expr_data.clone()).clone(),
                                                    match n.declaration.clone() { Some(d) => format!("{}::{}", d.module_path, d.decl_name), None => "NONE".into() },
                                                    n.connective.clone(),
                                                    crate::v1_compiler_infer_types::authored_name_at(source_indices.clone(), n.clone())));
                                            } else { for c in n.children.iter() { __leaves9(c, out); } }
                                        }
                                        let __sl9 = crate::v1_compiler_infer_types::node_type_shape(bound.clone(), source_indices.clone());
                                        let __sr9 = crate::v1_compiler_infer_types::node_type_shape(actual.clone(), source_indices.clone());
                                        if __sl9.contains("Primitive()") || __sr9.contains("Primitive()") {
                                            let mut lv9 = Vec::new(); __leaves9(&bound, &mut lv9);
                                            let mut la9 = Vec::new(); __leaves9(&actual, &mut la9);
                                            eprintln!("KM span={}:{}-{} module={} bind={}\n  BOUND {}\n  ACTUAL {}\n  bound-leaves:\n    {}\n  actual-leaves:\n    {}",
                                                actual.span.file, actual.span.start, actual.span.end, module_name, bind_name.clone(), __sl9, __sr9,
                                                lv9.join("\n    "), la9.join("\n    "));
                                        }
                                    }
                                    ''' + anchor
s=s.replace(anchor, probe)
open(p,'w',encoding='utf-8').write(s)
print("instrumented (anchor hit)")
PY
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2
GUNBC_EMIT_TRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/km2.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/km2.log) KM=$(grep -ac '^KM span' /tmp/km2.log)"
grep -A8 "^KM span" /tmp/km2.log | head -60
restore_mine
echo "restored seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo KM2_DONE
