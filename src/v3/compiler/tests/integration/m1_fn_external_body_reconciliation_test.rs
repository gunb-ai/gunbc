// DB-16 — `FnExternalBody` / `ArrowBody::Unparsed` semantic reconciliation.
//
// Host-backed pipeline stages parse as `FnExternalBody` → `Unparsed`, then
// `materialize_pipeline_realizations` rewrites each bound stage to
// `ExternalRealization`. This test locks that case-2 path.

use v3_compiler::dag::{ArrowBody, Dag, TypeConnective};

/// Pipeline stage order is read structurally from `PipelineStageBinding`
/// declarations in the Dag — the `fn compile { ... }` body span is no
/// longer the ordering authority. Pin the literal sequence so a stray
/// reorder of bindings in `pipeline.dag` fails closed (rather than
/// agreeing with itself through the same code path).
#[test]
fn pipeline_stage_order_is_read_structurally_from_bindings() {
    let actual = v3_compiler::pipeline_compile_order_stage_names()
        .expect("pipeline stage order must resolve");

    let expected = vec![
        "parse".to_string(),
        "lower".to_string(),
        "infer".to_string(),
        "compute_ownership".to_string(),
        "lens_complexity".to_string(),
        "emit".to_string(),
    ];

    assert_eq!(
        actual, expected,
        "pipeline stage order must match the canonical L1.5 pipeline; \
         update `pipeline.dag` bindings and `fn compile` body together"
    );
}

#[test]
fn pipeline_stages_lower_to_external_realization_not_unparsed() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap DAG should be clean, got {:?}",
        dag.diagnostics()
    );

    let stages = v3_compiler::pipeline_compile_order_stage_names()
        .expect("pipeline.dag `compile` body must list stages");
    for stage in stages {
        let decl = dag.declaration_by_name(&stage).unwrap_or_else(|| {
            panic!("pipeline stage `{stage}` missing from bootstrap DAG");
        });
        let TypeConnective::Arrow { body, .. } = &decl.connective else {
            panic!("pipeline stage `{stage}` must lower to an Arrow");
        };
        match body {
            ArrowBody::ExternalRealization(_) => {}
            ArrowBody::Unparsed(_) => panic!(
                "pipeline stage `{stage}` still has Unparsed body — \
                 the PipelineStageBinding bootstrap pass did not run \
                 or is broken. This is the DB-16 case-2 dissolution gate."
            ),
            other => panic!("pipeline stage `{stage}` has unexpected body {other:?}"),
        }
    }
}
