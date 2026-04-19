use v3_compiler::dag::{ArrowBody, TypeConnective};
use v3_compiler::{
    compare_stage_snapshots, compile_stage_snapshots, default_fixed_point_source, parse_for_test,
    tokenize_for_test, Dag,
};

#[test]
fn pipeline_dag_parses() {
    let source = include_str!("../../pipeline.dag");
    let tokens = tokenize_for_test(source, "pipeline.dag")
        .unwrap_or_else(|diag| panic!("tokenize pipeline.dag failed: {diag:?}"));
    let _module = parse_for_test(&tokens, "pipeline.dag")
        .unwrap_or_else(|diag| panic!("parse pipeline.dag failed: {diag:?}"));
}

#[test]
fn bootstrap_loads_pipeline_stage_realizations() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "pipeline bootstrap should be clean, got {:?}",
        dag.diagnostics()
    );

    let meta = dag
        .declaration_by_name("CompilerHostRealization")
        .expect("pipeline realization meta present")
        .id;

    for (stage, realization) in [
        ("parse", "parse_realization"),
        ("lower", "lower_realization"),
        ("infer", "infer_realization"),
        ("compute_ownership", "compute_ownership_realization"),
        ("emit", "emit_realization"),
        ("lens_complexity", "lens_complexity_realization"),
    ] {
        let realization_id = dag
            .declaration_by_name(realization)
            .unwrap_or_else(|| panic!("realization `{realization}` present"))
            .id;
        let realization_decl = dag.declaration(realization_id);
        assert_eq!(
            realization_decl.meta_tag,
            Some(meta),
            "`{realization}` should point at CompilerHostRealization"
        );
        assert!(
            matches!(realization_decl.connective, TypeConnective::Conj { .. }),
            "`{realization}` should be materialized as a realization-shaped Conj"
        );

        let stage_decl = dag
            .declaration_by_name(stage)
            .unwrap_or_else(|| panic!("stage `{stage}` present"));
        match &stage_decl.connective {
            TypeConnective::Arrow { body, .. } => assert!(
                matches!(body, ArrowBody::ExternalRealization(id) if *id == realization_id),
                "`{stage}` should point at `{realization}` via ExternalRealization"
            ),
            other => panic!("stage `{stage}` should lower to Arrow, got {other:?}"),
        }
    }
}

#[test]
fn serialize_dag_is_deterministic() {
    let snapshots_a = compile_stage_snapshots(default_fixed_point_source(), "fixed_point_input.v3")
        .expect("snapshots compile");
    let snapshots_b = compile_stage_snapshots(default_fixed_point_source(), "fixed_point_input.v3")
        .expect("snapshots compile");
    assert!(
        snapshots_a
            .iter()
            .zip(snapshots_b.iter())
            .all(|(left, right)| left.bytes == right.bytes),
        "per-stage snapshots must be byte-stable across identical compiles"
    );
}

#[test]
fn snapshots_include_declared_lens_boundary() {
    let snapshots = compile_stage_snapshots(default_fixed_point_source(), "fixed_point_input.v3")
        .expect("snapshots compile");
    let stages: Vec<_> = snapshots
        .iter()
        .map(|snapshot| snapshot.stage.as_str())
        .collect();
    assert_eq!(
        stages,
        vec![
            "parse",
            "lower",
            "infer",
            "compute_ownership",
            "lens_complexity",
            "emit",
        ]
    );
}

#[test]
fn synthetic_divergence_names_stage() {
    let pass1 = compile_stage_snapshots(default_fixed_point_source(), "fixed_point_input.v3")
        .expect("pass1 compiles");
    let mut pass2 = compile_stage_snapshots(default_fixed_point_source(), "fixed_point_input.v3")
        .expect("pass2 compiles");

    let stage = pass2
        .iter_mut()
        .find(|snapshot| snapshot.stage == "infer")
        .expect("infer stage present");
    stage.bytes.extend_from_slice(b"\n# synthetic divergence\n");
    stage.dag = None;

    let mismatch = compare_stage_snapshots(&pass1, &pass2)
        .expect_err("synthetic divergence should be reported");
    assert_eq!(mismatch.stage, "infer");
}
