// SCAFFOLD — Emit-only whole-corpus audit for the complexity/linearity lens family (SYNTACTIC half).
// Lane 7 — DESIGN.md §6, ROADMAP.md §3 "complexity budget gates the whole codebase"
// (row `3-gates-whole`, gated on fn-body reflection / `decl_facts(roots)` host builtin #5966).
//
// AUDIT-FIRST BRIDGE: parse-only walk over `witness_layer_roots` until whole-corpus fn-body
// reflection grounds. Substrate: `coproduct_reflection::decl_facts_for_roots(roots)` — locked carrier
// shape `{qualified_name, name, kind, node}` via the `decl_facts(roots)` host builtin (#5966).
//
// DISSOLUTION TRIGGER (named, checkable):
//   1. ~~Swap `decl_facts_parse_only` body for real `decl_facts(roots)` when #5966 lands.~~ DONE.
//   2. Move `triage_wildcard` / `triage_complexity` site-classification tables on-carrier
//      alongside decl_facts — do not let hand-Rust substring heuristics survive the swap.
//   3. Fold SYNTACTIC projections into a pure `.dag` Node-tree reader when compile-graph access
//      lands (gunbc#5364) — same additive builtin seam as non_fold_residue_* / inert_carrier_*.
//   4. Flip floor enrollment once whole-corpus reflection grounds (ROADMAP §3 `3-gates-whole`).
//
// RESOLVED-half findings (closed-coproduct-param non_fold, inert-carrier consumers)
// remain #5364-gated — reported via existing roster builtins, not whole-corpus here.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::OnceLock;

use crate::cli_run::{
    non_fold_residue_count, non_fold_residue_live_sites, non_fold_residue_roster_size,
    non_fold_residue_site_is_rostered, non_fold_residue_unrostered_count, witness_layer_roots,
};

