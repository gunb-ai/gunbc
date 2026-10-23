#!/bin/bash
# Product(Unit<>) COMMON FACTOR, round 3. Rounds 1-2 died on MY seed-patch (im::Vector has
# no pop; format-string arg-count and a Debug/Display mix). This round formats by pushing
# Strings only -- one eprintln with a single {} -- so there is no arg-count or Display
# obligation to get wrong. Question unchanged (parent msg_999da06a): at the arm whose
# binding prints Product(Unit<>), what is the value expression? If it is an empty literal
# or unit accumulator mistyped as Unit instead of the declared element type, those 15 rows
# are one producer bug (false positives, kin to the nameless leaf), not three v2-source bugs.
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
                                            let mut __o = String::from("UNIT");
                                            __o.push_str(" bind="); __o.push_str(&bind_name.clone());
                                            __o.push_str(" file="); __o.push_str(&actual.span.file.clone());
                                            __o.push_str(" aspan="); __o.push_str(&actual.span.start.to_string());
                                            __o.push_str("-"); __o.push_str(&actual.span.end.to_string());
                                            __o.push_str(" BOUND="); __o.push_str(&__s1);
                                            __o.push_str(" ACTUAL="); __o.push_str(&__s2);
                                            __o.push_str(" bname="); __o.push_str(&bound.name.clone());
                                            __o.push_str(" bspan="); __o.push_str(&bound.span.file.clone());
                                            __o.push_str(":"); __o.push_str(&bound.span.start.to_string());
                                            __o.push_str("-"); __o.push_str(&bound.span.end.to_string());
                                            __o.push_str(" aname="); __o.push_str(&actual.name.clone());
                                            __o.push_str(" bexpr="); __o.push_str(&format!("{:?}", bound.expr_data.clone()));
                                            __o.push_str(" aexpr="); __o.push_str(&format!("{:?}", actual.expr_data.clone()));
                                            eprintln!("{}", __o);
                                        }
                                    }
                                    ''' + anchor
s=s.replace(anchor, probe)
open(p,'w',encoding='utf-8').write(s)
print("instrumented refusal site (anchor hit)")
PY
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error" | head -8
GUNBC_EMIT_TRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/unit3.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/unit3.log) UNIT=$(grep -ac '^UNIT bind=' /tmp/unit3.log)"
grep "^UNIT bind=" /tmp/unit3.log | head -20
restore_mine
echo "restored seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo UNIT3_DONE
