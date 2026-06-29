// Emit-only whole-corpus audit for the complexity/linearity lens family (SYNTACTIC half).
//
// Substrate: parse-only `medium_structure_project::parse_dag_file` over
// `witness_layer_roots` — the same walk `decl_facts(roots)` will supersede when it
// merges (#5966). DISSOLUTION: swap the file walk for `decl_facts(roots)`; keep the
// pure Node projections unchanged.
//
// RESOLVED-half findings (closed-coproduct-param non_fold, inert-carrier consumers)
// remain #5364-gated — reported via existing roster builtins, not whole-corpus here.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

use crate::cli_run::collect_dag_files_tolerant;
use crate::corpus_lex::{is_test_dag, repo_rel};
use crate::medium_structure_project::parse_dag_file;
use crate::module_path_index::witness_layer_roots;
use crate::v1_compiler_infer_items::{item_kind, ItemKind};
use crate::v1_std_core::{
    arm_pattern, authored_name_at, expr_var_name_at, match_arm_nodes, match_scrutinee, ExprData,
    MatchPattern, Node,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AuditFinding {
    pub site: String,
    pub lens: &'static str,
    pub rule: &'static str,
    pub triage: &'static str,
}

#[derive(Debug, Clone, Default)]
pub struct AuditSummary {
    pub files_scanned: usize,
    pub files_parsed: usize,
    pub fns_scanned: usize,
    pub findings: Vec<AuditFinding>,
}

fn rel_path(path: &Path) -> String {
    repo_rel(path)
}

fn corpus_dag_files(roots: &[String]) -> Vec<PathBuf> {
    let ws = crate::module_path_index::workspace_root();
    let mut files = Vec::new();
    for root in roots {
        let root_path = ws.join(root);
        if root_path.is_dir() {
            collect_dag_files_tolerant(&root_path, &mut files);
        }
    }
    files.sort();
    files
}

fn is_wildcard_arm(arm: &Rc<Node>) -> bool {
    matches!(arm_pattern(arm.clone()).as_ref(), MatchPattern::Wildcard)
}

fn walk_expr(
    node: &Rc<Node>,
    si: &Rc<std::collections::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    stats: &mut FnBodyStats,
) {
    stats.node_count += 1;
    if let ExprData::ExprMatch = node.expr_data.as_ref() {
        stats.match_count += 1;
        let scrutinee = match_scrutinee(node.clone());
        let scrutinee_name = expr_var_name_at(scrutinee, si.clone());
        let has_wildcard = match_arm_nodes(node.clone())
            .iter()
            .any(|arm| is_wildcard_arm(arm));
        if has_wildcard {
            stats.wildcard_matches += 1;
            if !scrutinee_name.is_empty() {
                stats.wildcard_scrutinee_names.insert(scrutinee_name);
            }
        }
    }
    for child in node.children.iter() {
        walk_expr(child, si, stats);
    }
}

#[derive(Default)]
struct FnBodyStats {
    node_count: usize,
    match_count: usize,
    wildcard_matches: usize,
    wildcard_scrutinee_names: BTreeSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RosterFictionReport {
    pub resolved_residue_sites: i64,
    pub resolved_unrostered_sites: i64,
    pub floor_red_if_roster_dropped: i64,
    pub syntactic_wildcard_total: i64,
    pub syntactic_wildcard_on_roster: i64,
    pub syntactic_wildcard_off_roster: i64,
    pub eval_interpreter_debt: i64,
    pub grammar_ladder_debt: i64,
    pub kernel_permanent: i64,
}

pub fn roster_fiction_report(summary: &AuditSummary) -> RosterFictionReport {
    let resolved_residue_sites = crate::non_fold_residue_project::non_fold_residue_count();
    let resolved_unrostered_sites =
        crate::non_fold_residue_project::non_fold_residue_unrostered_count();
    let mut report = RosterFictionReport {
        resolved_residue_sites,
        resolved_unrostered_sites,
        floor_red_if_roster_dropped: resolved_residue_sites,
        ..Default::default()
    };
    for f in &summary.findings {
        if f.rule != "syntactic_match_wildcard_arm" {
            continue;
        }
        report.syntactic_wildcard_total += 1;
        if crate::non_fold_residue_project::non_fold_residue_site_is_rostered(&f.site) {
            report.syntactic_wildcard_on_roster += 1;
        } else {
            report.syntactic_wildcard_off_roster += 1;
        }
        match f.triage {
            "eval-interpreter-debt" => report.eval_interpreter_debt += 1,
            "grammar-ladder-debt" => report.grammar_ladder_debt += 1,
            "kernel-permanent" => report.kernel_permanent += 1,
            _ => {}
        }
    }
    report
}

fn triage_wildcard(site: &str, fn_name: &str) -> &'static str {
    if fn_name.ends_with("_eq")
        || fn_name.contains("dominates")
        || fn_name.contains("lattice_join")
        || fn_name.contains("lattice_meet")
        || fn_name == "exit_ok"
        || fn_name.contains("_relation_eq")
        || fn_name.contains("_mode_eq")
    {
        return "kernel-permanent";
    }
    if matches!(
        fn_name,
        "eval_bind_node_eval"
            | "eval_branch_node_eval"
            | "eval_loop_node"
            | "eval_match_node_eval"
            | "eval_transform_node"
            | "eval_value_node"
    ) && site.contains("src/v2/compiler/05_eval.dag")
    {
        return "eval-interpreter-debt";
    }
    if site.contains("dag.dag::dag_grammar_terminal_for_mvp1_") {
        return "grammar-ladder-debt";
    }
    if site.contains("src/v2/compiler/")
        || site.contains("src/v2/extdeps/languages/")
        || site.contains("dsl/std/induction.dag")
    {
        return "real-debt";
    }
    "triage-pending"
}

fn triage_complexity(site: &str) -> &'static str {
    if site.contains("src/v2/compiler/") || site.contains("src/v2/std/compilers/") {
        "real-debt"
    } else if site.contains("dsl/gunbc/plans/") {
        "triage-pending"
    } else {
        "triage-pending"
    }
}

