//! Structural regression guard for the #5146-class redundant-re-traversal
//! anti-pattern that made `resolve_expr_types` O(2^depth) on nested literals
//! (the N_v2 self-host blocker; fixed by bind-once in `04_resolve.dag` +
//! `v1_compiler_infer_resolve.rs`).
//!
//! THE ANTI-PATTERN (general, not a `resolve_expr_types` one-off): a single
//! `match`-arm performs TWO sibling traversals of the same children collection
//! that EACH invoke the same recursive self-call — once to build the resolved
//! children, once to re-collect their diagnostics. Because a nested
//! constructor's tail child is itself such a node, every level doubles the
//! recursion → exponential. The fix binds the per-child result ONCE and splits
//! it (`map(r => r.expr)` / `flat_map(r => r.diagnostics)`), so the self-call
//! appears in exactly one traversal per arm.
//!
//! This guard SOURCE-BRIDGES the REAL function (parses `src/v1/04_resolve.dag`
//! to a `Node` tree and inspects its AST — shape, not text), so it regression-
//! guards the actual authority: if anyone reintroduces a second self-calling
//! traversal in any arm, `current` goes RED. The `pre_fix` half re-runs the
//! detector against the committed pre-fix source (the real buggy Node) and
//! asserts it DOES fire — so the detector keeps its teeth (a guard that can no
//! longer catch the known bug is itself a regression).

use std::collections::HashMap;
use std::process::Command;
use std::rc::Rc;

use v1_compiler::v1_compiler_parse::parse_with_table;
use v1_compiler::v1_compiler_tokenize::tokenize;
use v1_compiler::v1_std_core::{
    authored_name_at, build_newline_index, empty_intern_table, module_items, ExprData,
    NewlineIndex, Node,
};

use crate::helpers::workspace_root;

const RESOLVE_DAG: &str = "src/v1/04_resolve.dag";
const SELF_FN: &str = "resolve_expr_types";
// The pre-fix commit (merge-base of the FIX-A branch) where every variable-arity
// arm still double-resolved. Used only to prove the detector discriminates.
const PRE_FIX_REV: &str = "b7d11aa73";

type SiMap = Rc<HashMap<String, Rc<NewlineIndex>>>;

fn parse_module_from_source(path: &str, content: &str) -> (Rc<Node>, SiMap) {
    let tokens = tokenize(content.to_string(), path.to_string());
    let nl = build_newline_index(path.to_string(), content.to_string());
    let mut si: HashMap<String, Rc<NewlineIndex>> = HashMap::new();
    si.insert(nl.file.clone(), nl.clone());
    let si: SiMap = Rc::new(si);
    let parsed = parse_with_table(tokens, si.clone(), empty_intern_table());
    let module = parsed
        .result
        .module
        .clone()
        .unwrap_or_else(|| panic!("parse failed for {path}"));
    (module, si)
}

fn is_lambda(n: &Rc<Node>) -> bool {
    matches!(*n.expr_data.clone(), ExprData::ExprLambda)
}

fn is_self_call(n: &Rc<Node>, si: &SiMap) -> bool {
    matches!(*n.expr_data.clone(), ExprData::ExprCall { .. })
        && authored_name_at(si.clone(), n.clone()) == SELF_FN
}

/// Does this subtree contain a call to the enclosing recursive function?
fn subtree_has_self_call(n: &Rc<Node>, si: &SiMap) -> bool {
    if is_self_call(n, si) {
        return true;
    }
    n.children.iter().any(|c| subtree_has_self_call(c, si))
}

/// Count the SIBLING traversals (lambdas) within `n` whose body invokes the
/// self-call. A self-calling lambda is counted as one traversal unit and we do
/// NOT descend into it (so a single map-over-map is one traversal, not two) —
/// what we are counting is independent re-traversals of the same input.
fn count_self_calling_traversals(n: &Rc<Node>, si: &SiMap) -> usize {
    if is_lambda(n) && subtree_has_self_call(n, si) {
        return 1;
    }
    n.children
        .iter()
        .map(|c| count_self_calling_traversals(c, si))
        .sum()
}

