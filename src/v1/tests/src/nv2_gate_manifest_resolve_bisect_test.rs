//! Manifest-overlay resolve timing for full N_v2 gate entry.
//! dissolve-on: #5146-class N_v2 resolve hang root-caused — delete with nv2_gate_resolve_bisect_* bisect harness.

use std::time::Instant;

use v1_compiler::cli_run::resolve_entry_graph;

use crate::helpers::workspace_root;

const NV2_GATE_ENTRY: &str = "src/v2/compiler/self_host/compiler_closure_emit_from_ingest_gate.dag";

#[test]
#[ignore]
fn nv2_gate_resolve_without_manifest_overlay() {
    let ws = workspace_root();
    let entry = ws.join(NV2_GATE_ENTRY);
    let roots = vec![ws.join("src/v2").to_string_lossy().to_string()];
    eprintln!("nv2 gate resolve (no manifest): starting...");
    let start = Instant::now();
    let result = resolve_entry_graph(&roots, entry.to_str().expect("entry utf8"));
    eprintln!(
        "nv2 gate resolve (no manifest): {:?} in {:?}",
        result.as_ref().map(|_| "Ok").map_err(|e| e.as_str()),
        start.elapsed()
    );
}

#[test]
#[ignore]
fn nv2_gate_resolve_with_manifest_overlay() {
    let ws = workspace_root();
    let manifest_dir = ws.join("target");
    let entry = ws.join(NV2_GATE_ENTRY);
    let roots = vec![
        ws.join("src/v2").to_string_lossy().to_string(),
        manifest_dir.to_string_lossy().to_string(),
    ];
    eprintln!("nv2 gate resolve (manifest overlay, 59-module ingest): starting...");
    let start = Instant::now();
    let result = resolve_entry_graph(&roots, entry.to_str().expect("entry utf8"));
    eprintln!(
        "nv2 gate resolve (manifest overlay): {:?} in {:?}",
        result.as_ref().map(|_| "Ok").map_err(|e| e.as_str()),
        start.elapsed()
    );
}

#[test]
#[ignore]
fn nv2_compile_closure_resolve_timing() {
    use std::time::Instant;
    let ws = workspace_root();
    let entry = ws.join("src/v2/compiler/00_compile.dag");
    let roots = vec![ws.join("src/v2").to_string_lossy().to_string()];
    eprintln!("00_compile resolve: starting...");
    let start = Instant::now();
    let result = resolve_entry_graph(&roots, entry.to_str().expect("entry utf8"));
    eprintln!(
        "00_compile resolve: {:?} in {:?}",
        result.as_ref().map(|_| "Ok").map_err(|e| e.as_str()),
        start.elapsed()
    );
}

#[test]
#[ignore]
fn nv2_gate_resolve_with_real_generated_manifest() {
    use std::time::Instant;
    use v1_compiler::cli_run::{
        discover_source_root_reads_for_entry, emit_source_root_ingest_manifest,
        parse_source_root_entry_admission,
    };
    use std::fs;
    let ws = workspace_root();
    let temp = std::env::temp_dir().join(format!(
        "gunbc-nv2-bisect-{}",
        std::process::id()
    ));
    fs::create_dir_all(&temp).expect("temp");
    let manifest_path = temp.join("host_source_root_ingest_manifest.dag");
    let entry = ws.join("src/v2/compiler/00_compile.dag");
    let roots = vec![ws.join("src/v2").to_string_lossy().to_string()];
    let records = discover_source_root_reads_for_entry(
        &roots,
        entry.to_str().expect("entry"),
        &["host_source_root_ingest_manifest.dag".to_string()],
    )
    .expect("discover");
    let admission = parse_source_root_entry_admission(
        &fs::read_to_string(&entry).expect("read entry"),
    )
    .expect("admission");
    emit_source_root_ingest_manifest(&manifest_path, &records, Some(&admission))
        .expect("emit");
    let gate = ws.join("src/v2/compiler/self_host/compiler_closure_emit_from_ingest_gate.dag");
    let overlay_roots = vec![
        ws.join("src/v2").to_string_lossy().to_string(),
        temp.to_string_lossy().to_string(),
    ];
    eprintln!(
        "gate resolve with real manifest ({} modules): starting...",
        records.len()
    );
    let start = Instant::now();
    let result = resolve_entry_graph(&overlay_roots, gate.to_str().expect("gate"));
    eprintln!(
        "gate resolve with real manifest: {:?} in {:?}",
        result.as_ref().map(|_| "Ok").map_err(|e| e.as_str()),
        start.elapsed()
    );
    let _ = fs::remove_dir_all(&temp);
}
