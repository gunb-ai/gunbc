//! **Layer:** integration (TESTING.md § test layers — multi-stage
//! pipeline fixed-point convergence).

use std::fs;
use std::path::{Path, PathBuf};

use v3_compiler::dag::{ArrowBody, TypeConnective};
use v3_compiler::{
    compare_stage_snapshots, compile_stage_snapshots, default_fixed_point_source, Dag,
};

/// Matches `pipeline_authority::PIPELINE_AUTHORITY_FILE` — integration tests cannot import `pub(crate)` helpers.
const PIPELINE_AUTHORITY_FILE: &str = "src/v3/compiler/pipeline.dag";

#[test]
fn pipeline_dag_bootstrap_authority_is_loaded_structurally() {
    let dag = Dag::new();
    let pipeline_types = [
        "PipelineStageBinding",
        "PipelineSnapshotKind",
        "CompilerHostRealization",
    ];
    for name in pipeline_types {
        let decl = dag
            .declaration_by_name(name)
            .unwrap_or_else(|| panic!("pipeline authority declaration `{name}` present"));
        assert_eq!(
            decl.span.file.as_str(),
            PIPELINE_AUTHORITY_FILE,
            "`{name}` must come from the bootstrapped pipeline authority"
        );
    }
}

/// Ratchet for `bridge_include_str_side_channels_retired` (pipeline slice): library sources under
/// `src/v3/compiler/src/` and compiler tests must not embed `pipeline.dag` via `include_str!` —
/// ordering authority stays structural (`PipelineStageBinding` / bootstrap witness).
fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {} failed: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("read_dir entry failed: {e}"));
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn include_str_pipeline_dag_offenders(manifest_dir: &Path, path: &Path, text: &str) -> Vec<String> {
    let forbidden_macro = concat!("include", "_str!");
    let forbidden_path = concat!("pipeline", ".dag");
    let mut offenders = Vec::new();
    for (idx, _) in text.match_indices(forbidden_macro) {
        let line_start = text[..idx].rfind('\n').map_or(0, |pos| pos + 1);
        let line = &text[line_start..text[idx..].find('\n').map_or(text.len(), |pos| idx + pos)];
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }

        let after_macro = idx + forbidden_macro.len();
        let Some(open_offset) = text[after_macro..].find('(') else {
            continue;
        };
        if !text[after_macro..after_macro + open_offset]
            .chars()
            .all(char::is_whitespace)
        {
            continue;
        }
        let open = after_macro + open_offset;
        let mut depth = 0usize;
        let mut close = None;
        for (pos, ch) in text[open..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(open + pos);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(close) = close else {
            continue;
        };

        let macro_arg = &text[open + 1..close];
        let joined_literals: String = macro_arg
            .split('"')
            .skip(1)
            .step_by(2)
            .collect::<Vec<_>>()
            .join("");
        if macro_arg.contains(forbidden_path) || joined_literals.contains(forbidden_path) {
            offenders.push(format!(
                "{}:{}:{}",
                path.strip_prefix(manifest_dir).unwrap_or(path).display(),
                text[..idx].bytes().filter(|byte| *byte == b'\n').count() + 1,
                trimmed
            ));
        }
    }
    offenders
}

#[test]
fn compiler_sources_and_tests_have_no_include_str_pipeline_dag_authority() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect_rs_files(&manifest_dir.join("src"), &mut files);
    collect_rs_files(&manifest_dir.join("tests"), &mut files);
    files.sort();
    let mut offenders = Vec::new();
    let forbidden_macro = concat!("include", "_str!");
    let forbidden_path = concat!("pipeline", ".dag");
    for path in files {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {} failed: {e}", path.display()));
        offenders.extend(include_str_pipeline_dag_offenders(
            &manifest_dir,
            &path,
            &text,
        ));
    }
    assert!(
        offenders.is_empty(),
        "`src/**/*.rs` and `tests/**/*.rs` must not use {} on {} (see bridge_ledger \
         bridge_include_str_side_channels_retired + pipeline_authority.rs). Offenders:\n{}",
        forbidden_macro,
        forbidden_path,
        offenders.join("\n")
    );
}

#[test]
fn include_str_pipeline_dag_ratchet_catches_split_literals() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("tests/integration/synthetic.rs");
    let synthetic = format!(
        r#"
const OK: &str = include{}("other.dag");
const BAD: &str = include{}(concat!("../../", "pipeline", ".dag"));
"#,
        "_str!", "_str!"
    );
    let offenders = include_str_pipeline_dag_offenders(manifest_dir, &path, &synthetic);
    assert_eq!(offenders.len(), 1);
    assert!(offenders[0].contains("synthetic.rs:3:"));
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

/// Provenance ratchet: `PipelineStageBinding` rows must remain authored in `pipeline.dag` (same
/// stable path the bootstrap embed uses), not only parse/infer shape checks (`bootstrap_loads_pipeline_stage_realizations`).
#[test]
fn pipeline_stage_bindings_are_pipeline_dag_sourced() {
    let dag = Dag::new();
    let binding_type = dag
        .declaration_by_name("PipelineStageBinding")
        .expect("PipelineStageBinding type present in bootstrap");
    let mut bindings: Vec<_> = dag
        .declarations()
        .iter()
        .filter(|d| d.meta_tag == Some(binding_type.id))
        .collect();
    assert!(
        !bindings.is_empty(),
        "expected at least one PipelineStageBinding row from pipeline authority"
    );
    bindings.sort_by_key(|d| d.id.raw());
    for decl in bindings {
        assert_eq!(
            decl.span.file.as_str(),
            PIPELINE_AUTHORITY_FILE,
            "PipelineStageBinding `{}` must carry pipeline.dag provenance (got file {:?})",
            decl.name.as_deref().unwrap_or("<anonymous>"),
            decl.span.file
        );
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