/// Find the first `match` expression in a subtree (the outer
/// `match texpr.expr_data { ... }` of `resolve_expr_types`).
fn find_match(n: &Rc<Node>) -> Option<&Rc<Node>> {
    if matches!(*n.expr_data.clone(), ExprData::ExprMatch) {
        return Some(n);
    }
    for c in n.children.iter() {
        if let Some(m) = find_match(c) {
            return Some(m);
        }
    }
    None
}

fn resolve_expr_types_body(module: &Rc<Node>, si: &SiMap) -> Rc<Node> {
    let item = module_items(module.clone())
        .iter()
        .find(|it| authored_name_at(si.clone(), (*it).clone()) == SELF_FN)
        .cloned()
        .unwrap_or_else(|| panic!("{SELF_FN} not found in {RESOLVE_DAG}"));
    item.body
        .clone()
        .unwrap_or_else(|| panic!("{SELF_FN} has no body"))
}

/// Returns, per top-level match-arm, the number of sibling self-calling
/// traversals. Any value >= 2 is the redundant-re-traversal anti-pattern.
fn arm_retraversal_counts(content: &str) -> Vec<usize> {
    let (module, si) = parse_module_from_source(RESOLVE_DAG, content);
    let body = resolve_expr_types_body(&module, &si);
    let m = find_match(&body).unwrap_or_else(|| panic!("no match expr in {SELF_FN} body"));
    // children = [scrutinee, arm0, arm1, ...]; arms carry their RHS as children.
    m.children
        .iter()
        .skip(1)
        .map(|arm| count_self_calling_traversals(arm, &si))
        .collect()
}

#[test]
fn resolve_expr_types_has_no_redundant_child_retraversal() {
    let ws = workspace_root();
    let content = std::fs::read_to_string(ws.join(RESOLVE_DAG))
        .unwrap_or_else(|e| panic!("read {RESOLVE_DAG}: {e}"));
    let counts = arm_retraversal_counts(&content);
    assert!(
        !counts.is_empty(),
        "detector found no match-arms — source bridge broke"
    );
    let offenders: Vec<usize> = counts.iter().cloned().filter(|c| *c >= 2).collect();
    assert!(
        offenders.is_empty(),
        "redundant re-traversal in {SELF_FN}: {} arm(s) invoke the recursive \
         self-call in >=2 sibling traversals (O(2^depth) on nested literals). \
         Bind the per-child result once and split it (map(.expr)/flat_map(.diagnostics)). \
         per-arm self-calling-traversal counts: {:?}",
        offenders.len(),
        counts
    );
}

#[test]
fn retraversal_detector_fires_on_real_pre_fix_source() {
    // Teeth check: the detector MUST flag the committed pre-fix source (the real
    // buggy Node), else it has stopped discriminating the class it guards.
    let ws = workspace_root();
    let out = Command::new("git")
        .arg("-C")
        .arg(&ws)
        .arg("show")
        .arg(format!("{PRE_FIX_REV}:{RESOLVE_DAG}"))
        .output()
        .expect("git show pre-fix source");
    if !out.status.success() {
        // Shallow clone / missing history: skip rather than false-fail.
        eprintln!(
            "skip: cannot read {PRE_FIX_REV}:{RESOLVE_DAG} ({})",
            String::from_utf8_lossy(&out.stderr)
        );
        return;
    }
    let pre_fix = String::from_utf8(out.stdout).expect("pre-fix utf8");
    let counts = arm_retraversal_counts(&pre_fix);
    let offenders = counts.iter().filter(|c| **c >= 2).count();
    assert!(
        offenders > 0,
        "detector did NOT flag the known pre-fix double-resolve shape — it has \
         lost its teeth. pre-fix per-arm counts: {counts:?}"
    );
    eprintln!("detector teeth ok: pre-fix flagged {offenders} redundant arm(s): {counts:?}");
}
