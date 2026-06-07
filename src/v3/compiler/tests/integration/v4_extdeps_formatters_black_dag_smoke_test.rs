//! Smoke `compile_to_dag` on `src/v4/extdeps/formatters/black.dag` — T-4.16
//! `ConfigPatchRecord` / `config_patch_layer` consumers must lower+infer with zero
//! module diagnostics (no `apply_field_patch` in consumer imports).
//!
//! Single-file [`compile_to_dag`] cannot load `v4.std.patch` peers; this harness
//! lowers `node` → `algebra` → `patch` → `black` in order (flat declaration table).
//!
//! **Wave-A W1 classification (B-interim, stays as-is):** 0 foldable-now-A. The behavioral
//! content (config-patch projection) is NOT witnessable through v2 `run --claim-run`:
//! `config_patch_layer` is a v3-lowerer-expanded stub (its surface body is identity and the
//! lowerer rejects direct execution), and `ConfigPatchRecord` is not constructible in plain
//! v4 surface. Tag: **config_patch-v3lowerer-gap** (a construction/lowerer dependency, NOT a
//! v2 crash) — distinct from the reflection consumers. TRIGGER: migrate when the config-patch
//! record can be constructed + projected through v2 (substrate lane). No standalone PR.

use v3_compiler::compile_to_dag_modules_in_order;
use v3_compiler::dag::TypeConnective;
use v3_compiler::CompileError;

const NODE_DAG: &str = include_str!("../../../../v4/std/node.dag");
const NODE_PATH: &str = "src/v4/std/node.dag";
const ALGEBRA_DAG: &str = include_str!("../../../../v4/std/algebra.dag");
const ALGEBRA_PATH: &str = "src/v4/std/algebra.dag";
const PATCH_DAG: &str = include_str!("../../../../v4/std/patch.dag");
const PATCH_PATH: &str = "src/v4/std/patch.dag";
const BLACK_DAG: &str = include_str!("../../../../v4/extdeps/formatters/black.dag");
const BLACK_PATH: &str = "src/v4/extdeps/formatters/black.dag";

fn black_dag_or_panic() -> v3_compiler::dag::Dag {
    let sources = [
        (NODE_DAG, NODE_PATH),
        (ALGEBRA_DAG, ALGEBRA_PATH),
        (PATCH_DAG, PATCH_PATH),
        (BLACK_DAG, BLACK_PATH),
    ];
    match compile_to_dag_modules_in_order(&sources) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => {
            panic!(
                "{BLACK_PATH}: semantic errors: {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
        }
        Err(other) => panic!("{BLACK_PATH}: {other:?}"),
    }
}

#[test]
fn v4_extdeps_formatters_black_dag_compiles_with_config_patch_projection() {
    let dag = black_dag_or_panic();
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
