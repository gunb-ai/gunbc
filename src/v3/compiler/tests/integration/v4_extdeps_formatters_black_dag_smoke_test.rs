//! Smoke `compile_to_dag` on `src/v4/extdeps/formatters/black.dag` — T-4.16
//! `ConfigPatchRecord` / `config_patch_layer` consumers must lower+infer with zero
//! module diagnostics (no `apply_field_patch` in consumer imports).

use v3_compiler::compile_to_dag;
use v3_compiler::dag::TypeConnective;
use v3_compiler::CompileError;

const BLACK_DAG: &str = include_str!("../../../../v4/extdeps/formatters/black.dag");
const BLACK_PATH: &str = "src/v4/extdeps/formatters/black.dag";

#[test]
fn v4_extdeps_formatters_black_dag_compiles_with_config_patch_projection() {
    let dag = match compile_to_dag(BLACK_DAG, BLACK_PATH) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => {
            panic!(
                "{BLACK_PATH}: semantic errors: {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
        }
        Err(other) => panic!("{BLACK_PATH}: {other:?}"),
    };
    assert!(
        dag.diagnostics().is_empty(),
        "{BLACK_PATH}: expected empty diagnostics, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    let patch = dag
        .declaration_by_name("BlackConfigPatch")
        .expect("BlackConfigPatch type alias should exist");
    let field_patch = dag
        .declaration_by_name("FieldPatch")
        .expect("FieldPatch template should resolve");
    let TypeConnective::Conj { children } = &patch.connective else {
        panic!(
            "BlackConfigPatch: expected materialized Conj, got {:?}",
            patch.connective
        );
    };
    assert!(
        !children.is_empty(),
        "BlackConfigPatch should have at least one FieldPatch field"
    );
    for field in children {
        let TypeConnective::Instantiation { template, .. } = &dag.declaration(field.ty).connective
        else {
            panic!(
                "BlackConfigPatch field `{}` should be FieldPatch<T>",
                field.label
            );
        };
        assert_eq!(
            *template, field_patch.id,
            "BlackConfigPatch field `{}` should instantiate FieldPatch",
            field.label
        );
    }
}
