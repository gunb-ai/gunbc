#![allow(clippy::disallowed_macros)]

// CI2-0 Commit B sweep driver (operator mandate 2026-08-08; convergence mandate 2026-08-09):
// per roster entry, discover the import closure with the executor's OWN
// discover_source_root_reads_for_entry (never a second discovery), write the same
// host_source_root_ingest_manifest overlay the frontier probe survey rides, and evaluate the
// .dag-side sweep probe (src/v2/test/claim/long/realization_sweep_probe_entry.dag —
// realization_sweep_entry_receipt) so the canonical assembly, exactly-one identity join,
// infer, and translate all run IN the substrate. The host's role is transport + aggregation
// only: per-identity rows come back as TSV, and the phase-and-cause histogram is counted
// across entries. A host-side failure is a typed row (phase=host, located cause), never a
// skipped entry — the roster in equals the roster accounted.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use v1_compiler::cli_run::{
    discover_source_root_reads_for_entry, emit_source_ref_dag_for_path,
    emit_source_root_ingest_manifest, parse_source_root_entry_admission, resolve_entry_graph,
};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

const PROBE_ENTRY: &str = "src/v2/test/claim/long/realization_sweep_probe_entry.dag";
const PROBE_RECEIPT_FN: &str = "realization_sweep_entry_receipt";
const WITNESS_LAYER_ROOTS: &[&str] = &["dag", "src/v2"];
const DEFAULT_SURVEY_DIR: &str = "target/realization-sweep";

fn write_probe_overlay(
    entry_path: &str,
    ingest_manifest: &Path,
    entry_source_ref: &str,
) -> Result<(), String> {
    let escaped = entry_path.replace('\\', "/");
    let append = format!(
        "\ndata frontier_probe_entry_module_path: String = \"{escaped}\"\n\
         data frontier_probe_entry_source_ref: SourceRef = {entry_source_ref}\n"
    );
    let mut body =
        fs::read_to_string(ingest_manifest).map_err(|e| format!("read ingest manifest: {e}"))?;
    if body.contains("frontier_probe_entry_module_path") {
        return Ok(());
    }
    body.push_str(&append);
    fs::write(ingest_manifest, body).map_err(|e| format!("append entry overlay: {e}"))
}

fn overlay_dir_for(survey_dir: &Path, entry_path: &str) -> PathBuf {
    let slug = entry_path.replace('\\', "/").replace('/', "__");
    survey_dir.join(slug)
}

fn sweep_rows_for_entry(survey_dir: &Path, entry_path: &str) -> Result<String, String> {
    let exclude = vec!["host_source_root_ingest_manifest.dag".to_string()];
    let overlay_dir = overlay_dir_for(survey_dir, entry_path);
    fs::create_dir_all(&overlay_dir).map_err(|e| format!("mkdir overlay {overlay_dir:?}: {e}"))?;

    let discover_roots: Vec<String> = WITNESS_LAYER_ROOTS.iter().map(|r| r.to_string()).collect();
    let records = discover_source_root_reads_for_entry(&discover_roots, entry_path, &exclude)?;
    let entry_source =
        fs::read_to_string(entry_path).map_err(|e| format!("read entry {entry_path}: {e}"))?;
    let admission = parse_source_root_entry_admission(&entry_source)?;
    let ingest_manifest = overlay_dir.join("host_source_root_ingest_manifest.dag");
    emit_source_root_ingest_manifest(&ingest_manifest, &records, Some(&admission))?;
    let entry_source_ref = emit_source_ref_dag_for_path(&records, entry_path)?;
    write_probe_overlay(entry_path, &ingest_manifest, &entry_source_ref)?;

    let mut roots: Vec<String> = WITNESS_LAYER_ROOTS.iter().map(|r| r.to_string()).collect();
    roots.push(overlay_dir.to_string_lossy().into_owned());

    let (graph, source_indices) = resolve_entry_graph(&roots, PROBE_ENTRY)?;
    let ctx = InterpContext::new(&graph, source_indices, ExecutionMode::Hermetic);
    let value =
        v1_interpreter::run_in_context(&ctx, PROBE_RECEIPT_FN, true).map_err(|e| format!("{e}"))?;
    match value {
        Value::Str(tsv) => Ok(tsv),
        other => Err(format!(
            "receipt fn returned {}, expected String",
            other.type_label_public()
        )),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut entries: Vec<String> = Vec::new();
    let mut survey_dir = PathBuf::from(DEFAULT_SURVEY_DIR);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--entry" => {
                if i + 1 >= args.len() {
                    eprintln!("--entry requires a path");
                    return ExitCode::FAILURE;
                }
                entries.push(args[i + 1].clone());
                i += 2;
            }
            "--entries-file" => {
                if i + 1 >= args.len() {
                    eprintln!("--entries-file requires a path");
                    return ExitCode::FAILURE;
                }
                match fs::read_to_string(&args[i + 1]) {
                    Ok(body) => entries.extend(
                        body.lines()
                            .map(|l| l.trim().to_string())
                            .filter(|l| !l.is_empty() && !l.starts_with('#')),
                    ),
                    Err(e) => {
                        eprintln!("read entries file {}: {e}", args[i + 1]);
                        return ExitCode::FAILURE;
                    }
                }
                i += 2;
            }
            "--survey-dir" => {
                if i + 1 >= args.len() {
                    eprintln!("--survey-dir requires a path");
                    return ExitCode::FAILURE;
                }
                survey_dir = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            other => {
                eprintln!("unknown argument: {other}");
                return ExitCode::FAILURE;
            }
        }
    }
    if entries.is_empty() {
        eprintln!(
            "usage: realization_sweep_survey --entry <path> [--entry <path> ...] \
             [--entries-file <path>] [--survey-dir <dir>]"
        );
        return ExitCode::FAILURE;
    }

    let mut histogram: BTreeMap<(String, String), u64> = BTreeMap::new();
    let mut located: BTreeMap<String, u64> = BTreeMap::new();
    let mut identity_rows: u64 = 0;
    let mut host_error_rows: u64 = 0;
    for entry in &entries {
        match sweep_rows_for_entry(&survey_dir, entry) {
            Ok(tsv) => {
                for line in tsv.lines().filter(|l| !l.is_empty()) {
                    println!("[sweep-row] {line}");
                    let cols: Vec<&str> = line.split('\t').collect();
                    let phase = cols.get(2).unwrap_or(&"").to_string();
                    let cause = cols.get(3).unwrap_or(&"").to_string();
                    let at = cols.get(4).unwrap_or(&"").to_string();
                    if !at.is_empty() {
                        *located.entry(at).or_insert(0) += 1;
                    }
                    *histogram.entry((phase, cause)).or_insert(0) += 1;
                    identity_rows += 1;
                }
            }
            Err(e) => {
                // A host-side failure is a counted row, never a silently skipped entry.
                println!("[sweep-row] {entry}\t\thost\trealization_sweep_host_error");
                eprintln!("[sweep-host-error] {entry}: {e}");
                *histogram
                    .entry((
                        "host".to_string(),
                        "realization_sweep_host_error".to_string(),
                    ))
                    .or_insert(0) += 1;
                host_error_rows += 1;
            }
        }
    }

    println!("[sweep-summary] entries {} identity_rows {identity_rows} host_error_rows {host_error_rows}", entries.len());
    for ((phase, cause), count) in &histogram {
        println!("[sweep-histogram] {phase}\t{cause}\t{count}");
    }
    for (at, count) in &located {
        println!("[sweep-located] {at}\t{count}");
    }
    ExitCode::SUCCESS
}
