// DB-16 — `FnExternalBody` / `ArrowBody::Unparsed` semantic reconciliation.
//
// Host-backed pipeline stages parse as `FnExternalBody` → `Unparsed`, then
// `materialize_pipeline_realizations` rewrites each bound stage to
// `ExternalRealization`. This test locks that case-2 path.

use v3_compiler::dag::{ArrowBody, Dag, TypeConnective};

#[test]
fn pipeline_stages_lower_to_external_realization_not_unparsed() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap DAG should be clean, got {:?}",
        dag.diagnostics()
    );

    // Order matches `compile` in pipeline.dag (stages with PipelineStageBinding
    // data — not `compile` itself, which has no separate binding).
    for stage in [
        "parse",
        "lower",
        "infer",
        "compute_ownership",
        "lens_complexity",
        "emit",
    ] {
        let decl = dag.declaration_by_name(stage).unwrap_or_else(|| {
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
