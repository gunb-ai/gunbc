set -o pipefail
F=dag/gunbc/recurring_failure_mode/optional_equality_answers_by_representation.dag
git rev-parse HEAD; git status --porcelain | grep -v '^??' | head -3
cargo build --release -p v1-compiler --bin v1_src_dag_parse 2>&1 | tail -2
echo "==CLEAN"; ./target/release/v1_src_dag_parse > /tmp/clean.out 2>&1; echo rc=$?; grep -c "CITED-.*-ABSENT" /tmp/clean.out; grep -c "optional_equality_answers" /tmp/clean.out
sed -i '/^  evidence: \[/a\    DeclarationRef { module_path: "v1.compiler.infer_types", decl_name: "no_such_forged_declaration_zz", field: WholeDeclaration },' $F
echo "==FORGED"; ./target/release/v1_src_dag_parse > /tmp/forged.out 2>&1; echo rc=$?; grep -c 'CITED-.*-ABSENT' /tmp/forged.out; grep 'CITED-.*-ABSENT' /tmp/forged.out | head -3 | cut -c1-300
git checkout -- $F; git status --porcelain | grep -v '^??' | head -2; echo END
