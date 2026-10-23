#!/bin/bash
# UNSETTLED-GENERIC-AS-UNIT hunt (parent msg_fca596f5). A nested call's result type IS
# legitimately the inner callee's return annotation with ITS substitution applied, so the
# question is why the inner T became Product(Unit<>). If the inner T was never settled and
# something DEFAULTED it to Unit, that is a fabricated type (§5) at the substitution step,
# and the fold is only the first consumer to notice. Measure the substitution, do not guess.
#
# Instrument v1.compiler.infer substitute_generics_apply in the SEED, at both arms that
# apply a substitution (the TypeVariable arm and the bare-leaf arm). Print, ONLY when the
# inserted node's printed shape contains 'Unit', the variable/leaf name, the inserted
# shape, the inserted node's span/expr_data/inferred, and whether the variable had any
# binding at all. Nothing is printed for ordinary substitutions, so the volume stays small.
# All formatting via push_str on one String (rounds 1-2 of my earlier probes died on
# eprintln arg counts, im::Vector::pop and Debug-on-Rc<ExprData>).
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/usr/local/bin:/usr/local/sbin:$PATH
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

# Arm 1: TypeVariable arm -- a binding IS found for the variable.
a1 = """                    Some(c) => {
                        if (type_node_label(c.clone(), source_indices.clone()) == tv_id.clone()) {"""
assert s.count(a1)==1, "a1=%d" % s.count(a1)
w1 = """                    Some(c) => {
                        if std::env::var("GUNBC_EMIT_TRACE").is_ok() {
                            let __sh = crate::v1_compiler_infer_types::node_type_shape(c.clone(), source_indices.clone());
                            if __sh.contains("Unit") {
                                let mut __o = String::from("SUBTV var=");
                                __o.push_str(&tv_id.clone());
                                __o.push_str(" inserted="); __o.push_str(&__sh);
                                __o.push_str(" ins_name="); __o.push_str(&c.name.clone());
                                __o.push_str(" ins_span="); __o.push_str(&c.span.file.clone());
                                __o.push_str(":"); __o.push_str(&c.span.start.to_string());
                                __o.push_str("-"); __o.push_str(&c.span.end.to_string());
                                __o.push_str(" ins_children="); __o.push_str(&c.children.len().to_string());
                                __o.push_str(" ins_expr="); __o.push_str(&format!("{:?}", c.expr_data.clone()));
                                __o.push_str(" ins_inferred="); __o.push_str(&format!("{:?}", c.inferred.clone()));
                                __o.push_str(" node_span="); __o.push_str(&n.span.file.clone());
                                __o.push_str(":"); __o.push_str(&n.span.start.to_string());
                                __o.push_str("-"); __o.push_str(&n.span.end.to_string());
                                eprintln!("{}", __o);
                            }
                        }
"""
s = s.replace(a1, w1 + a1)

# Arm 2: bare-leaf arm -- a binding found for the leaf NAME.
a2 = """                    Some(c) => crate::v1_std_core::preserve_outer_optional_cardinality(
                        n.clone(),
                        c.clone(),
                    ),"""
n2 = s.count(a2)
print("bare-leaf arm count:", n2)
if n2 == 1:
    w2 = """                    Some(c) => {
                        if std::env::var("GUNBC_EMIT_TRACE").is_ok() {
                            let __sh = crate::v1_compiler_infer_types::node_type_shape(c.clone(), source_indices.clone());
                            if __sh.contains("Unit") {
                                let mut __o = String::from("SUBLEAF leaf=");
                                __o.push_str(&type_node_label(n.clone(), source_indices.clone()));
                                __o.push_str(" inserted="); __o.push_str(&__sh);
                                __o.push_str(" ins_name="); __o.push_str(&c.name.clone());
                                __o.push_str(" ins_span="); __o.push_str(&c.span.file.clone());
                                __o.push_str(":"); __o.push_str(&c.span.start.to_string());
                                __o.push_str("-"); __o.push_str(&c.span.end.to_string());
                                __o.push_str(" ins_expr="); __o.push_str(&format!("{:?}", c.expr_data.clone()));
                                __o.push_str(" ins_inferred="); __o.push_str(&format!("{:?}", c.inferred.clone()));
                                __o.push_str(" node_span="); __o.push_str(&n.span.file.clone());
                                __o.push_str(":"); __o.push_str(&n.span.start.to_string());
                                __o.push_str("-"); __o.push_str(&n.span.end.to_string());
                                eprintln!("{}", __o);
                            }
                        }
                        crate::v1_std_core::preserve_outer_optional_cardinality(
                            n.clone(),
                            c.clone(),
                        )
                    },"""
    s = s.replace(a2, w2)
open(p,'w',encoding='utf-8').write(s)
print("instrumented substitution arms")
PY
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error" | head -8
GUNBC_EMIT_TRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/sub.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/sub.log) SUBTV=$(grep -ac '^SUBTV ' /tmp/sub.log) SUBLEAF=$(grep -ac '^SUBLEAF ' /tmp/sub.log)"
echo "--- distinct SUBTV (var / inserted / ins_span):"
grep "^SUBTV " /tmp/sub.log | sed 's/ ins_children.*//' | sort | uniq -c | sort -rn | head -12
echo "--- distinct SUBLEAF:"
grep "^SUBLEAF " /tmp/sub.log | sed 's/ ins_expr.*//' | sort | uniq -c | sort -rn | head -12
restore_mine
echo "restored seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo SUB_DONE
