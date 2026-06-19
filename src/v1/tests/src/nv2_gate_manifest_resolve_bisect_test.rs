//! Manifest-overlay resolve timing for full N_v2 gate entry.
//! dissolve-on: #5146-class N_v2 resolve hang root-caused — delete with nv2_gate_resolve_bisect_* bisect harness.

use std::fs;
use std::time::Instant;

use v1_compiler::cli_run::{
    discover_source_root_reads_for_entry, emit_source_root_ingest_manifest,
    parse_source_root_entry_admission, resolve_entry_graph, SOURCE_ROOT_INGEST_INLINE_MAX,
};

use crate::helpers::workspace_root;

const NV2_GATE_ENTRY: &str = "src/v2/compiler/self_host/compiler_closure_emit_from_ingest_gate.dag";
const COMPILE_ENTRY: &str = "src/v2/compiler/00_compile.dag";

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
fn nv2_gate_resolve_with_transport_sidecar_manifest() {
    let ws = workspace_root();
    let temp = std::env::temp_dir().join(format!("gunbc-nv2-transport-{}", std::process::id()));
    fs::create_dir_all(&temp).expect("temp");
    let manifest_path = temp.join("host_source_root_ingest_manifest.dag");
    let entry = ws.join(COMPILE_ENTRY);
    let roots = vec![ws.join("src/v2").to_string_lossy().to_string()];
    let records = discover_source_root_reads_for_entry(
        &roots,
        entry.to_str().expect("entry"),
        &["host_source_root_ingest_manifest.dag".to_string()],
    )
    .expect("discover");
    assert!(
        records.len() > SOURCE_ROOT_INGEST_INLINE_MAX,
        "compiler closure must exceed inline cap to exercise transport sidecar"
    );
    let admission =
        parse_source_root_entry_admission(&fs::read_to_string(&entry).expect("read entry"))
            .expect("admission");
    emit_source_root_ingest_manifest(&manifest_path, &records, Some(&admission)).expect("emit");
    let manifest = fs::read_to_string(&manifest_path).expect("read manifest");
    assert!(
        manifest.contains("host_source_root_ingest: SourceRootIngest = Empty"),
        "large closure must omit inline ingest"
    );
    assert!(
        temp.join("host_source_root_ingest_transport.tsv").is_file(),
        "transport TSV must be emitted beside manifest"
    );
    let gate = ws.join(NV2_GATE_ENTRY);
    let overlay_roots = vec![
        ws.join("src/v2").to_string_lossy().to_string(),
        temp.to_string_lossy().to_string(),
    ];
    eprintln!(
        "gate resolve with transport manifest ({} modules): starting...",
        records.len()
    );
    let start = Instant::now();
    let result = resolve_entry_graph(&overlay_roots, gate.to_str().expect("gate"));
    let elapsed = start.elapsed();
    eprintln!(
        "gate resolve with transport manifest: {:?} in {:?}",
        result.as_ref().map(|_| "Ok").map_err(|e| e.as_str()),
        elapsed
    );
    assert!(result.is_ok(), "transport manifest resolve must succeed");
    assert!(
        elapsed.as_secs() < 300,
        "transport manifest resolve must finish under 300s (got {elapsed:?}) — #5146-class hang is sidecar+Empty, not sub-120s"
    );
    let _ = fs::remove_dir_all(&temp);
}
