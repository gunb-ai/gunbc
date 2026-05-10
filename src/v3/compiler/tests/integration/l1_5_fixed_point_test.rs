//! **Layer:** integration (TESTING.md § test layers — multi-stage
//! pipeline fixed-point convergence).

use v3_compiler::dag::{ArrowBody, TypeConnective};
use v3_compiler::{
    compare_stage_snapshots, compile_stage_snapshots, default_fixed_point_source,
    generated_full_bootstrap_dag, Dag,
};

fn visit_rust_sources(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|err| {
        panic!(
            "failed to read compiler source directory `{}`: {err}",
            dir.display()
        )
    }) {
        let path = entry.expect("read_dir entry").path();
        if path.is_dir() {
            visit_rust_sources(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn has_pipeline_dag_include_str(source: &str) -> bool {
    let include_macro = concat!("include_", "str!");
    let mut offset = 0;
    while let Some(found) = source[offset..].find(include_macro) {
        let start = offset + found + include_macro.len();
        let Some(close) = source[start..].find(')') else {
            return false;
        };
        if source[start..start + close].contains("pipeline.dag") {
            return true;
        }
        offset = start + close + 1;
    }
    false
}

#[test]
fn compiler_has_no_pipeline_dag_include_str_side_channel() {
    let compiler_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut rust_sources = Vec::new();
    visit_rust_sources(&compiler_root, &mut rust_sources);

    let offenders: Vec<_> = rust_sources
        .into_iter()
        .filter(|path| {
            let source = std::fs::read_to_string(path)
                .unwrap_or_else(|err| panic!("failed to read `{}`: {err}", path.display()));
            has_pipeline_dag_include_str(&source)
        })
        .collect();
    assert!(
        offenders.is_empty(),
        "R3 bridge_include_str_side_channels_retired forbids {} of {} under \
         src/v3/compiler; offenders: {offenders:?}",
        concat!("include_", "str!"),
        "pipeline.dag"
    );
}

#[test]
fn pipeline_dag_is_present_in_structural_bootstrap_witness() {
    let dag = generated_full_bootstrap_dag();
    for name in [
        "PipelineStageBinding",
        "parse_stage_binding",
        "lower_stage_binding",
        "infer_stage_binding",
        "compute_ownership_stage_binding",
        "lens_complexity_stage_binding",
        "emit_stage_binding",
    ] {
        let decl = dag
            .declaration_by_name(name)
            .unwrap_or_else(|| panic!("`{name}` missing from structural bootstrap witness"));
        assert_eq!(
            decl.span.file, "src/v3/compiler/pipeline.dag",
            "`{name}` should be sourced from the generated pipeline.dag witness"
        );
    }
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
    compare_stage_snapshots(&snapshots_a, &snapshots_b)
        .expect("per-stage snapshots must be byte-stable across identical compiles");
}

#[test]
fn fixed_point_emit_stage_is_byte_stable() {
    let snapshots_a = compile_stage_snapshots(default_fixed_point_source(), "fixed_point_input.v3")
        .expect("pass1 compiles");
    let snapshots_b = compile_stage_snapshots(default_fixed_point_source(), "fixed_point_input.v3")
        .expect("pass2 compiles");

    let emit_a = snapshots_a
        .iter()
        .find(|snapshot| snapshot.stage == "emit")
        .expect("emit stage present in pass1");
    let emit_b = snapshots_b
        .iter()
        .find(|snapshot| snapshot.stage == "emit")
        .expect("emit stage present in pass2");
    assert_eq!(
        emit_a.bytes, emit_b.bytes,
        "emit-stage fixed-point bytes should stay stable while bootstrap moves"
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
