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
// Split these sentinel strings so the ratchet does not match its own source.
const FORBIDDEN_PIPELINE_DAG_MACRO_NAME: &str = concat!("include", "_str");
const FORBIDDEN_PIPELINE_DAG_PATH: &str = concat!("pipeline", ".dag");

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

fn code_position_mask(text: &str) -> Vec<bool> {
    let bytes = text.as_bytes();
    let mut mask = vec![true; bytes.len()];
    let mut idx = 0;

    while idx < bytes.len() {
        match bytes[idx] {
            b'/' if bytes.get(idx + 1) == Some(&b'/') => {
                let start = idx;
                idx += 2;
                while idx < bytes.len() && bytes[idx] != b'\n' {
                    idx += 1;
                }
                mask[start..idx].fill(false);
            }
            b'/' if bytes.get(idx + 1) == Some(&b'*') => {
                let start = idx;
                idx += 2;
                let mut depth = 1usize;
                while idx < bytes.len() && depth > 0 {
                    if bytes[idx] == b'/' && bytes.get(idx + 1) == Some(&b'*') {
                        depth += 1;
                        idx += 2;
                    } else if bytes[idx] == b'*' && bytes.get(idx + 1) == Some(&b'/') {
                        depth -= 1;
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                mask[start..idx].fill(false);
            }
            b'b' | b'c' if bytes.get(idx + 1) == Some(&b'"') => {
                idx = mask_quoted_literal(bytes, &mut mask, idx, idx + 1);
            }
            b'b' if bytes.get(idx + 1) == Some(&b'r') => {
                if let Some(end) = raw_string_end(bytes, idx + 2) {
                    mask[idx..end].fill(false);
                    idx = end;
                } else {
                    idx += 1;
                }
            }
            b'r' => {
                if let Some(end) = raw_string_end(bytes, idx + 1) {
                    mask[idx..end].fill(false);
                    idx = end;
                } else {
                    idx += 1;
                }
            }
            b'"' => {
                idx = mask_quoted_literal(bytes, &mut mask, idx, idx);
            }
            b'\'' => {
                if apostrophe_starts_lifetime_or_label(bytes, idx) {
                    idx += 1;
                } else {
                    idx = mask_quoted_literal(bytes, &mut mask, idx, idx);
                }
            }
            _ => idx += 1,
        }
    }

    mask
}

fn apostrophe_starts_lifetime_or_label(bytes: &[u8], quote: usize) -> bool {
    let Some(next) = bytes.get(quote + 1).copied() else {
        return false;
    };
    if !(next == b'_' || next.is_ascii_alphabetic()) {
        return false;
    }
    let mut idx = quote + 2;
    while bytes.get(idx).is_some_and(|byte| is_rust_ident_byte(*byte)) {
        idx += 1;
    }
    bytes.get(idx) != Some(&b'\'')
}

fn raw_string_end(bytes: &[u8], mut idx: usize) -> Option<usize> {
    let hash_start = idx;
    while bytes.get(idx) == Some(&b'#') {
        idx += 1;
    }
    if bytes.get(idx) != Some(&b'"') {
        return None;
    }
    let hash_count = idx - hash_start;
    idx += 1;
    while idx < bytes.len() {
        if bytes[idx] == b'"' {
            let hashes_match =
                (0..hash_count).all(|offset| bytes.get(idx + 1 + offset) == Some(&b'#'));
            if hashes_match {
                return Some(idx + 1 + hash_count);
            }
        }
        idx += 1;
    }
    Some(bytes.len())
}

fn mask_quoted_literal(bytes: &[u8], mask: &mut [bool], start: usize, quote: usize) -> usize {
    let delimiter = bytes[quote];
    let mut idx = quote + 1;
    while idx < bytes.len() {
        if bytes[idx] == b'\\' {
            idx = (idx + 2).min(bytes.len());
        } else if bytes[idx] == delimiter {
            idx += 1;
            break;
        } else {
            idx += 1;
        }
    }
    mask[start..idx].fill(false);
    idx
}

fn is_rust_ident_byte(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}

fn skip_rust_trivia(bytes: &[u8], code_mask: &[bool], mut idx: usize) -> usize {
    while idx < bytes.len()
        && (bytes[idx].is_ascii_whitespace() || !code_mask.get(idx).copied().unwrap_or(false))
    {
        idx += 1;
    }
    idx
}

fn include_str_pipeline_dag_offenders(manifest_dir: &Path, path: &Path, text: &str) -> Vec<String> {
    let mut offenders = Vec::new();
    let code_mask = code_position_mask(text);
    let bytes = text.as_bytes();
    for (idx, _) in text.match_indices(FORBIDDEN_PIPELINE_DAG_MACRO_NAME) {
        if !code_mask.get(idx).copied().unwrap_or(false) {
            continue;
        }
        if idx > 0 && is_rust_ident_byte(bytes[idx - 1]) {
            continue;
        }
        let after_macro_name = idx + FORBIDDEN_PIPELINE_DAG_MACRO_NAME.len();
        if bytes
            .get(after_macro_name)
            .is_some_and(|byte| is_rust_ident_byte(*byte))
        {
            continue;
        }
        let bang = skip_rust_trivia(bytes, &code_mask, after_macro_name);
        if bytes.get(bang) != Some(&b'!') {
            continue;
        }
        let line_start = text[..idx].rfind('\n').map_or(0, |pos| pos + 1);
        let line = &text[line_start..text[idx..].find('\n').map_or(text.len(), |pos| idx + pos)];
        let trimmed = line.trim_start();

        let after_bang = bang + 1;
        let open = skip_rust_trivia(bytes, &code_mask, after_bang);
        let Some((open_delimiter, close_delimiter)) = macro_delimiters(bytes.get(open).copied())
        else {
            continue;
        };
        let mut depth = 0usize;
        let mut close = None;
        for (pos, ch) in text[open..].char_indices() {
            let absolute_pos = open + pos;
            if !code_mask.get(absolute_pos).copied().unwrap_or(false) {
                continue;
            }
            match ch {
                _ if ch == open_delimiter => depth += 1,
                _ if ch == close_delimiter => {
                    if depth == 0 {
                        break;
                    }
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
        let joined_literals = joined_rust_string_literals(macro_arg);
        if macro_arg.contains(FORBIDDEN_PIPELINE_DAG_PATH)
            || joined_literals.contains(FORBIDDEN_PIPELINE_DAG_PATH)
        {
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

fn joined_rust_string_literals(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut joined = String::new();
    let mut idx = 0usize;
    while idx < bytes.len() {
        if let Some((content_start, content_end, end)) = raw_string_literal_bounds(bytes, idx) {
            joined.push_str(&text[content_start..content_end]);
            idx = end;
            continue;
        }
        if bytes[idx] == b'"' {
            idx += 1;
            while idx < bytes.len() {
                match bytes[idx] {
                    b'\\' => {
                        if let Some(escaped) = bytes.get(idx + 1) {
                            joined.push(*escaped as char);
                            idx += 2;
                        } else {
                            idx += 1;
                        }
                    }
                    b'"' => {
                        idx += 1;
                        break;
                    }
                    byte => {
                        joined.push(byte as char);
                        idx += 1;
                    }
                }
            }
            continue;
        }
        if let Some((ch, end)) = char_literal_value(bytes, idx) {
            joined.push(ch);
            idx = end;
            continue;
        }
        idx += 1;
    }
    joined
}

fn char_literal_value(bytes: &[u8], start: usize) -> Option<(char, usize)> {
    if bytes.get(start) != Some(&b'\'') || apostrophe_starts_lifetime_or_label(bytes, start) {
        return None;
    }
    let content = start + 1;
    match bytes.get(content).copied()? {
        b'\\' => {
            let escaped = bytes.get(content + 1).copied()?;
            if bytes.get(content + 2) == Some(&b'\'') {
                Some((escaped as char, content + 3))
            } else {
                None
            }
        }
        byte if bytes.get(content + 1) == Some(&b'\'') => Some((byte as char, content + 2)),
        _ => None,
    }
}

fn raw_string_literal_bounds(bytes: &[u8], start: usize) -> Option<(usize, usize, usize)> {
    if bytes.get(start) != Some(&b'r') {
        return None;
    }
    if start > 0 && is_rust_ident_byte(bytes[start - 1]) {
        return None;
    }
    let mut quote = start + 1;
    while bytes.get(quote) == Some(&b'#') {
        quote += 1;
    }
    if bytes.get(quote) != Some(&b'"') {
        return None;
    }
    let hash_count = quote - start - 1;
    let content_start = quote + 1;
    let mut idx = content_start;
    while idx < bytes.len() {
        if bytes[idx] == b'"'
            && idx + 1 + hash_count <= bytes.len()
            && bytes[idx + 1..idx + 1 + hash_count]
                .iter()
                .all(|byte| *byte == b'#')
        {
            return Some((content_start, idx, idx + 1 + hash_count));
        }
        idx += 1;
    }
    None
}

fn macro_delimiters(open: Option<u8>) -> Option<(char, char)> {
    match open? {
        b'(' => Some(('(', ')')),
        b'[' => Some(('[', ']')),
        b'{' => Some(('{', '}')),
        _ => None,
    }
}

#[test]
fn compiler_sources_and_tests_have_no_include_str_pipeline_dag_authority() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect_rs_files(&manifest_dir.join("src"), &mut files);
    collect_rs_files(&manifest_dir.join("tests"), &mut files);
    files.sort();
    let mut offenders = Vec::new();
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
        format_args!("{FORBIDDEN_PIPELINE_DAG_MACRO_NAME}!"),
        FORBIDDEN_PIPELINE_DAG_PATH,
        offenders.join("\n")
    );
}

#[test]
fn include_str_pipeline_dag_ratchet_catches_split_literals() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("tests/integration/synthetic.rs");
    let synthetic = format!(
        r##"
const OK: &str = include{}("other.dag");
const BAD: &str = include{}(concat!("../../", "pipeline", ".dag"));
const ALSO_BAD: &str = include_str /* trivia */ ! (concat!("../../", "pipeline", ".dag"));
const BRACKET_BAD: &str = include_str![concat!("../../", "pipeline", ".dag")];
const BRACE_BAD: &str = include_str!{{concat!("../../", "pipeline", ".dag")}};
const RAW_BAD: &str = include_str!(concat!(r#"../../pipeline"#, r#".dag"#));
const CHAR_BAD: &str = include_str!(concat!("../../pipe", 'l', "ine.dag"));
"##,
        "_str!", "_str!"
    );
    let offenders = include_str_pipeline_dag_offenders(manifest_dir, &path, &synthetic);
    assert_eq!(offenders.len(), 6);
    assert!(offenders[0].contains("synthetic.rs:3:"));
    assert!(offenders[1].contains("synthetic.rs:4:"));
    assert!(offenders[2].contains("synthetic.rs:5:"));
    assert!(offenders[3].contains("synthetic.rs:6:"));
    assert!(offenders[4].contains("synthetic.rs:7:"));
    assert!(offenders[5].contains("synthetic.rs:8:"));
}

#[test]
fn include_str_pipeline_dag_ratchet_ignores_inert_text() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("tests/integration/inert.rs");
    let synthetic = format!(
        r##"
const STRING: &str = "include{}(\"pipeline.dag\")";
const RAW: &str = r#"include{}("pipeline.dag")"#;
// include{}("pipeline.dag")
/*
include{}("pipeline.dag")
*/
let _actual = include{}(concat!("../../", "pipeline", ".dag"));
"##,
        "_str!", "_str!", "_str!", "_str!", "_str!"
    );
    let offenders = include_str_pipeline_dag_offenders(manifest_dir, &path, &synthetic);
    assert_eq!(offenders.len(), 1);
    assert!(offenders[0].contains("inert.rs:8:"));
}

#[test]
fn include_str_pipeline_dag_ratchet_handles_lifetimes_before_offenders() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("tests/integration/lifetime.rs");
    let synthetic = format!(
        r#"
fn lifetime_marker<'a>(input: &'a str) -> &'a str {{
    input
}}
const BAD: &str = include{}(concat!("../../", "pipeline", ".dag"));
"#,
        "_str!"
    );
    let offenders = include_str_pipeline_dag_offenders(manifest_dir, &path, &synthetic);
    assert_eq!(offenders.len(), 1);
    assert!(offenders[0].contains("lifetime.rs:5:"));
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
