#!/bin/bash
# KERNEL-EMPTY-LEAF ROOT CAUSE, round 3. Round 2 failed E0434 (a fn item cannot capture
# source_indices); this version walks the tree iteratively so nothing is captured, and calls
# authored_name_at through its real home v1.std.core (re-exported into this module).
# Same question as rounds 1-2 (parent msg_d7ee8f8a ruling 3): the 16 KERNEL_EMPTY_LEAF rows
# print the FIRST binding as Map(Primitive(),Primitive()) at <kernel:Map>:0-3. Is the leaf
# (a) named K/V with an empty ident_span [builder dropped the span -> fix the builder, (c)
# then handles it], or (b) v1.compiler.types missing_kernel_container_profile_type, which is
# name:"", ident_span:none, inferred CompilerError, expr_data ExprError [an ERROR-shaped
# node carried as a type -> must refuse; a nameless-leaf acceptance rule would wrongly
# accept it]. Report which before building.
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
                                        let __sl9 = crate::v1_compiler_infer_types::node_type_shape(bound.clone(), source_indices.clone());
                                        let __sr9 = crate::v1_compiler_infer_types::node_type_shape(actual.clone(), source_indices.clone());
                                        if __sl9.contains("Primitive()") || __sr9.contains("Primitive()") {
                                            let mut lv9: Vec<String> = Vec::new();
                                            let mut st9: Vec<Rc<Node>> = vec![bound.clone()];
                                            while let Some(n) = st9.pop() {
                                                if n.children.is_empty() {
                                                    lv9.push(format!("name={:?} ident_span={:?} inferred={:?} expr_data={:?} decl={} conn={:?} authored={:?}",
                                                        n.name.clone(), n.ident_span.clone(), n.inferred.clone(),
                                                        (*n.expr_data.clone()).clone(),
                                                        match n.declaration.clone() { Some(d) => format!("{}::{}", d.module_path, d.decl_name), None => "NONE".to_string() },
                                                        n.connective.clone(),
                                                        crate::v1_std_core::authored_name_at(source_indices.clone(), n.clone())));
                                                } else { for c in n.children.iter().rev() { st9.push(c.clone()); } }
                                            }
                                            let mut la9: Vec<String> = Vec::new();
                                            let mut sa9: Vec<Rc<Node>> = vec![actual.clone()];
                                            while let Some(n) = sa9.pop() {
                                                if n.children.is_empty() {
                                                    la9.push(format!("name={:?} ident_span={:?} inferred={:?} expr_data={:?} decl={} conn={:?} authored={:?}",
                                                        n.name.clone(), n.ident_span.clone(), n.inferred.clone(),
                                                        (*n.expr_data.clone()).clone(),
                                                        match n.declaration.clone() { Some(d) => format!("{}::{}", d.module_path, d.decl_name), None => "NONE".to_string() },
                                                        n.connective.clone(),
                                                        crate::v1_std_core::authored_name_at(source_indices.clone(), n.clone())));
                                                } else { for c in n.children.iter().rev() { sa9.push(c.clone()); } }
                                            }
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
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -3
GUNBC_EMIT_TRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/km3.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/km3.log) KM=$(grep -ac '^KM span' /tmp/km3.log)"
grep -A8 "^KM span" /tmp/km3.log | head -56
restore_mine
echo "restored seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo KM3_DONE
