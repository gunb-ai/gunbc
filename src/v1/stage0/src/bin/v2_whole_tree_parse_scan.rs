#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;
use std::sync::Arc;

use v1_compiler::cli_run::{discover_source_root_reads, load_sources_for_entry};
use v1_compiler::v1_compiler_compile::compile_to_resolved;
use v1_compiler::v1_interpreter::{self, Value};

const PARSE_HARNESS_ENTRY: &str = "src/v2/test/claim/long/gap4_parse_tokens_remain_test.dag";
const PARSE_FN: &str = "parses";

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("v2_whole_tree_parse_scan: {} requires a value", flag);
            Err(ExitCode::from(2))
        }
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut scan_dir = "src/v2".to_string();
    let mut exclude_subpaths: Vec<String> =
        vec!["host_source_root_ingest_manifest.dag".to_string()];

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
            other => {
                eprintln!("v2_whole_tree_parse_scan: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("v2_whole_tree_parse_scan: at least one --source-root is required");
        return Err(ExitCode::from(2));
    }

    let records =
        discover_source_root_reads(&source_roots, &scan_dir, &exclude_subpaths).map_err(|e| {
            eprintln!("v2_whole_tree_parse_scan: discovery failed: {e}");
            ExitCode::from(1)
        })?;

    let sources = load_sources_for_entry(&source_roots, PARSE_HARNESS_ENTRY).map_err(|e| {
        eprintln!("v2_whole_tree_parse_scan: harness load failed: {e}");
        ExitCode::from(2)
    })?;
    let resolved = compile_to_resolved(Arc::new(sources.into()));
    let hard: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    if !hard.is_empty() || resolved.graph.is_none() {
        eprintln!(
            "v2_whole_tree_parse_scan: harness compile failed: {}",
            hard.join("\n")
        );
        return Err(ExitCode::from(2));
    }
    let graph = resolved.graph.clone().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(
        &graph,
        resolved.source_indices.clone(),
        v1_interpreter::ExecutionMode::Wet,
    );

    let mut ok = 0usize;
    for rec in &records {
        if ok > 0 && ok.is_multiple_of(25) {
            eprintln!(
                "v2_whole_tree_parse_scan: progress {ok}/{} ...",
                records.len()
            );
        }
        let args = [(Some("src".to_string()), Value::Str(rec.source.clone()))];
        match v1_interpreter::run_in_context_with_args(&ctx, PARSE_FN, &args, false) {
            Ok(Value::Bool(true)) => ok += 1,
            Ok(Value::Bool(false)) => {
                eprintln!(
                    "v2_whole_tree_parse_scan: parse reject: {} (module {})",
                    rec.file_path, rec.module_path
                );
                return Ok(ExitCode::from(1));
            }
            Ok(other) => {
                eprintln!(
                    "v2_whole_tree_parse_scan: unexpected result from {}: {:?}",
                    rec.file_path, other
                );
                return Ok(ExitCode::from(1));
            }
            Err(e) => {
                eprintln!(
                    "v2_whole_tree_parse_scan: interpreter error on {}: {e}",
                    rec.file_path
                );
                return Ok(ExitCode::from(1));
            }
        }
    }

    eprintln!(
        "v2_whole_tree_parse_scan: {} file(s) parsed with zero rejects",
        ok
    );
    Ok(ExitCode::from(0))
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
