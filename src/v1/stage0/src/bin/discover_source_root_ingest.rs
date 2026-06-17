//! Whole-tree `.dag` source read discovery for Stage C Lane 3a SourceRootIngest.
//!
//! Host boundary: filesystem walk + per-file read (fail-closed) + expose neutral
//! `DagSourceReadWitness` facts as a `SourceRootIngest` monoid manifest.
//!
//! Usage:
//!   discover_source_root_ingest --source-root src/v2 \
//!       [--scan-dir src/v2/test/fixture/program_assembly] \
//!       [--exclude-subpath host_source_root_ingest_manifest.dag] \
//!       [--emit-dag-manifest target/v2-source-root-ingest-manifest.dag]
//!
//! Exit codes: 0 = success; 1 = discovery failure; 2 = usage error.

#![allow(clippy::disallowed_macros)]

use std::path::PathBuf;
use std::process::ExitCode;

use v1_compiler::cli_run::{
    discover_source_root_reads, emit_source_root_ingest_manifest, source_root_ingest_content_hash_fnv1a64,
};

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("discover_source_root_ingest: {} requires a value", flag);
            Err(ExitCode::from(2))
        }
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut scan_dir = "src/v2/test/fixture/program_assembly".to_string();
    let mut exclude_subpaths: Vec<String> = vec![
        "host_source_root_ingest_manifest.dag".to_string(),
    ];
    let mut manifest_path: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
            }
            "--scan-dir" => {
                i += 1;
                scan_dir = require_value(&args, i, "--scan-dir")?;
            }
            "--exclude-subpath" => {
                i += 1;
                exclude_subpaths.push(require_value(&args, i, "--exclude-subpath")?);
            }
            "--emit-dag-manifest" => {
                i += 1;
                manifest_path = Some(PathBuf::from(require_value(
                    &args,
                    i,
                    "--emit-dag-manifest",
                )?));
            }
            other => {
                eprintln!("discover_source_root_ingest: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("discover_source_root_ingest: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }

    let records = match discover_source_root_reads(&source_roots, &scan_dir, &exclude_subpaths) {
        Ok(records) => records,
        Err(msg) => {
            eprintln!("discover_source_root_ingest: {}", msg);
            return Err(ExitCode::from(1));
        }
    };
    eprintln!(
        "discover_source_root_ingest: {} source read(s), content_hash={}",
        records.len(),
        source_root_ingest_content_hash_fnv1a64(&records)
    );

    if let Some(path) = manifest_path {
        if let Err(msg) = emit_source_root_ingest_manifest(&path, &records) {
            eprintln!("discover_source_root_ingest: {}", msg);
            return Err(ExitCode::from(1));
        }
        eprintln!(
            "discover_source_root_ingest: wrote manifest {} ({} read witness(es))",
            path.display(),
            records.len()
        );
    }

    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