fn audit_function(
    rel: &str,
    fn_name: &str,
    body: &Rc<Node>,
    si: &Rc<std::collections::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Vec<AuditFinding> {
    let mut stats = FnBodyStats::default();
    walk_expr(body, si, &mut stats);
    let site = format!("{rel}::{fn_name}");
    let mut out = Vec::new();
    if stats.wildcard_matches > 0 {
        out.push(AuditFinding {
            site: site.clone(),
            lens: "non_fold_residue",
            rule: "syntactic_match_wildcard_arm",
            triage: triage_wildcard(&site, fn_name),
        });
    }
    if stats.match_count >= 8 || (stats.node_count >= 200 && stats.match_count >= 4) {
        out.push(AuditFinding {
            site,
            lens: "cost",
            rule: "syntactic_high_match_fanout",
            triage: triage_complexity(rel),
        });
    }
    out
}

pub fn audit_corpus_parse_only(roots: &[String]) -> AuditSummary {
    let mut summary = AuditSummary::default();
    for file in corpus_dag_files(roots) {
        let rel = rel_path(&file);
        if is_test_dag(&rel) {
            continue;
        }
        summary.files_scanned += 1;
        let Some(parsed) = parse_dag_file(&file) else {
            continue;
        };
        summary.files_parsed += 1;
        let si = parsed.source_indices;
        for item in parsed.items.iter() {
            if !matches!(
                item_kind(item.clone()),
                ItemKind::FnItem | ItemKind::FuncItem
            ) {
                continue;
            }
            let fn_name = authored_name_at(si.clone(), item.clone());
            if fn_name.is_empty() {
                continue;
            }
            let Some(body) = item.body.as_ref() else {
                continue;
            };
            summary.fns_scanned += 1;
            summary
                .findings
                .extend(audit_function(&rel, &fn_name, body, &si));
        }
    }
    summary.findings.sort();
    summary
}

pub fn audit_corpus_default_roots() -> AuditSummary {
    audit_corpus_parse_only(&witness_layer_roots())
}

fn cached_summary() -> &'static AuditSummary {
    static REPORT: OnceLock<AuditSummary> = OnceLock::new();
    REPORT.get_or_init(audit_corpus_default_roots)
}

pub fn complexity_linearity_syntactic_finding_count() -> i64 {
    cached_summary().findings.len() as i64
}

pub fn complexity_linearity_syntactic_wildcard_finding_count() -> i64 {
    cached_summary()
        .findings
        .iter()
        .filter(|f| f.rule == "syntactic_match_wildcard_arm")
        .count() as i64
}

pub fn complexity_linearity_syntactic_site_fired(site: &str) -> bool {
    cached_summary().findings.iter().any(|f| f.site == site)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_temp_module(content: &str) -> (String, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "complexity-linearity-audit-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("tempdir");
        let path = dir.join("audit_wildcard.dag");
        fs::write(&path, content).expect("write");
        (path.to_string_lossy().to_string(), dir)
    }

    #[test]
    fn syntactic_wildcard_finding_on_closed_coproduct_match() {
        let (path, _guard) = write_temp_module(
            "module audit_wildcard\n\
             type Mode = A | B | C\n\
             fn f(x: Mode) -> Bool {\n\
               match x {\n\
                 A => true\n\
                 _ => false\n\
               }\n\
             }\n",
        );
        let root = Path::new(&path)
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let summary = audit_corpus_parse_only(&[root]);
        assert!(
            summary
                .findings
                .iter()
                .any(|f| { f.rule == "syntactic_match_wildcard_arm" && f.site.contains("::f") }),
            "expected wildcard finding; got {:?}",
            summary.findings
        );
    }

    #[test]
    fn eval_interpreter_handlers_tagged_eval_interpreter_debt() {
        let summary = audit_corpus_default_roots();
        for site in [
            "src/v2/compiler/05_eval.dag::eval_bind_node_eval",
            "src/v2/compiler/05_eval.dag::eval_match_node_eval",
        ] {
            let finding = summary.findings.iter().find(|f| f.site == site);
            assert!(finding.is_some(), "expected syntactic finding for {site}");
            assert_eq!(
                finding.unwrap().triage,
                "eval-interpreter-debt",
                "expected eval-interpreter-debt triage for {site}"
            );
        }
    }

    #[test]
    fn live_tree_parse_audit_runs_over_witness_roots() {
        let summary = audit_corpus_default_roots();
        assert!(summary.files_scanned > 100, "corpus walk fail-opened");
        assert!(summary.files_parsed > 50, "parse fail-opened");
        assert!(summary.fns_scanned > 100, "fn scan fail-opened");
        assert!(
            !summary.findings.is_empty(),
            "expected syntactic findings on the live corpus"
        );
    }
}
