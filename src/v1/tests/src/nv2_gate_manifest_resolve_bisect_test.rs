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
