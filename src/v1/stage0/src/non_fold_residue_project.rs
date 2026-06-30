// The non-fold-residue audit (Lane 7 — DESIGN.md §6, docs/plans/fold-ergonomics.md §0/§3).
//
// DESIGN §6: "a finished stage is one fold (any non-fold residue is either a named irreducible kernel
// or un-migrated modeling — there is no third)." A fold over a CLOSED coproduct is total by
// construction and has NO `_ =>` wildcard arm; a hand-rolled `match` with a `_ =>` over a closed
// coproduct carries a fail-open escape that a fold would not. So a wildcard arm over a closed
// coproduct is non-fold residue — DESIGN §6's un-migrated modeling, the §0 fail-open shape that
// fold-ergonomics §0 traces "it compiles but nothing works" to.
//
// CONSTRUCTION-FIRST NOTE (DESIGN §5): the by-construction ideal is exhaustiveness-by-default — a
// typechecker that rejects a non-exhaustive / wildcard-over-closed-coproduct match (the §4
// CoproductExhaustiveness direction). That is a load-bearing typechecker change (escalate) AND a
// wildcard is legitimate over OPEN/primitive domains, so a blanket construction ban is wrong. Until
// exhaustiveness-by-default lands, the residue over CLOSED coproducts is the genuinely-unstructurable
// remainder a lens is for — walled fail-closed against a named, shrinking roster.
//
// DETECTION (conservative, decidable, low-false-positive — chosen to never false-red main):
//   Flag a `match` iff its SCRUTINEE is a bare identifier that is a FUNCTION PARAMETER whose declared
//   type is a known CLOSED coproduct (`type X = A | B | ...`), AND the match body has a top-level `_`
//   wildcard arm (`_ =>`). Bare-param + declared-coproduct-type is the case where the scrutinee type
//   is known WITHOUT inference; field accesses / call results / generic params are skipped
//   (conservative — fewer flags, never a false positive on an open domain). A `_` used as a field
//   placeholder (`Atom { identity: _ }`) is NOT a pattern wildcard (it is not followed by `=>`), so it
//   is excluded.
//
// Host-fed today; DISSOLUTION: folds into a pure `.dag` Node-tree reader (match nodes + scrutinee
// type) when exhaustiveness-by-default / compile-graph access lands (gunbc#5364). Additive corpus-gate
// builtin seam, sibling to inert_carrier_* / doc_graph_* — does NOT touch cli_run.rs's #5433 closure.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use crate::corpus_lex::{brace_delta, corpus_dag_files, is_test_dag, strip_line_comment};

