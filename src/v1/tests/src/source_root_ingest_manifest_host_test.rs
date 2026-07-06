use std::fs;
use std::path::PathBuf;

use v1_compiler::cli_run::{
    discover_source_root_reads_for_entry, emit_source_root_ingest_manifest,
    parse_source_root_entry_admission,
};

use crate::helpers::workspace_root;

const COMPILER_ENTRY: &str = "src/v2/compiler/00_compile.dag";

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("gunbc-{label}-{nanos}"))
}

#[test]
fn source_root_ingest_symbol_leading_digit_gets_sr_prefix() {
    assert_eq!(
        v1_compiler::cli_run::source_root_ingest_artifact_id_for_path(
            "src/v2/compiler/00_compile.dag"
        ),
        "^sr_00_compile"
    );
}

#[test]
fn manifest_entry_admission_qualified_name_is_well_formed() {
    let ws = workspace_root();
    let temp = unique_temp_dir("manifest-qn");
    fs::create_dir_all(&temp).expect("temp dir");
    let manifest_path = temp.join("manifest.dag");
    let entry_source = fs::read_to_string(ws.join(COMPILER_ENTRY)).expect("read entry");
    let admission = parse_source_root_entry_admission(&entry_source).expect("parse admission");
    let records = discover_source_root_reads_for_entry(
        &[ws.join("src/v2").to_string_lossy().to_string()],
        ws.join(COMPILER_ENTRY).to_str().expect("entry utf8"),
        &["host_source_root_ingest_manifest.dag".to_string()],
    )
    .expect("discover reads");
    emit_source_root_ingest_manifest(&manifest_path, &records[..1], Some(&admission))
        .expect("emit manifest");
    let manifest = fs::read_to_string(&manifest_path).expect("read manifest");
    assert!(
        manifest.contains("import v2.std.cross_tree.import_model { V2Tree"),
        "manifest must import grounded SourceRootRef variants:\n{manifest}"
    );
    assert!(
        manifest.contains("source_root: V2Tree"),
        "manifest witness rows must carry grounded source_root tags:\n{manifest}"
    );
    assert!(
        manifest.contains(
            "Cons { head: ^v2, tail: Cons { head: ^compiler, tail: Cons { head: ^compile, tail: Empty } } }"
        ),
        "manifest admission subject QN malformed:\n{manifest}"
    );
    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn parse_compiler_entry_admission_imports_compile_module() {
    let ws = workspace_root();
    let source = fs::read_to_string(ws.join(COMPILER_ENTRY)).expect("read entry");
    let admission = parse_source_root_entry_admission(&source).expect("parse admission");
    assert_eq!(admission.subject, vec!["v2", "compiler", "compile"]);
    assert!(
        admission
            .imports
            .iter()
            .any(|p| p == &["v2", "compiler", "emit"]),
        "expected compile module imports to include v2.compiler.emit: {:?}",
        admission.imports
    );
}
