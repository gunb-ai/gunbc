#![allow(clippy::disallowed_macros)]

//! Emit-only whole-corpus audit for the complexity/linearity lens family (SYNTACTIC half).
//!
//! Parse-only walk over `witness_layer_roots` using `parse_dag_file` — swaps to
//! `decl_facts(roots)` when the standalone base PR merges (#5966).
//!
//! Prints `site`, `lens`, `rule`, `triage` (TSV). Exit 0 always unless `--fail-on-findings`
//! (for discriminating tests). NOT floor-enrolled.

use std::process::ExitCode;

use v1_compiler::complexity_linearity_audit_project::{
    audit_corpus_default_roots, audit_corpus_parse_only,
};
use v1_compiler::non_fold_residue_project::{
    non_fold_residue_count, non_fold_residue_unrostered_count,
};
use v1_compiler::inert_carrier_project::{
    inert_carrier_count, inert_carrier_unrostered_count,
};

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("complexity_linearity_audit: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut fail_on_findings = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
            }
            "--fail-on-findings" => fail_on_findings = true,
            other => {
                eprintln!("complexity_linearity_audit: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        return print_summary(&audit_corpus_default_roots(), fail_on_findings);
    }

    print_summary(&audit_corpus_parse_only(&source_roots), fail_on_findings)
}

fn print_summary(
    summary: &v1_compiler::complexity_linearity_audit_project::AuditSummary,
    fail_on_findings: bool,
) -> Result<ExitCode, ExitCode> {
    eprintln!(
        "complexity_linearity_audit: scanned {} file(s), parsed {}, {} fn(s), {} syntactic finding(s)",
        summary.files_scanned,
        summary.files_parsed,
        summary.fns_scanned,
        summary.findings.len()
    );
    eprintln!(
        "complexity_linearity_audit: resolved-half roster proxies (not whole-corpus): \
         non_fold_residue total={} unrostered={}, inert_carrier total={} unrostered={}",
        non_fold_residue_count(),
        non_fold_residue_unrostered_count(),
        inert_carrier_count(),
        inert_carrier_unrostered_count()
    );
    eprintln!("site\tlens\trule\ttriage");
    for f in &summary.findings {
        println!("{}\t{}\t{}\t{}", f.site, f.lens, f.rule, f.triage);
    }

    if fail_on_findings && !summary.findings.is_empty() {
        return Ok(ExitCode::from(1));
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