// Migration-debt sub-roster — subset of NON_FOLD_RESIDUE_ROSTER in cli_run.
// Dissolves to Chunk F when residue classification moves on-carrier (gunbc#5364).
const NON_FOLD_MIGRATION_DEBT_ROSTER: &[&str] = &[
    // (a) eval interpreter escapes — §5 fail-open on EVAL path; escalate before editing 05_eval.
    "src/v2/compiler/05_eval.dag::eval_branch_node_eval",
    "src/v2/compiler/05_eval.dag::eval_loop_node",
    "src/v2/compiler/05_eval.dag::eval_match_node_eval",
    "src/v2/compiler/05_eval.dag::eval_transform_node",
    "src/v2/compiler/05_eval.dag::eval_value_node",
    "src/v2/compiler/05_eval.dag::run_test_claim_assert_decided",
    "src/v2/compiler/05_eval.dag::run_test_claim_runtime_assert",
    // (a) grammar-ladder dispatch + other pipeline stages (escalate before stage edits).
    "src/v2/compiler/01_tokenize.dag::lex_try_rules_prefer_longer",
    "src/v2/compiler/06_translate.dag::translate_algebra_finalize",
    "src/v2/compiler/emit_host.dag::run_test_claim_emit_vs_eval_verdict",
    // (a) other un-migrated modeling awaiting total fold.
    "dag/extdeps/languages/markdown.dag::md_nested",
    "src/v2/extdeps/formats/spice_passive_projection.dag::passive_spec_from_component",
    "src/v2/extdeps/formats/spice_passive_projection.dag::passive_topology_from_component",
    "src/v2/extdeps/runtimes/v2_effect_io_pure.dag::effect_io_pure_backends_match",
    "src/v2/std/compilers/target_model.dag::target_type_expr_emitted_validate_wire_shape",
    "src/v2/std/compilers/target_model.dag::target_use_site_ownership_catalog_lookup_step",
    "src/v2/test/claim/manual/eval_runtime_mvp.dag::eval_mvp2_arg_is_two_literal",
    // (a) ManualAnchorKey closed-coproduct wildcards discovered by multiline type-index fix.
    "src/v2/lens/testgen.dag::algebra_law_subject_for_manual_anchor",
    "src/v2/lens/testgen.dag::nat_manual_anchor_key_eq",
    "src/v2/lens/testgen.dag::testgen_emit_language_behavior_equivalence_claim",
    "src/v2/lens/testgen.dag::testgen_emit_refinement_preservation_claim",
    "src/v2/test/claim/generated/coproduct_exhaustiveness.dag::anchor_is",
    "src/v2/test/claim/generated/cross_representation_equality.dag::anchor_is_straddle",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NonFoldRosterBucket {
    Irreducible,
    MigrationDebt,
}

fn nfr_roster_bucket(site: &str) -> Option<NonFoldRosterBucket> {
    if !non_fold_residue_site_is_rostered(site) {
        return None;
    }
    if NON_FOLD_MIGRATION_DEBT_ROSTER.contains(&site) {
        Some(NonFoldRosterBucket::MigrationDebt)
    } else {
        Some(NonFoldRosterBucket::Irreducible)
    }
}
use crate::coproduct_reflection::{decl_facts_corpus_walk, DeclFactRaw};
use crate::v1_compiler_infer_items::ItemKind;
use crate::v1_std_core::{
    arm_pattern, authored_name_at, expr_var_name_at, match_arm_nodes, match_scrutinee,
    param_node_name_at, param_node_type_expr, ExprData, MatchPattern, Node,
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

fn is_wildcard_arm(arm: &Rc<Node>) -> bool {
    matches!(arm_pattern(arm.clone()).as_ref(), MatchPattern::Wildcard)
}

fn type_expr_head(
    ty: Rc<Node>,
    si: &Rc<std::collections::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> String {
    let name = authored_name_at(si.clone(), ty);
    name.chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}

fn fn_param_type_heads(
    item: &Rc<Node>,
    si: &Rc<std::collections::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for param in item.params.iter() {
        let pname = param_node_name_at(param.clone(), si.clone());
        if pname.is_empty() {
            continue;
        }
        let head = type_expr_head(param_node_type_expr(param.clone()), si);
        if !head.is_empty() {
            out.insert(pname, head);
        }
    }
    out
}

fn is_closed_coproduct_param_scrutinee(
    scrutinee_name: &str,
    param_types: &BTreeMap<String, String>,
    closed: &BTreeSet<String>,
) -> bool {
    param_types
        .get(scrutinee_name)
        .is_some_and(|ty| closed.contains(ty))
}

fn audit_decl_fact(
    fact: &DeclFactRaw,
    si: &Rc<std::collections::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Vec<AuditFinding> {
    let Some(body) = fact.node.body.as_ref() else {
        return Vec::new();
    };
    let param_types = fn_param_type_heads(&fact.node, si);
    audit_function_body(&fact.rel_path, &fact.name, body, si, &param_types)
}

fn audit_function_body(
    rel: &str,
    fn_name: &str,
    body: &Rc<Node>,
    si: &Rc<std::collections::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    param_types: &BTreeMap<String, String>,
) -> Vec<AuditFinding> {
    let closed = crate::cli_run::non_fold_residue_closed_coproduct_type_names();
    let mut stats = FnBodyStats::default();
    walk_expr(body, si, param_types, closed, &mut stats);
    let site = format!("{rel}::{fn_name}");
    let mut out = Vec::new();
    if stats.wildcard_matches > 0 {
        // Triage is grounded in `v2.lens.complexity_linearity_audit` (.dag), which owns the
        // bucket decision tree over `complexity_linearity_wildcard_facts()` raw facts.
        // This field is a non-authoritative rule echo — do not classify here (single authority §3).
        out.push(AuditFinding {
            site: site.clone(),
            lens: "non_fold_residue",
            rule: "syntactic_match_wildcard_arm",
            triage: "wildcard-arm",
        });
    }
    if stats.match_count >= 8 || (stats.node_count >= 200 && stats.match_count >= 4) {
        out.push(AuditFinding {
            site: site.clone(),
            lens: "cost",
            rule: "syntactic_high_match_fanout",
            triage: triage_complexity(&site),
        });
    }
    out
}

fn walk_expr(
    node: &Rc<Node>,
    si: &Rc<std::collections::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    param_types: &BTreeMap<String, String>,
    closed_coproducts: &BTreeSet<String>,
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
        // SYNTACTIC (audit-first): any `match` with a `_ =>` arm — signal mixed with kernel noise,
        // filtered by closed-coproduct param resolution below. GATE PROMOTION (WallAfterGrounding):
        // must resolve scrutinee to a closed coproduct (see `non_fold_residue_project` conservative
        // detection) — not path/name substring triage; otherwise §5 validation-not-construction.
        if has_wildcard {
            stats.wildcard_matches += 1;
            if !scrutinee_name.is_empty() {
                stats
                    .wildcard_scrutinee_names
                    .insert(scrutinee_name.clone());
                if is_closed_coproduct_param_scrutinee(
                    &scrutinee_name,
                    param_types,
                    closed_coproducts,
                ) {
                    stats.closed_coproduct_wildcard_matches += 1;
                }
            }
        }
    }
    for child in node.children.iter() {
        walk_expr(child, si, param_types, closed_coproducts, stats);
    }
}

#[derive(Default)]
struct FnBodyStats {
    node_count: usize,
    match_count: usize,
    wildcard_matches: usize,
    closed_coproduct_wildcard_matches: usize,
    wildcard_scrutinee_names: BTreeSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RosterFictionReport {
    pub resolved_residue_sites: i64,
    pub resolved_unrostered_sites: i64,
    pub migration_debt_live: i64,
    pub irreducible_live: i64,
    pub migration_debt_roster_slots: i64,
    pub irreducible_roster_slots: i64,
    pub floor_red_if_migration_roster_fiction_dropped: i64,
    pub honest_irreducible_rostered: i64,
}

// Wildcard-arm triage counters (`syntactic_wildcard_*`, `*_debt`, `kernel_permanent`, ...) were
// removed from this Rust report: their single authority is now `v2.lens.complexity_linearity_audit`
// (.dag), folding over `complexity_linearity_wildcard_facts()`. The `eval_interpreter_debt`,
// `grammar_ladder_debt`, and `triage_pending` counters were structurally dead here (the old
// `triage_wildcard` never emitted those strings) and are not reintroduced downstream.
pub fn roster_fiction_report(_summary: &AuditSummary) -> RosterFictionReport {
    let resolved_residue_sites = non_fold_residue_count();
    let resolved_unrostered_sites = non_fold_residue_unrostered_count();
    let live_sites = crate::cli_run::non_fold_residue_live_sites();
    let migration_debt_roster_slots = NON_FOLD_MIGRATION_DEBT_ROSTER.len() as i64;
    let migration_debt_live = live_sites
        .iter()
        .filter(|s| {
            matches!(
                nfr_roster_bucket(s),
                Some(NonFoldRosterBucket::MigrationDebt)
            )
        })
        .count() as i64;
    let irreducible_live = live_sites
        .iter()
        .filter(|s| matches!(nfr_roster_bucket(s), Some(NonFoldRosterBucket::Irreducible)))
        .count() as i64;
    let irreducible_roster_slots =
        crate::cli_run::non_fold_residue_roster_size() - migration_debt_roster_slots;
    RosterFictionReport {
        resolved_residue_sites,
        resolved_unrostered_sites,
        migration_debt_live,
        irreducible_live,
        migration_debt_roster_slots,
        irreducible_roster_slots,
        floor_red_if_migration_roster_fiction_dropped: migration_debt_live,
        honest_irreducible_rostered: irreducible_live,
    }
}

fn is_kernel_permanent_fn(fn_name: &str) -> bool {
    fn_name.ends_with("_eq")
        || fn_name.contains("dominates")
        || fn_name.contains("lattice_join")
        || fn_name.contains("lattice_meet")
        || fn_name == "exit_ok"
        || fn_name.contains("_relation_eq")
        || fn_name.contains("_mode_eq")
        || fn_name.ends_with("_combine")
        || fn_name == "constant_bound_value"
        || fn_name == "is_constant_bound"
        || fn_name == "create_double_init_collapsible"
        || fn_name == "create_effect_is_dedupable"
        || fn_name.starts_with("compose_sub_value")
        || fn_name == "promote_to_strict"
        || fn_name.starts_with("program_runtime_bool")
        || fn_name == "is_text_encoding"
        || fn_name == "is_strict_style_structural"
}

// Emit-only path buckets for `syntactic_high_match_fanout` (cost lens) — not used by floor gates.
// Dissolution trigger #2: delete when triage moves on-carrier with decl_facts (#5966).
fn triage_complexity(site: &str) -> &'static str {
    let fn_name = site.rsplit("::").next().unwrap_or("");
    if is_kernel_permanent_fn(fn_name) {
        return "kernel-permanent";
    }
    if site.starts_with("dag/extdeps/")
        || site.starts_with("dag/ctrl/")
        || site.starts_with("dag/gunbc/plans/")
        || site.starts_with("dag/test/")
    {
        "open-domain"
    } else if site.starts_with("dag/std/") || site.starts_with("dag/gunbc/") {
        "kernel-permanent"
    } else {
        "open-domain"
    }
}

pub fn audit_corpus_over_decl_facts(roots: &[String]) -> AuditSummary {
    let walk = decl_facts_corpus_walk(roots);
    let mut summary = AuditSummary::default();
    summary.files_scanned = walk.files_scanned;
    summary.files_parsed = walk.files_parsed;

    for fact in &walk.facts {
        if !matches!(fact.kind, ItemKind::FnItem | ItemKind::FuncItem) {
            continue;
        }
        summary.fns_scanned += 1;
        summary
            .findings
            .extend(audit_decl_fact(fact, &fact.source_indices));
    }
    summary.findings.sort();
    summary
}

/// Parse-only audit walk — implemented via `decl_facts` stub carrier.
pub fn audit_corpus_parse_only(roots: &[String]) -> AuditSummary {
    audit_corpus_over_decl_facts(roots)
}

pub fn audit_corpus_default_roots() -> AuditSummary {
    audit_corpus_parse_only(&witness_layer_roots())
}

struct AuditBuiltinCache {
    finding_count: i64,
    sites: BTreeSet<String>,
}

// Host builtins cache a single witness-layer-roots census (global / default-roots-only).
// Per-root `audit_corpus_parse_only(roots)` is for the emit bin and unit tests only.
fn cached_builtin_cache() -> &'static AuditBuiltinCache {
    static CACHE: OnceLock<AuditBuiltinCache> = OnceLock::new();
    CACHE.get_or_init(|| {
        let summary = audit_corpus_default_roots();
        AuditBuiltinCache {
            finding_count: summary.findings.len() as i64,
            sites: summary.findings.iter().map(|f| f.site.clone()).collect(),
        }
    })
}

pub fn complexity_linearity_syntactic_finding_count() -> i64 {
    cached_builtin_cache().finding_count
}

pub fn complexity_linearity_syntactic_site_fired(site: &str) -> bool {
    cached_builtin_cache().sites.contains(site)
}

/// Raw structural fact for one function carrying at least one wildcard match arm.
/// The thin Rust bridge: only what requires real fn-body/type parsing. All triage/bucket
/// classification is grounded in `v2.lens.complexity_linearity_audit` (.dag) over these facts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WildcardSiteFactRaw {
    pub site: String,
    pub fn_name: String,
    pub closed_coproduct_wildcard: bool,
    pub rostered: bool,
}

struct WildcardFactsCache {
    facts: Vec<WildcardSiteFactRaw>,
}

fn cached_wildcard_facts() -> &'static WildcardFactsCache {
    static CACHE: OnceLock<WildcardFactsCache> = OnceLock::new();
    CACHE.get_or_init(|| WildcardFactsCache {
        facts: compute_wildcard_facts(&witness_layer_roots()),
    })
}