// The NAMED exception roster: `file::fn` sites carrying a wildcard over a closed-coproduct param.
// Per DESIGN §6 each is EITHER (a) un-migrated modeling awaiting its fold OR (b) a named irreducible
// kernel — "there is no third." The wall is that no NEW such site merges unlisted; the ratchet is
// that a roster entry which STOPS being residue (file deleted, or the match migrated to a total fold)
// reds until removed. The two §6 categories dissolve differently, so the roster does NOT shrink
// uniformly to zero:
//   (a) un-migrated modeling — e.g. the `eval_*_node*` Node-interpreter handlers,
//       `dag_grammar_terminal_for_mvp1_*_token` — SHOULD become an exhaustive fold over the
//       coproduct; dissolve-on is to migrate the match (delete the wildcard). This subset shrinks.
//   (b) named irreducible kernel — the algebraic totals whose off-diagonal IS the residue the §6
//       kernel exemption was written for: `*_eq`/`*_relation_eq`/`*_mode_eq` (the `_ => false`
//       off-diagonal), the lattice `*_join`/`*_meet`/`*_combine` and partial-order `*_dominates`,
//       the boolean collapses (`exit_ok` = `ExitSuccess => true _ => false`, `program_runtime_bool_*`,
//       `is_*`/`float_body_is_nan` predicates). Enumerating every off-diagonal pair would NOT make
//       these "more total" — they wall PERMANENTLY and are expected to remain on the roster.
// (A per-entry kernel-vs-un-migrated TAG is deferred to the gunbc#5364 dissolution — the pure `.dag`
// Node-tree reader carries each site's match shape and scrutinee type structurally, so the
// classification is derived, not hand-maintained here, where a mistag would dishonestly mark
// migratable debt as a permanent kernel.)
//
// Roster partition (operator adjudication 2026-06-29): DECIDABLE-IRREDUCIBLE only vs MIGRATION-DEBT
// (must drain as folds land — must never stay hidden as permanent roster fiction).
const NON_FOLD_MIGRATION_DEBT_ROSTER: &[&str] = &[
    // (a) eval interpreter escapes — §5 fail-open on EVAL path; escalate before editing 05_eval.
    "src/v2/compiler/05_eval.dag::eval_bind_node_eval",
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
    "src/v2/extdeps/languages/dag.dag::dag_grammar_terminal_for_mvp1_bind_token",
    "src/v2/extdeps/languages/dag.dag::dag_grammar_terminal_for_mvp1_loop_token",
    "src/v2/extdeps/languages/dag.dag::dag_grammar_terminal_for_mvp1_match_token",
    "src/v2/extdeps/languages/dag.dag::dag_grammar_terminal_for_mvp1_pick_token",
    "src/v2/extdeps/languages/dag.dag::dag_grammar_terminal_for_mvp1_rec_token",
    // (a) other un-migrated modeling awaiting total fold.
    "dsl/extdeps/languages/markdown.dag::md_nested",
    "src/v2/extdeps/formats/spice_passive_projection.dag::passive_spec_from_component",
    "src/v2/extdeps/formats/spice_passive_projection.dag::passive_topology_from_component",
    "src/v2/extdeps/runtimes/v2_effect_io_pure.dag::effect_io_pure_backends_match",
    "src/v2/std/compilers/target_model.dag::target_type_expr_emitted_validate_wire_shape",
    "src/v2/std/compilers/target_model.dag::target_use_site_ownership_catalog_lookup_step",
    "src/v2/test/claim/manual/eval_runtime_mvp.dag::eval_mvp2_arg_is_two_literal",
    // (a) ManualAnchorKey closed-coproduct wildcards discovered by multiline type-index fix (2026-06-30).
    "src/v2/lens/testgen.dag::algebra_law_subject_for_manual_anchor",
    "src/v2/lens/testgen.dag::nat_manual_anchor_key_eq",
    "src/v2/lens/testgen.dag::testgen_emit_language_behavior_equivalence_claim",
    "src/v2/lens/testgen.dag::testgen_emit_refinement_preservation_claim",
    "src/v2/test/claim/generated/coproduct_exhaustiveness.dag::anchor_is",
    "src/v2/test/claim/generated/cross_representation_equality.dag::anchor_is_straddle",
];

