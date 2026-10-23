#!/bin/bash
# KERNEL-EMPTY-LEAF ROOT CAUSE (parent msg_d7ee8f8a ruling 3: "Why does the kernel Map
# template reach the fold with NAMELESS leaves? ... Report which it is before building").
# node_type_shape prints a leaf as Primitive(authored_name_at(...)), and v1.compiler.core
# authored_name_at returns "" when ident_span is Absent -- so a blank print does NOT mean
# the node has no NAME. Two candidate producers with opposite fixes:
#   (a) a node that HAS .name (e.g. "K") but ident_span: none  -> the builder dropped the
#       ident span; give it the declared parameter name/span and (c) handles the row.
#   (b) v1.compiler.types missing_kernel_container_profile_type (name: "", ident_span: none,
#       inferred CompilerError, expr_data ExprError) -> an ERROR-shaped node is being carried
#       as a type; separate defect, must refuse, not a naming fix.
# Method: instrument the fold's refusal site in the SEED and dump, for each refusal whose
# first binding prints with an empty leaf name, the leaf fields (.name, ident_span, inferred,
# expr_data) of every leaf in the stored binding. Throwaway seed patch; mirrors restored by
# cp from carriers (NEVER git checkout on the runner). NO git clean.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
restore_mine() { for f in $MIRROR_FILES; do cp "$HOME/../workspace/repo-root/seedgreen/$(basename $f)" "$f" 2>/dev/null || cp "seedgreen/$(basename $f)" "$f"; done; }
restore_mine
echo "seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs|cut -c1-16) (want 36 / 5e9c37585fe088ac)"
python3 - <<'PY'
p="src/v1/stage0/src/v1_compiler_infer.rs"
s=open(p,encoding='utf-8').read()
anchor='''break Rc::new(UnifyGenericsAcc {
                                    subst: acc.clone(),
                                    diagnostics: Rc::new(vec![type_mismatch_error('''
assert s.count(anchor)==1, "anchor=%d" % s.count(anchor)
probe = r'''if std::env::var("GUNBC_EMIT_TRACE").is_ok() {
                                        fn __leaves(n: &std::rc::Rc<crate::v1_compiler_infer_types::Node>, out: &mut Vec<String>) {
                                            if n.children.is_empty() {
                                                out.push(format!("name={:?} ident_span={} inferred={:?} expr_data={:?} decl={}",
                                                    n.name.clone(),
                                                    if n.ident_span.is_some() {"Some"} else {"NONE"},
                                                    n.inferred.clone(),
                                                    (*n.expr_data.clone()).clone(),
                                                    match n.declaration.clone() { Some(d) => format!("{}::{}", d.module_path, d.decl_name), None => "NONE".into() }));
                                            } else { for c in n.children.iter() { __leaves(c, out); } }
                                        }
                                        let __sl = crate::v1_compiler_infer_types::node_type_shape(bound.clone(), source_indices.clone());
                                        let __sr = crate::v1_compiler_infer_types::node_type_shape(actual.clone(), source_indices.clone());
                                        if __sl.contains("Primitive()") || __sr.contains("Primitive()") {
                                            let mut lv = Vec::new(); __leaves(&bound, &mut lv);
                                            let mut la = Vec::new(); __leaves(&actual, &mut la);
                                            eprintln!("KM row span={}:{}-{} module={} bind={}\n  FORMAL {}\n  ACTUAL {}\n  bound-leaves:\n    {}\n  actual-leaves:\n    {}",
                                                actual.span.file, actual.span.start, actual.span.end, module_name, bind_name.clone(), __sl, __sr,
                                                lv.join("\n    "), la.join("\n    "));
                                        }
                                    }
                                    ''' + anchor
s=s.replace(anchor, probe)
open(p,'w',encoding='utf-8').write(s)
print("instrumented fold refusal site")
PY
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2
GUNBC_EMIT_TRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/km.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/km.log) KM=$(grep -ac '^KM row' /tmp/km.log)"
grep -A9 "^KM row" /tmp/km.log | head -46
restore_mine
echo "restored seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo KM_DONE