fn compute_wildcard_facts(roots: &[String]) -> Vec<WildcardSiteFactRaw> {
    let walk = decl_facts_corpus_walk(roots);
    let closed = crate::cli_run::non_fold_residue_closed_coproduct_type_names();
    let mut out = Vec::new();
    for fact in &walk.facts {
        if !matches!(fact.kind, ItemKind::FnItem | ItemKind::FuncItem) {
            continue;
        }
        let Some(body) = fact.node.body.as_ref() else {
            continue;
        };
        let param_types = fn_param_type_heads(&fact.node, &fact.source_indices);
        let mut stats = FnBodyStats::default();
        walk_expr(body, &fact.source_indices, &param_types, closed, &mut stats);
        if stats.wildcard_matches > 0 {
            let site = format!("{}::{}", fact.rel_path, fact.name);
            out.push(WildcardSiteFactRaw {
                fn_name: fact.name.clone(),
                closed_coproduct_wildcard: stats.closed_coproduct_wildcard_matches > 0,
                rostered: crate::cli_run::non_fold_residue_site_is_rostered(&site),
                site,
            });
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Host-builtin surface: raw wildcard-site facts over the default witness-layer roots.
pub fn complexity_linearity_wildcard_facts() -> &'static [WildcardSiteFactRaw] {
    &cached_wildcard_facts().facts
}

/// Host-builtin surface: the migration-debt sub-roster exposed as data so the `.dag` lens
/// owns the membership classification (rather than querying a Rust predicate).
pub fn complexity_linearity_migration_debt_roster() -> Vec<String> {
    NON_FOLD_MIGRATION_DEBT_ROSTER
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
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

    // Triage itself (bucket strings) is grounded + verified in
    // `src/v2/test/claim/complexity_linearity/syntactic_audit_witness_test.dag`. Here we assert only
    // the RAW facts that drive it: presence, `rostered`, and migration-debt roster membership.
    #[test]
    fn eval_interpreter_handler_is_migration_debt_raw_fact() {
        let facts = complexity_linearity_wildcard_facts();
        let eval_bind_site = "src/v2/compiler/05_eval.dag::eval_bind_node_eval";
        assert!(
            !facts.iter().any(|f| f.site == eval_bind_site),
            "eval_bind_node_eval wildcard dissolved; should not appear in wildcard facts"
        );
        let site = "src/v2/compiler/05_eval.dag::eval_match_node_eval";
        let fact = facts.iter().find(|f| f.site == site);
        assert!(fact.is_some(), "expected wildcard fact for {site}");
        assert!(
            fact.unwrap().rostered,
            "{site} must be rostered (drives migration-debt/kernel-permanent triage in .dag)"
        );
        let migration_roster = complexity_linearity_migration_debt_roster();
        assert!(
            migration_roster.iter().any(|s| s == site),
            "{site} must be in the migration-debt roster (→ migration-debt triage)"
        );
    }

    #[test]
    fn testgen_anchor_match_is_migration_debt_raw_fact() {
        let site = "src/v2/lens/testgen.dag::testgen_emit_language_behavior_equivalence_claim";
        let facts = complexity_linearity_wildcard_facts();
        let fact = facts.iter().find(|f| f.site == site);
        assert!(
            fact.is_some(),
            "expected wildcard fact for testgen anchor match"
        );
        let migration_roster = complexity_linearity_migration_debt_roster();
        assert!(
            migration_roster.iter().any(|s| s == site),
            "enrolled ManualAnchorKey wildcard site must be in migration-debt roster"
        );
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

    #[test]
    fn roster_partition_covers_every_entry_without_overlap() {
        // Every migration-debt entry must exist in the full roster.
        for site in NON_FOLD_MIGRATION_DEBT_ROSTER {
            assert!(
                non_fold_residue_site_is_rostered(site),
                "migration-debt roster entry missing from full roster: {site}"
            );
        }
        // Partition sums: migration slots + irreducible slots == full roster size.
        let migration_slots = NON_FOLD_MIGRATION_DEBT_ROSTER.len() as i64;
        let irreducible_slots = non_fold_residue_roster_size() - migration_slots;
        assert!(
            irreducible_slots >= 0,
            "migration-debt roster is larger than full roster"
        );
        assert_eq!(
            migration_slots + irreducible_slots,
            non_fold_residue_roster_size(),
            "roster partition must cover every slot"
        );
        // Live sites must partition into migration-debt + irreducible (no unclassified rostered sites).
        let live = non_fold_residue_live_sites();
        let migration_live = live
            .iter()
            .filter(|s| {
                matches!(
                    nfr_roster_bucket(s),
                    Some(NonFoldRosterBucket::MigrationDebt)
                )
            })
            .count() as i64;
        let irreducible_live = live
            .iter()
            .filter(|s| matches!(nfr_roster_bucket(s), Some(NonFoldRosterBucket::Irreducible)))
            .count() as i64;
        assert_eq!(
            migration_live + irreducible_live,
            non_fold_residue_count(),
            "live residue sites must partition into migration-debt + irreducible"
        );
    }
}
