#![allow(clippy::disallowed_macros)]

//! SCAFFOLD — emit-only whole-corpus audit for the complexity/linearity lens family (SYNTACTIC half).
//!
//! ROADMAP §3 `3-gates-whole` ("complexity budget gates the whole codebase"); audit-first bridge
//! until whole-corpus asymptotic cost gate grounds. Parse-only walk over `witness_layer_roots`
//! using `decl_facts(roots)`.
//!
//! NOT floor-enrolled. Prints `site`, `lens`, `rule`, `triage` (TSV). Exit 0 unless
//! `--fail-on-findings` (for discriminating tests).
//!
//! DISSOLUTION: fold SYNTACTIC projections into a pure `.dag` Node-tree reader, then enroll floor.

use std::process::ExitCode;

use v1_compiler::cli_run::{
    complexity_linearity_audit_corpus_default_roots, complexity_linearity_audit_corpus_parse_only,
    complexity_linearity_wildcard_facts, non_fold_residue_count, non_fold_residue_unrostered_count,
    ComplexityLinearityAuditSummary,
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
        return print_summary(
            &complexity_linearity_audit_corpus_default_roots(),
            fail_on_findings,
        );
    }

    print_summary(
        &complexity_linearity_audit_corpus_parse_only(&source_roots),
        fail_on_findings,
    )
}

fn print_summary(
    summary: &ComplexityLinearityAuditSummary,
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
         non_fold_residue total={} unrostered={}, inert_carrier inert={}",
        non_fold_residue_count(),
        non_fold_residue_unrostered_count(),
        v1_compiler::cli_run::inert_carrier_names_live().len()
    );
    let wildcard_facts = complexity_linearity_wildcard_facts();
    let on_roster = wildcard_facts.iter().filter(|f| f.rostered).count();
    eprintln!(
        "complexity_linearity_audit: syntactic wildcard arms — total={} on_roster={} off_roster={} \
         | triage buckets are grounded in v2.lens.complexity_linearity_audit (.dag), verified by \
         src/v2/test/claim/long/syntactic_audit_witness_test.dag",
        wildcard_facts.len(),
        on_roster,
        wildcard_facts.len() - on_roster,
    );
    let cost_findings = summary.findings.iter().filter(|f| f.lens == "cost").count();
    eprintln!(
        "complexity_linearity_audit: cost syntactic proxy (Node.body walk) — \
         syntactic_high_match_fanout={} site(s) (full cost_lens on DeclFact.node.body pending whole-corpus gate)",
        cost_findings
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
