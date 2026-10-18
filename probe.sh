set -e
cd "$(git rev-parse --show-toplevel)"
S=src/v1/stage0/src
grep -c UnifyGenericsAcc $S/v1_compiler_infer.rs | grep -qv '^0$' || { echo IDENTITY-GUARD-FAIL; exit 9; }
python3 - <<'P'
p='src/v1/stage0/src/v1_std_core.rs'
s=open(p).read()
a="pub fn authored_name_at(\n    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,\n    node: Rc<Node>,\n) -> String {\n"
assert s.count(a)==1
w='''pub fn authored_name_at(
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    node: Rc<Node>,
) -> String {
    let r = authored_name_at_inner(source_indices.clone(), node.clone());
    if r.is_empty() && node.children.len() == 0 && std::env::var("GUNBC_EMIT_TRACE").is_ok() {
        let mut out = String::new();
        out.push_str("QE name=");
        out.push_str(&node.name);
        out.push_str(" conn=");
        out.push_str(&format!("{:?}", node.connective.clone()));
        out.push_str(" ident=");
        out.push_str(&format!("{}", node.ident_span.is_some()));
        out.push_str(" inferred=");
        let inf = format!("{:?}", node.inferred.clone());
        let head: String = inf.chars().take(60).collect();
        out.push_str(&head);
        out.push_str(" expr=");
        let ed = format!("{:?}", node.expr_data.clone());
        let eh: String = ed.chars().take(40).collect();
        out.push_str(&eh);
        eprintln!("{}", out);
    }
    r
}

'''
s=s.replace(a,w+a.replace("authored_name_at(","authored_name_at_inner("))
open(p,'w').write(s)
P
cargo build --release -p v1-compiler --bin gunbc
GUNBC_EMIT_TRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/qe.log 2>&1 || echo "rc=$?"
for f in v1_compiler_infer v1_compiler_infer_types v1_compiler_emit_rust v1_tests_claim_generic_identity_census v1_std_core; do :; done
echo ==HIST2; echo QE-total=$(grep -c "^QE " /tmp/qe.log); echo bound-as=$(grep -c "bound as " /tmp/qe.log)
grep '^QE ' /tmp/qe.log | sed -E 's/^QE name= conn=[A-Za-z]+ ident=[a-z]+ //' | cut -c1-150 | sort | uniq -c | sort -rn | head -30