const NON_FOLD_RESIDUE_ROSTER: &[&str] = &[
    // SEEDED FROM THE LIVE CENSUS 2026-06-22 (re-derive with the live_tree gate below). Each is a
    // `file::fn` with a `match <coproduct-param> { ... _ => ... }` — a wildcard escape over a closed
    // coproduct. This is the honest baseline; the wall is that no NEW residue merges.
    "dsl/extdeps/languages/markdown.dag::md_nested",
    "dsl/gunbc/generated_artifact.dag::artifact_eq",
    // Category (b) kernel — `*_eq` off-diagonal; dissolves with exhaustiveness-by-default (gunbc#5364).
    "dsl/gunbc/commit_workflow.dag::commit_workflow_surface_eq",
    "dsl/gunbc/commit_workflow.dag::gate_eq",
    "dsl/gunbc/commit_workflow.dag::local_tidy_check_eq",
    "dsl/std/computation.dag::constant_bound_value",
    "dsl/std/computation.dag::is_constant_bound",
    "dsl/std/effects.dag::create_double_init_collapsible",
    "dsl/std/effects.dag::create_effect_is_dedupable",
    "dsl/std/effects.dag::key_source_eq",
    "dsl/std/encoding.dag::encoding_lattice_join",
    "dsl/std/encoding.dag::encoding_lattice_meet",
    "dsl/std/filesystem.dag::is_text_encoding",
    "dsl/std/induction.dag::compose_sub_value",
    "dsl/std/induction.dag::compose_sub_value_relations",
    "dsl/std/induction.dag::is_strict_style_structural",
    "dsl/std/induction.dag::recursion_shape_eq",
    "dsl/std/induction.dag::shrink_factor_eq",
    "dsl/std/induction.dag::sub_value_structural_eq",
    "dsl/std/reducible.dag::reduce_verdict_combine",
    "dsl/std/termination.dag::descent_evidence_lattice_join",
    "dsl/std/termination.dag::descent_evidence_lattice_meet",
    "dsl/std/termination.dag::promote_to_strict",
    "dsl/tools/ci_gates.dag::exit_ok",
    "dsl/tools/generated_artifact_gate.dag::exit_ok",
    "src/v2/compiler/01_tokenize.dag::lex_try_rules_prefer_longer",
    "src/v2/compiler/05_eval.dag::eval_bind_node_eval",
    "src/v2/compiler/05_eval.dag::eval_branch_node_eval",
    "src/v2/compiler/05_eval.dag::eval_loop_node",
    "src/v2/compiler/05_eval.dag::eval_match_node_eval",
    "src/v2/compiler/05_eval.dag::eval_transform_node",
    "src/v2/compiler/05_eval.dag::eval_value_node",
    "src/v2/compiler/05_eval.dag::run_test_claim_assert_decided",
    "src/v2/compiler/05_eval.dag::run_test_claim_runtime_assert",
    "src/v2/compiler/06_translate.dag::translate_algebra_finalize",
    "src/v2/compiler/emit_host.dag::run_test_claim_emit_vs_eval_verdict",
    "src/v2/test/claim/manual/eval_runtime_mvp.dag::eval_mvp2_arg_is_two_literal",
    "src/v2/extdeps/formats/spice_passive_projection.dag::passive_spec_from_component",
    "src/v2/extdeps/formats/spice_passive_projection.dag::passive_topology_from_component",
    "src/v2/extdeps/languages/dag.dag::dag_grammar_terminal_for_mvp1_bind_token",
    "src/v2/extdeps/languages/dag.dag::dag_grammar_terminal_for_mvp1_loop_token",
    "src/v2/extdeps/languages/dag.dag::dag_grammar_terminal_for_mvp1_match_token",
    "src/v2/extdeps/languages/dag.dag::dag_grammar_terminal_for_mvp1_pick_token",
    "src/v2/extdeps/languages/dag.dag::dag_grammar_terminal_for_mvp1_rec_token",
    "src/v2/extdeps/runtimes/v2_effect_io_pure.dag::effect_io_pure_backends_match",
    "src/v2/lens/testgen.dag::algebra_law_subject_for_manual_anchor",
    "src/v2/lens/testgen.dag::nat_manual_anchor_key_eq",
    "src/v2/lens/testgen.dag::testgen_emit_language_behavior_equivalence_claim",
    "src/v2/lens/testgen.dag::testgen_emit_refinement_preservation_claim",
    "src/v2/test/claim/generated/coproduct_exhaustiveness.dag::anchor_is",
    "src/v2/test/claim/generated/cross_representation_equality.dag::anchor_is_straddle",
    "src/v2/lens/complexity.dag::complexity_bound_dominates",
    "src/v2/lens/complexity.dag::complexity_bound_from_class",
    "src/v2/lens/cost.dag::asymptotic_class_dominates",
    "src/v2/lens/cost.dag::multiply_classes",
    "src/v2/lens/cost.dag::symbolic_cost_dominates",
    "src/v2/lens/cost.dag::symbolic_cost_witness",
    "src/v2/lens/cost.dag::symbolic_max",
    "src/v2/lens/cost.dag::symbolic_product",
    "src/v2/lens/cost.dag::symbolic_sequential",
    "src/v2/lens/fact_density.dag::connective_is_kernel_ambient_atom",
    "src/v2/lens/idempotency.dag::idempotency_verdict_eq",
    "src/v2/lens/ownership.dag::ownership_mode_eq",
    "src/v2/lens/parallelism.dag::parallelism_relation_eq",
    "src/v2/lens/registry.dag::lens_id_v0_eq",
    "src/v2/lens/unused_parameters.dag::use_relation_eq",
    "src/v2/program.dag::program_runtime_bool_false",
    "src/v2/program.dag::program_runtime_bool_true",
    "src/v2/std/compilers/target_model.dag::source_atom_value_as_bool",
    "src/v2/std/compilers/target_model.dag::source_atom_value_as_char",
    "src/v2/std/compilers/target_model.dag::source_atom_value_as_string",
    "src/v2/std/compilers/target_model.dag::source_atom_value_as_symbol",
    "src/v2/std/compilers/target_model.dag::target_type_expr_emitted_validate_wire_shape",
    "src/v2/std/compilers/target_model.dag::target_use_site_ownership_catalog_lookup_step",
    "src/v2/std/effects.dag::key_source_eq",
    "src/v2/std/determinism.dag::determinism_class_eq",
    "src/v2/std/determinism.dag::non_det_source_eq",
    "src/v2/std/float.dag::float_body_is_nan",
    "src/v2/std/node_minimal.dag::node_superset_field_eq",
    "src/v2/std/probe_selector.dag::diagnostic_interface_kind_eq",
    "src/v2/std/qualified_name.dag::qn_fold_step",
];

