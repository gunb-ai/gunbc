#![allow(clippy::disallowed_macros)]

use std::path::PathBuf;
use std::process::ExitCode;

use v1_compiler::cli_run::{
    discover_owned_data_decls, emit_owned_data_manifest, owned_data_bool_witness_transport_tsv,
};

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("discover_owned_data: {} requires a value", flag);
            Err(ExitCode::from(2))
        }
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut scan_dir = "src/v2/test/claim".to_string();
    let mut exclude_subpaths: Vec<String> = vec![
        "impossible_bug".to_string(),
        "glob_discovery.dag".to_string(),
        "glob_discovery_law.dag".to_string(),
        "host_discovered_owned_data_manifest.dag".to_string(),
        "host_source_root_ingest_manifest.dag".to_string(),
        "unified_test_claim_substrate_equivalence.dag".to_string(),
    ];
    let mut format = "json".to_string();
    let mut manifest_path: Option<PathBuf> = None;
    let mut max_resolves: Option<usize> = None;

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
            "--format" => {
                i += 1;
                format = require_value(&args, i, "--format")?;
            }
            "--emit-dag-manifest" => {
                i += 1;
                manifest_path = Some(PathBuf::from(require_value(
                    &args,
                    i,
                    "--emit-dag-manifest",
                )?));
            }
            "--max-resolves" => {
                i += 1;
                let raw = require_value(&args, i, "--max-resolves")?;
                max_resolves = Some(raw.parse::<usize>().map_err(|_| {
                    eprintln!(
                        "discover_owned_data: --max-resolves expects a non-negative integer, got '{}'",
                        raw
                    );
                    ExitCode::from(2)
                })?);
            }
            other => {
                eprintln!("discover_owned_data: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("discover_owned_data: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }

    let discovery = match discover_owned_data_decls(&source_roots, &scan_dir, &exclude_subpaths) {
        Ok(discovery) => discovery,
        Err(msg) => {
            eprintln!("discover_owned_data: {}", msg);
            return Err(ExitCode::from(1));
        }
    };
    eprintln!(
        "discover_owned_data: {} graph resolve(s) covered {} entry file(s)",
        discovery.graph_resolves, discovery.entry_count
    );
    if let Some(ceiling) = max_resolves {
        if discovery.graph_resolves > ceiling {
            eprintln!(
                "discover_owned_data: {} graph resolve(s) exceed --max-resolves {} -- a top-level decl-name collision between entry closures forced a resolve split; rename the colliding decl (latency ratchet, not a budget to raise)",
                discovery.graph_resolves, ceiling
            );
            for collision in &discovery.group_split_collisions {
                eprintln!("discover_owned_data:   {}", collision);
            }
            return Err(ExitCode::from(1));
        }
    }
    let records = discovery.records;

    if let Some(path) = manifest_path {
        if let Err(msg) = emit_owned_data_manifest(&path, &records) {
            eprintln!("discover_owned_data: {}", msg);
            return Err(ExitCode::from(1));
        }
        eprintln!(
            "discover_owned_data: wrote manifest {} ({} owned decl record(s))",
            path.display(),
            records.len()
        );
    }

    match format.as_str() {
        "json" => match serde_json::to_string_pretty(&records) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("discover_owned_data: json encode failed: {}", e);
                return Err(ExitCode::from(1));
            }
        },
        "transport-tsv" => match owned_data_bool_witness_transport_tsv(&records) {
            Ok(tsv) => print!("{}", tsv),
            Err(msg) => {
                eprintln!("discover_owned_data: {}", msg);
                return Err(ExitCode::from(1));
            }
        },
        other => {
            eprintln!(
                "discover_owned_data: unsupported --format {} (expected json or transport-tsv)",
                other
            );
            return Err(ExitCode::from(2));
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
