#!/bin/bash
# THE Product(Unit<>) COMMON FACTOR (parent msg_999da06a). All three 'true positive'
# groups have Product(Unit<>) on one side -- suspicious enough that a real misbinding is
# questionable; it may be one producer typing an EMPTY value ([] / unit accumulator) as
# Unit instead of the declared element type, which would make those rows false positives
# close kin to the nameless leaf. Method: at the fold's refusal site in the SEED, print
# for each side the node name, span, and each IMMEDIATE CHILD's name/expr_data/inferred,
# filtered to the three suspect sites. Rounds 1-3 of an earlier probe died on E0434/E0282
# from closures and nested fns, so this version uses only explicit while-pop stacks and
# Debug on Rc fields (no deref, no captures, no nested fn). NO git checkout/clean.
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
                                        let __s1 = crate::v1_compiler_infer_types::node_type_shape(bound.clone(), source_indices.clone());
                                        let __s2 = crate::v1_compiler_infer_types::node_type_shape(actual.clone(), source_indices.clone());
                                        let __hit = (__s1.contains("Product(Unit)") || __s2.contains("Product(Unit)"))
                                            && (actual.span.file.contains("std/diagnostic.dag")
                                                || actual.span.file.contains("languages/python.dag")
                                                || actual.span.file.contains("source_root_eval.dag"));
                                        if __hit {
                                            let mut __v: Vec<String> = Vec::new();
                                            let mut __st: Vec<Rc<Node>> = Vec::new();
                                            __st.push(bound.clone());
                                            let mut __d = 0i64;
                                            while __st.len() > 0 {
                                                let __n = __st.pop().unwrap();
                                                __v.push(format!("d{} name={:?} children={} expr_data={:?} inferred={:?} span={}:{}-{}",
                                                    __d, __n.name.clone(), __n.children.len(),
                                                    __n.expr_data.clone(), __n.inferred.clone(),
                                                    __n.span.file.clone(), __n.span.start, __n.span.end));
                                                if __d < 2 {
                                                    let mut __i = 0usize;
                                                    while __i < __n.children.len() {
                                                        __st.push(__n.children[__i].clone());
                                                        __i = __i + 1;
                                                    }
                                                }
                                                __d = __d + 1;
                                            }
                                            let mut __w: Vec<String> = Vec::new();
                                            let mut __st2: Vec<Rc<Node>> = Vec::new();
                                            __st2.push(actual.clone());
                                            let mut __e = 0i64;
                                            while __st2.len() > 0 {
                                                let __m = __st2.pop().unwrap();
                                                __w.push(format!("d{} name={:?} children={} expr_data={:?} inferred={:?} span={}:{}-{}",
                                                    __e, __m.name.clone(), __m.children.len(),
                                                    __m.expr_data.clone(), __m.inferred.clone(),
                                                    __m.span.file.clone(), __m.span.start, __m.span.end));
                                                if __e < 2 {
                                                    let mut __j = 0usize;
                                                    while __j < __m.children.len() {
                                                        __st2.push(__m.children[__j].clone());
                                                        __j = __j + 1;
                                                    }
                                                }
                                                __e = __e + 1;
                                            }
                                            eprintln!("UNIT file={} span={}:{}-{} bind={}\n  BOUND {}\n  ACTUAL {}\n  BOUND-TREE:\n    {}\n  ACTUAL-TREE:\n    {}",
                                                actual.span.file.clone(), actual.span.start, actual.span.end, bind_name.clone(), __s1, __s2,
                                                __v.join("\n    "), __w.join("\n    "));
                                        }
                                    }
                                    ''' + anchor
s=s.replace(anchor, probe)
open(p,'w',encoding='utf-8').write(s)
print("instrumented refusal site (anchor hit)")
PY
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|warning: unused" | head -8
GUNBC_EMIT_TRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/unit.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/unit.log) UNIT=$(grep -ac '^UNIT file=' /tmp/unit.log)"
grep -A12 "^UNIT file=" /tmp/unit.log | head -70
restore_mine
echo "restored seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo UNIT_DONE
