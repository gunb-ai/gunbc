#!/bin/bash
# THE Product(Unit<>) COMMON FACTOR, round 2 (parent msg_999da06a). Round 1 died because
# the seed's Vec is im::Vector (no pop) and one format string had 9 slots / 8 args. This
# version walks NOTHING: it prints the two ROOT spans plus each root's expr_data, and the
# source text is resolved locally from those offsets. That answers the question directly
# -- what expression sits at the arm whose binding prints Product(Unit<>).
# Suspect sites: src/v2/std/diagnostic.dag, src/v2/extdeps/languages/python.dag,
# src/v2/compiler/source_root_eval.dag -- one side prints Product(Unit<>) at each.
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
                                            eprintln!(
                                                "UNIT bind={} file={} aspan={}:{}-{} ashape={}\n  bname={:?} bspan={}:{}-{} bexpr={:?} binferred={:?}\n  aname={:?} aexpr={:?} ainferred={:?}",
                                                bind_name.clone(),
                                                actual.span.file.clone(), actual.span.start, actual.span.end, __s2,
                                                bound.name.clone(), bound.span.file.clone(), bound.span.start, bound.span.end, bound.expr_data.clone(), bound.inferred.clone(),
                                                actual.name.clone(), actual.expr_data.clone(), actual.inferred.clone()
                                            );
                                            eprintln!("  BOUND-SHAPE {}", __s1);
                                        }
                                    }
                                    ''' + anchor
s=s.replace(anchor, probe)
open(p,'w',encoding='utf-8').write(s)
print("instrumented refusal site (anchor hit)")
PY
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error" | head -8
GUNBC_EMIT_TRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/unit2.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/unit2.log) UNIT=$(grep -ac '^UNIT bind=' /tmp/unit2.log)"
grep -A2 "^UNIT bind=" /tmp/unit2.log | head -60
restore_mine
echo "restored seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo UNIT2_DONE
