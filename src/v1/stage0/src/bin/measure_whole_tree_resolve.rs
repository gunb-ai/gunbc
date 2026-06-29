#![allow(clippy::disallowed_macros)]

//! Phase-0 whole-tree resolve RSS probe (calm-ram-408 / stern-moth-225).
//!
//! Strict-resolves every `.dag` module under the given source roots that passes
//! the floor discovery exclude list plus `--exclude-subpath` filters, in ONE
//! `whole_tree_resolved_ctx` pass — the width-1 all-modules-live worst case
//! from representation-minimization.md.

use std::process::ExitCode;

use v1_compiler::cli_run::{
    peak_rss_vhwm_bytes, whole_tree_resolved_ctx, WholeTreeCtx, FLOOR_DISCOVERY_EXCLUDES,
};
use v1_compiler::v1_interpreter::ExecutionMode;

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("measure_whole_tree_resolve: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut exclude_subpaths: Vec<String> = FLOOR_DISCOVERY_EXCLUDES
        .iter()
        .map(|sub| (*sub).to_string())
        .collect();
    // Probe-specific extras beyond `FLOOR_DISCOVERY_EXCLUDES` (whole-tree resolve
    // cannot strict-resolve test trees or eval-only workflow scaffolds).
    exclude_subpaths.extend([
        "test/fixture/".to_string(),
        "/test/".to_string(),
        "nat_semiring_rung".to_string(),
        "lens/application/empty_required_lenses_skip_gate.dag".to_string(),
        "lens/application/rejecting_lens_blocks_before_compile.dag".to_string(),
    ]);

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
            }
            "--exclude-subpath" => {
                i += 1;
                exclude_subpaths.push(require_value(&args, i, "--exclude-subpath")?);
            }
            other => {
                eprintln!("measure_whole_tree_resolve: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("measure_whole_tree_resolve: at least one --source-root is required");
        return Err(ExitCode::from(2));
    }

    let WholeTreeCtx {
        modules_resolved,
        modules_excluded,
        ..
    } = whole_tree_resolved_ctx(&source_roots, &exclude_subpaths, ExecutionMode::Wet).map_err(
        |e| {
            eprintln!("measure_whole_tree_resolve: whole-tree resolve failed:\n{e}");
            ExitCode::from(2)
        },
    )?;

    eprintln!(
        "measure_whole_tree_resolve: resolved {modules_resolved} module(s) over {} source root(s) \
         ({modules_excluded} excluded)",
        source_roots.len(),
    );

    match peak_rss_vhwm_bytes() {
        Some(bytes) => eprintln!(
            "[measurement] whole-tree resolve peak RSS: {bytes} bytes (VmHWM) modules={modules_resolved}"
        ),
        None => eprintln!(
            "[measurement] whole-tree resolve peak RSS: unavailable (no /proc/self/status) modules={modules_resolved}"
        ),
    }

    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
