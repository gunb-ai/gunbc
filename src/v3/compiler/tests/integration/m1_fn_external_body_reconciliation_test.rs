// DB-16 — `FnExternalBody` / `ArrowBody::Unparsed` semantic reconciliation.
//
// Host-backed pipeline stages parse as `FnExternalBody` → `Unparsed`, then
// `materialize_pipeline_realizations` rewrites each bound stage to
// `ExternalRealization`. This test locks that case-2 path.

use v3_compiler::dag::{ArrowBody, Dag, TypeConnective};

/// Pipeline stage order is read structurally from `PipelineStageBinding`
/// declarations in the Dag — the `fn compile { ... }` body span is no
/// longer the ordering authority. This test locks that structural path by
/// matching the public stage-name list against the in-Dag iteration order
/// of stage bindings.
#[test]
fn pipeline_stage_order_is_read_structurally_from_bindings() {
    use v3_compiler::dag::{FieldValue, ValueBody};

    let dag = Dag::new();
    let binding_type_id = dag
        .declaration_by_name("PipelineStageBinding")
        .expect("PipelineStageBinding type must be bootstrapped")
        .id;

    let expected: Vec<String> = dag
        .declarations()
        .iter()
        .filter(|decl| decl.meta_tag == Some(binding_type_id))
        .map(|decl| {
            let Some(ValueBody::Structural { fields }) = &decl.value_body else {
                panic!("binding must have structural body");
            };
            let stage_id = fields
                .iter()
                .find(|(f, _)| f == "stage")
                .and_then(|(_, v)| match v {
                    FieldValue::Reference(id) => Some(*id),
                    _ => None,
                })
                .expect("stage field");
            dag.declaration(stage_id)
                .name
                .clone()
                .expect("stage has name")
        })
        .collect();

    let actual = v3_compiler::pipeline_compile_order_stage_names()
        .expect("pipeline stage order must resolve");

    assert_eq!(
        actual, expected,
        "stage order must be structurally read from PipelineStageBinding \
         declaration order"
    );
    assert!(
        !actual.is_empty(),
        "pipeline must have at least one bound stage"
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
