#![allow(clippy::disallowed_macros)]

//! SCAFFOLD — emit-only whole-corpus audit for the complexity/linearity lens family (SYNTACTIC half).
//!
//! ROADMAP §3 `3-gates-whole` ("complexity budget gates the whole codebase"); audit-first bridge
//! until `decl_facts(roots)` host builtin (#5966) grounds fn-body reflection. Parse-only walk over
//! `witness_layer_roots` using `decl_facts_parse_only` stub.
//!
//! NOT floor-enrolled. Prints `site`, `lens`, `rule`, `triage` (TSV). Exit 0 unless
//! `--fail-on-findings` (for discriminating tests).
//!
//! DISSOLUTION: swap stub for `decl_facts(roots)`, move triage roster on-carrier, fold SYNTACTIC
//! projections into a pure `.dag` reader (gunbc#5364), then enroll floor gate.

use std::process::ExitCode;

use v1_compiler::cli_run::{non_fold_residue_count, non_fold_residue_unrostered_count};
use v1_compiler::complexity_linearity_audit_project::{
    audit_corpus_default_roots, audit_corpus_parse_only, roster_fiction_report,
};
use v1_compiler::module_path_index::inert_carrier_census::{
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
    let fiction = roster_fiction_report(summary);
    eprintln!(
        "complexity_linearity_audit: ROSTER-FICTION — (a) MIGRATION-DEBT: {}/{} live on roster \
         (floor RED on {} if migration roster fiction dropped); \
         (b) IRREDUCIBLE: {}/{} honest permanent residue (operator-signed); \
         unrostered today: {}",
        fiction.migration_debt_live,
        fiction.migration_debt_roster_slots,
        fiction.floor_red_if_migration_roster_fiction_dropped,
        fiction.irreducible_live,
        fiction.irreducible_roster_slots,
        fiction.resolved_unrostered_sites
    );
    eprintln!(
        "complexity_linearity_audit: syntactic wildcard arms — total={} on_roster={} off_roster={} \
         | triage: eval-interpreter-debt={} grammar-ladder-debt={} kernel-permanent={} \
         migration-debt={} closed-coproduct-debt={} open-domain={} triage-pending={}",
        fiction.syntactic_wildcard_total,
        fiction.syntactic_wildcard_on_roster,
        fiction.syntactic_wildcard_off_roster,
        fiction.eval_interpreter_debt,
        fiction.grammar_ladder_debt,
        fiction.kernel_permanent,
        fiction.migration_debt_tagged,
        fiction.closed_coproduct_debt,
        fiction.open_domain,
        fiction.triage_pending
    );
    let cost_findings = summary.findings.iter().filter(|f| f.lens == "cost").count();
    eprintln!(
        "complexity_linearity_audit: cost syntactic proxy (Node.body walk) — \
         syntactic_high_match_fanout={} site(s) (full cost_lens on DeclFact.node.body pending #5364)",
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
