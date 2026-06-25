#![allow(clippy::disallowed_macros)]

//! Whole-tree typecheck ADVISORY reporter (north-star, gunbc#5760 + #5772).
//!
//! Resolves every `.dag` module under the given source roots in ONE pass under
//! `ResolveTypecheckGate::DiagnosticsCollector` (collect-and-continue) and reports
//! the full set of resolve/typecheck diagnostics — including the orphan-module
//! diagnostics a closure-scoped `gunbc compile` never surfaces (it resolves only
//! the transitive import closure, leaving the whole-tree-only diagnostics masked).
//!
//! ADVISORY ONLY: this binary always exits 0. It is the counting/classification
//! consumer of the `resolve_diagnostics` carrier; promotion to a BLOCKING gate is
//! deferred to the operator once the count is driven to ~0 (DESIGN §5/§6 — do not
//! flip a floor gate from closure-scope to whole-tree-scope unilaterally).
//!
//! Output: one TSV line per diagnostic on stdout — `module_name<TAB>file<TAB>message`
//! — so a downstream classifier can bucket each diagnostic's module against the
//! import-reference graph (DEAD rot / ENROLL-as-root / LEAVE-or-WIRE transitional).
//! A summary (modules resolved, excluded, diagnostic count) goes to stderr.

use std::process::ExitCode;

use v1_compiler::cli_run::{whole_tree_resolved_ctx, ResolveTypecheckGate, WholeTreeCtx};
use v1_compiler::v1_interpreter::ExecutionMode;
use v1_compiler::v1_std_core::{diagnostic_to_message, diagnostic_to_span};

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("whole_tree_typecheck_advisory: {} requires a value", flag);
            Err(ExitCode::from(2))
        }
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    // Intentionally-malformed scanner fixture inputs declare imports of nonexistent
    // modules and cannot be part of a whole-tree resolve. Excluded by default
    // (mirrors the wiring-liveness whole-tree gate); extendable via flag.
    let mut exclude_subpaths: Vec<String> = vec!["test/fixture/".to_string()];

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
                eprintln!("whole_tree_typecheck_advisory: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("whole_tree_typecheck_advisory: at least one --source-root is required");
        return Err(ExitCode::from(2));
    }

    let WholeTreeCtx {
        ctx: _,
        modules_resolved,
        modules_excluded,
        resolve_diagnostics,
    } = whole_tree_resolved_ctx(
        &source_roots,
        &exclude_subpaths,
        ExecutionMode::Wet,
        ResolveTypecheckGate::DiagnosticsCollector,
    )
    .map_err(|e| {
        eprintln!("whole_tree_typecheck_advisory: whole-tree resolve failed:\n{e}");
        ExitCode::from(2)
    })?;

    for d in &resolve_diagnostics {
        let span = diagnostic_to_span(d.diagnostic.clone());
        let msg = diagnostic_to_message(d.diagnostic.clone());
        println!("{}\t{}\t{}", d.module_name, span.file, msg);
    }

    eprintln!(
        "whole_tree_typecheck_advisory: ADVISORY — {} diagnostic(s) over {} module(s) resolved \
         ({} excluded by subpath: {:?})",
        resolve_diagnostics.len(),
        modules_resolved,
        modules_excluded,
        exclude_subpaths
    );

    // ADVISORY: never blocks. Promotion to blocking is the operator's call once
    // the count reaches ~0 (DESIGN §5/§6).
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