// Corpus walk + lexical normalization (`corpus_dag_files`, `strip_line_comment`, `brace_delta`,
// `is_test_dag`) live in `crate::corpus_lex`, shared with the inert-carrier census — DESIGN §2/§3:
// one authority for "what is code text", not a copy per census module.

/// Source with every line's `//` comment removed and string-literal interiors blanked (positions
/// within a line up to the comment are preserved; the trailing comment becomes empty). Index-stable
/// enough for brace matching.
fn strip_comments(content: &str) -> String {
    content
        .lines()
        .map(strip_line_comment)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Closed-coproduct type names: a top-level `type X` whose declaration contains a `|` arm separator
/// (a sum type). Record types (`type X { ... }`) and aliases (`type X = Foo<Y>`) are not coproducts.
fn closed_coproduct_names(files: &[(String, String)]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (rel, content) in files {
        if is_test_dag(rel) {
            continue;
        }
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let trimmed = lines[i].trim_start();
            let Some(rest) = trimmed.strip_prefix("type ") else {
                i += 1;
                continue;
            };
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                i += 1;
                continue;
            }
            // Gather the decl block (same convention as inert_carrier_project) and check for `|`.
            let mut block = String::new();
            block.push_str(&strip_line_comment(lines[i]));
            let mut depth = brace_delta(lines[i]);
            i += 1;
            while i < lines.len() {
                let nt = lines[i].trim_start();
                if depth <= 0 {
                    if nt.is_empty() {
                        i += 1;
                        continue;
                    }
                    if !(nt.starts_with('|') || nt.starts_with('=')) {
                        if block.contains('|') || block.contains('=') {
                            break;
                        }
                        break;
                    }
                }
                block.push('\n');
                block.push_str(&strip_line_comment(lines[i]));
                depth += brace_delta(lines[i]);
                i += 1;
            }
            if block.contains('|') {
                out.insert(name);
            }
        }
    }
    out
}

fn is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !s.chars().next().unwrap().is_ascii_digit()
}

/// Find the index of the `}` matching the `{` at `open` (bytes; ASCII braces).
fn matching_brace(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut j = open;
    while j < bytes.len() {
        match bytes[j] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

/// Does `body` (a match's `{...}` interior) contain a top-level `_ =>` pattern wildcard — i.e. at
/// brace-depth 0 relative to the body, a `_` token immediately followed by `=>` (allowing spaces)?
/// Depth-0 only, so a nested match's wildcard is attributed to the nested match, not this one.
fn has_top_level_wildcard_arm(body: &str) -> bool {
    let bytes = body.as_bytes();
    let mut depth = 0i32;
    let mut k = 0;
    while k < bytes.len() {
        match bytes[k] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'_' => {
                // `_` must be a standalone token (not part of an identifier) and at depth 0.
                let prev_ok = k == 0 || !is_ident_byte(bytes[k - 1]);
                let next_is_ident = k + 1 < bytes.len() && is_ident_byte(bytes[k + 1]);
                if depth == 0 && prev_ok && !next_is_ident {
                    // skip spaces, expect `=>`
                    let mut m = k + 1;
                    while m < bytes.len()
                        && (bytes[m] == b' ' || bytes[m] == b'\n' || bytes[m] == b'\t')
                    {
                        m += 1;
                    }
                    if m + 1 < bytes.len() && bytes[m] == b'=' && bytes[m + 1] == b'>' {
                        return true;
                    }
                }
            }
            _ => {}
        }
        k += 1;
    }
    false
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Parse `fn NAME(params) ... {` signatures, returning (fn_name, params: name->bare-type, body_range)
/// over a comment-stripped source. The bare type is the type's head identifier (generics stripped).
struct FnSig {
    name: String,
    params: BTreeMap<String, String>,
    body: String,
}

fn parse_fns(src: &str) -> Vec<FnSig> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    for (start, _) in src.match_indices("fn ") {
        // `fn ` must start a token (preceded by whitespace/newline/start).
        if start > 0 && is_ident_byte(bytes[start - 1]) {
            continue;
        }
        let after = start + 3;
        let name: String = src[after..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }
        // params between the first `(` after the name and its matching `)`.
        let paren_open = match src[after..].find('(') {
            Some(p) => after + p,
            None => continue,
        };
        let paren_close = match matching_paren(bytes, paren_open) {
            Some(p) => p,
            None => continue,
        };
        let params = parse_params(&src[paren_open + 1..paren_close]);
        // body: first `{` after `)` to its matching `}`.
        let brace_open = match src[paren_close..].find('{') {
            Some(b) => paren_close + b,
            None => continue,
        };
        let brace_close = match matching_brace(bytes, brace_open) {
            Some(b) => b,
            None => continue,
        };
        out.push(FnSig {
            name,
            params,
            body: src[brace_open + 1..brace_close].to_string(),
        });
    }
    out
}

fn matching_paren(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut j = open;
    while j < bytes.len() {
        match bytes[j] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

/// Parse `name: Type, name2: Type2<G>` into name -> bare-type-head. Generics/brackets stripped.
fn parse_params(s: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    let mut parts: Vec<String> = Vec::new();
    for ch in s.chars() {
        match ch {
            '<' | '(' | '[' => {
                depth += 1;
                cur.push(ch);
            }
            '>' | ')' | ']' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => parts.push(std::mem::take(&mut cur)),
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur);
    }
    for part in parts {
        let Some((name, ty)) = part.split_once(':') else {
            continue;
        };
        let name = name.trim();
        let ty_head: String = ty
            .trim()
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if is_ident(name) && !ty_head.is_empty() {
            out.insert(name.to_string(), ty_head);
        }
    }
    out
}

/// A residue site: a `match <param> { ... _ => ... }` where param is a closed-coproduct type.
fn residue_sites(files: &[(String, String)]) -> Vec<String> {
    let coproducts = closed_coproduct_names(files);
    let mut out: BTreeSet<String> = BTreeSet::new();
    for (rel, content) in files {
        if is_test_dag(rel) {
            continue;
        }
        let src = strip_comments(content);
        for sig in parse_fns(&src) {
            // scan `match` occurrences in the fn body; resolve scrutinee against this fn's params.
            for (mi, _) in sig.body.match_indices("match ") {
                if mi > 0 && is_ident_byte(sig.body.as_bytes()[mi - 1]) {
                    continue;
                }
                let after = mi + "match ".len();
                let Some(brace_rel) = sig.body[after..].find('{') else {
                    continue;
                };
                let scrut = sig.body[after..after + brace_rel].trim();
                if !is_ident(scrut) {
                    continue; // not a bare identifier (field access / call) -> skip, conservative.
                }
                let Some(ty) = sig.params.get(scrut) else {
                    continue; // scrutinee is not a known param -> skip (no type without inference).
                };
                if !coproducts.contains(ty) {
                    continue; // param is not a closed coproduct -> legitimate / open domain.
                }
                // brace-match the match body and check for a top-level `_ =>` arm.
                let body_bytes = sig.body.as_bytes();
                let brace_abs = after + brace_rel;
                let Some(close) = matching_brace(body_bytes, brace_abs) else {
                    continue;
                };
                let body = &sig.body[brace_abs + 1..close];
                if has_top_level_wildcard_arm(body) {
                    out.insert(format!("{}::{}", rel, sig.name));
                }
            }
        }
    }
    out.into_iter().collect()
}

/// The memoized live-tree census: residue sites + closed-coproduct universe size, both derived from a
/// single `dsl/` + `src/v2/` walk. The on-disk corpus is fixed for a process's lifetime, so the four
/// `non_fold_residue_*_count` builtins (called one-by-one per witness eval) share one walk + scan
/// instead of re-walking per call. The pure `residue_sites` / `closed_coproduct_names` stay taking
/// `&[files]` so the synthetic RED/GREEN controls keep driving them with in-memory corpora.
struct NonFoldReport {
    sites: Vec<String>,
    coproduct_universe: usize,
    closed_coproduct_names: BTreeSet<String>,
}

fn build_report() -> &'static NonFoldReport {
    static REPORT: OnceLock<NonFoldReport> = OnceLock::new();
    REPORT.get_or_init(|| {
        let files = corpus_dag_files();
        let closed_coproduct_names = closed_coproduct_names(&files);
        NonFoldReport {
            sites: residue_sites(&files),
            coproduct_universe: closed_coproduct_names.len(),
            closed_coproduct_names,
        }
    })
}

pub fn non_fold_residue_closed_coproduct_type_names() -> &'static BTreeSet<String> {
    &build_report().closed_coproduct_names
}

pub fn non_fold_residue_count() -> i64 {
    build_report().sites.len() as i64
}

/// The fail-closed GATE: residue sites NOT on the named exception roster.
pub fn non_fold_residue_unrostered_count() -> i64 {
    let roster: BTreeSet<&str> = NON_FOLD_RESIDUE_ROSTER.iter().copied().collect();
    build_report()
        .sites
        .iter()
        .filter(|s| !roster.contains(s.as_str()))
        .count() as i64
}

pub fn non_fold_residue_site_is_rostered(site: &str) -> bool {
    NON_FOLD_RESIDUE_ROSTER.contains(&site)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonFoldRosterBucket {
    Irreducible,
    MigrationDebt,
}

pub fn non_fold_residue_roster_bucket(site: &str) -> Option<NonFoldRosterBucket> {
    if !non_fold_residue_site_is_rostered(site) {
        return None;
    }
    if NON_FOLD_MIGRATION_DEBT_ROSTER.contains(&site) {
        Some(NonFoldRosterBucket::MigrationDebt)
    } else {
        Some(NonFoldRosterBucket::Irreducible)
    }
}

pub fn non_fold_residue_migration_debt_live_count() -> i64 {
    build_report()
        .sites
        .iter()
        .filter(|s| {
            matches!(
                non_fold_residue_roster_bucket(s),
                Some(NonFoldRosterBucket::MigrationDebt)
            )
        })
        .count() as i64
}

pub fn non_fold_residue_irreducible_live_count() -> i64 {
    build_report()
        .sites
        .iter()
        .filter(|s| {
            matches!(
                non_fold_residue_roster_bucket(s),
                Some(NonFoldRosterBucket::Irreducible)
            )
        })
        .count() as i64
}

pub fn non_fold_residue_migration_debt_roster_slots() -> i64 {
    NON_FOLD_MIGRATION_DEBT_ROSTER.len() as i64
}

pub fn non_fold_residue_irreducible_roster_slots() -> i64 {
    (NON_FOLD_RESIDUE_ROSTER.len() - NON_FOLD_MIGRATION_DEBT_ROSTER.len()) as i64
}

/// The RATCHET: roster entries that are no longer residue (migrated to a fold, or deleted).
pub fn non_fold_residue_stale_roster_count() -> i64 {
    let live: BTreeSet<&str> = build_report().sites.iter().map(|s| s.as_str()).collect();
    NON_FOLD_RESIDUE_ROSTER
        .iter()
        .filter(|s| !live.contains(*s))
        .count() as i64
}

/// Closed-coproduct universe size — fail-open oracle (zero means the corpus walk found nothing).
pub fn non_fold_residue_coproduct_universe_count() -> i64 {
    build_report().coproduct_universe as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(p, c)| (p.to_string(), c.to_string()))
            .collect()
    }

    #[test]
    fn coproduct_index_finds_sums_not_records() {
        let f = files(&[(
            "t.dag",
            "module t\ntype Mode = A | B | C\ntype Rec { x: Int }\ntype Alias = Witness<Int>\n",
        )]);
        let cps = closed_coproduct_names(&f);
        assert!(cps.contains("Mode"));
        assert!(!cps.contains("Rec"));
        assert!(!cps.contains("Alias"));
    }

    #[test]
    fn red_control_wildcard_over_closed_coproduct_is_residue() {
        // A hand-rolled match on a coproduct param with a `_ =>` escape — non-fold residue (RED).
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B | C\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    _ => false\n  }\n}\n",
        )]);
        let sites = residue_sites(&f);
        assert!(
            sites.contains(&"m.dag::f".to_string()),
            "a wildcard over a closed-coproduct param must be flagged; got {sites:?}"
        );
    }

    #[test]
    fn green_control_total_fold_is_not_residue() {
        // The SAME function, exhaustive (no wildcard) — a total fold, NOT residue (GREEN). The only
        // difference from the RED control is the `_ =>` arm: the discrimination.
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B | C\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    B => false\n    C => false\n  }\n}\n",
        )]);
        let sites = residue_sites(&f);
        assert!(
            !sites.contains(&"m.dag::f".to_string()),
            "an exhaustive match (no wildcard) must NOT be flagged; got {sites:?}"
        );
    }

    #[test]
    fn green_control_wildcard_over_open_domain_is_not_residue() {
        // A wildcard over a NON-coproduct param (e.g. a String/primitive) is legitimate, not residue.
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B\nfn g(s: String) -> Bool {\n  match s {\n    \"y\" => true\n    _ => false\n  }\n}\n",
        )]);
        let sites = residue_sites(&f);
        assert!(
            !sites.contains(&"m.dag::g".to_string()),
            "a wildcard over an open/primitive domain must NOT be flagged; got {sites:?}"
        );
    }

    #[test]
    fn green_control_field_placeholder_underscore_is_not_a_wildcard_arm() {
        // `_` as a record-field placeholder (`A { v: _ }`) is not a pattern wildcard arm.
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A { v: Int } | B { v: Int }\nfn f(x: Mode) -> Int {\n  match x {\n    A { v: _ } => 1\n    B { v: _ } => 2\n  }\n}\n",
        )]);
        let sites = residue_sites(&f);
        assert!(
            !sites.contains(&"m.dag::f".to_string()),
            "field-placeholder `_` is not a wildcard arm; got {sites:?}"
        );
    }

    #[test]
    fn nested_match_wildcard_is_attributed_to_its_own_match() {
        // The outer match on `a` is exhaustive; the inner `match b` carries the wildcard — the fn is
        // flagged (it has residue), via the inner match. Confirms depth-0 wildcard attribution finds
        // the inner match's escape.
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B\nfn eq(a: Mode, b: Mode) -> Bool {\n  match a {\n    A => match b { A => true _ => false }\n    B => match b { B => true _ => false }\n  }\n}\n",
        )]);
        let sites = residue_sites(&f);
        assert!(sites.contains(&"m.dag::eq".to_string()));
    }

    #[test]
    fn green_control_wildcard_and_slashes_inside_string_literal_are_ignored() {
        // String-literal awareness: a `_ =>` and a `//` that live INSIDE a `"..."` string literal
        // are not code. The fn below is a TOTAL fold over `Mode` (arms A and B, no real wildcard); the
        // only `_ =>` and `//` appear inside a string. A naive scanner would (a) read the `//` as a
        // comment and truncate, and/or (b) read the in-string `_ =>` as a wildcard arm and false-RED.
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B\nfn f(x: Mode) -> String {\n  match x {\n    A => \"see https://x/y and _ => z\"\n    B => \"b\"\n  }\n}\n",
        )]);
        let sites = residue_sites(&f);
        assert!(
            !sites.contains(&"m.dag::f".to_string()),
            "`_ =>`/`//` inside a string literal must not be read as code; got {sites:?}"
        );
    }

    #[test]
    fn red_control_real_wildcard_survives_an_in_string_decoy() {
        // Discrimination partner of the green control: the SAME in-string decoy, but now a REAL
        // top-level `_ =>` wildcard arm is present too. It must still be flagged — string blanking
        // must not suppress genuine residue.
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B | C\nfn f(x: Mode) -> String {\n  match x {\n    A => \"see https://x/y and _ => z\"\n    _ => \"rest\"\n  }\n}\n",
        )]);
        let sites = residue_sites(&f);
        assert!(
            sites.contains(&"m.dag::f".to_string()),
            "a real wildcard arm must still be flagged despite an in-string decoy; got {sites:?}"
        );
    }

    #[test]
    fn live_tree_coproduct_universe_is_nonempty() {
        assert!(
            non_fold_residue_coproduct_universe_count() > 0,
            "expected a non-empty closed-coproduct universe; zero means the corpus walk fail-opened"
        );
    }

    #[test]
    fn roster_partition_covers_every_entry_without_overlap() {
        let migration: BTreeSet<&str> = NON_FOLD_MIGRATION_DEBT_ROSTER.iter().copied().collect();
        let roster: BTreeSet<&str> = NON_FOLD_RESIDUE_ROSTER.iter().copied().collect();
        assert!(
            migration.is_subset(&roster),
            "migration-debt roster must be subset of full roster; extra={:?}",
            migration.difference(&roster).collect::<Vec<_>>()
        );
        assert_eq!(
            NON_FOLD_MIGRATION_DEBT_ROSTER.len()
                + non_fold_residue_irreducible_roster_slots() as usize,
            NON_FOLD_RESIDUE_ROSTER.len(),
            "roster partition must cover every slot"
        );
        assert_eq!(
            non_fold_residue_migration_debt_live_count()
                + non_fold_residue_irreducible_live_count(),
            non_fold_residue_count(),
            "live residue sites must partition into migration-debt + irreducible"
        );
    }

    #[test]
    fn live_tree_no_unrostered_non_fold_residue() {
        let roster: BTreeSet<&str> = NON_FOLD_RESIDUE_ROSTER.iter().copied().collect();
        let unrostered: Vec<String> = residue_sites(&corpus_dag_files())
            .into_iter()
            .filter(|s| !roster.contains(s.as_str()))
            .collect();
        assert!(
            unrostered.is_empty(),
            "new non-fold residue (wildcard over a closed-coproduct param) — migrate to a total fold \
             or add to NON_FOLD_RESIDUE_ROSTER with a dissolve-on: {unrostered:?}"
        );
    }

    #[test]
    fn live_tree_residue_roster_has_no_stale_entries() {
        let live: BTreeSet<String> = residue_sites(&corpus_dag_files()).into_iter().collect();
        let stale: Vec<&&str> = NON_FOLD_RESIDUE_ROSTER
            .iter()
            .filter(|s| !live.contains(&s.to_string()))
            .collect();
        assert!(
            stale.is_empty(),
            "roster entries that are no longer residue (migrated or deleted) — remove them so the \
             roster shrinks: {stale:?}"
        );
    }
}
