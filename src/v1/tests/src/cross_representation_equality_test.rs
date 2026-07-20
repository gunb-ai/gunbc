use std::sync::Arc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpError, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const RECEIPTS_SOURCE: &str = r#"
module test.xrepr

import v2.std.logic { Bool }
import v2.std.nat { Nat, Succ, Zero, nat_add }
import v2.std.algebra { Cons, Empty }

fn reconciles_nat_add_eq_int() -> Bool { nat_add(a: 85, b: 32) == 117 }
fn reconciles_nat_add_1_1_eq_2() -> Bool { nat_add(a: 1, b: 1) == 2 }
fn reconciles_nat_add_noncanon() -> Bool { nat_add(a: 85, b: 32) == nat_add(a: 100, b: 17) }
fn reconciles_list_element() -> Bool { [nat_add(a: 1, b: 1)] == [2] }
fn reconciles_succ_zero_eq_one() -> Bool { (Succ { prev: Zero }) == 1 }
fn reconciles_zero_eq_int() -> Bool { Zero == 0 }
fn reconciles_ne_path_false() -> Bool { nat_add(a: 1, b: 1) != 2 }

fn ok_native_int_eq() -> Bool { (85 + 32) == 117 }
fn ok_nat_add_reflexive() -> Bool { nat_add(a: 85, b: 32) == nat_add(a: 85, b: 32) }
fn ok_list_freemonoid_reconciled() -> Bool {
  [1, 2, 3] == Cons { head: 1, tail: Cons { head: 2, tail: Cons { head: 3, tail: Empty } } }
}

fn diff_int() -> Bool { 1 == 2 }
fn diff_succ_zero_vs_zero() -> Bool { (Succ { prev: Zero }) == Zero }
fn diff_nat_add() -> Bool { nat_add(a: 1, b: 1) == 3 }
fn diff_list_elements() -> Bool { [1, 2] == [1, 3] }
"#;

fn assert_resolved(resolved: &ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "receipts source should resolve cleanly, got {:?} (graph present: {})",
        msgs,
        resolved.graph.is_some(),
    );
}

fn with_receipts_ctx<R>(body: impl FnOnce(&v1_interpreter::InterpContext) -> R) -> R {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources =
        resolve_imports_transitively_with_source_roots("test.dag", RECEIPTS_SOURCE, &roots);
    let resolved = compile_to_resolved(Arc::new(sources.into()));
    assert_resolved(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    body(&ctx)
}

#[test]
fn grounded_numeric_tower_reconciles() {
    with_receipts_ctx(|ctx| {
        for f in [
            "reconciles_nat_add_eq_int",
            "reconciles_nat_add_1_1_eq_2",
            "reconciles_nat_add_noncanon",
            "reconciles_list_element",
            "reconciles_succ_zero_eq_one",
            "reconciles_zero_eq_int",
            "ok_native_int_eq",
            "ok_nat_add_reflexive",
            "ok_list_freemonoid_reconciled",
        ] {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Ok(Value::Bool(true)) => {}
                other => panic!(
                    "{f}: expected Bool(true) — grounding makes the native form equal the \
                     modeled form, so this reconciles; a straddle error or wrong bool here is \
                     the regression this witness guards; got {other:?}"
                ),
            }
        }
        match v1_interpreter::run_in_context(ctx, "reconciles_ne_path_false", false) {
            Ok(Value::Bool(false)) => {}
            other => panic!("reconciles_ne_path_false: expected Bool(false), got {other:?}"),
        }
    });
}

#[test]
fn genuine_inequalities_stay_false_not_errors() {
    with_receipts_ctx(|ctx| {
        for f in [
            "diff_int",
            "diff_succ_zero_vs_zero",
            "diff_nat_add",
            "diff_list_elements",
        ] {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Ok(Value::Bool(false)) => {}
                other => panic!(
                    "{f}: expected Bool(false) — grounding reconciles representations without \
                     turning genuine differences into errors; got {other:?}"
                ),
            }
        }
    });
}

// Discriminating witness for the application-site contract wall.
//
// Before the wall, `call_function_inner` inserted caller argument labels into the call env
// without checking them against the callee's declared parameters, and dropped surplus
// positional args. A mismatched call therefore failed only if the body happened to read a
// parameter the caller had not supplied (surfacing later as `undefined variable: X`), and
// computed silently with wrong bindings when the names overlapped.
//
// The module is named `v2.*` deliberately: `module_skips_direct_call_arg_check` exempts
// `v2.*` and `v1.compiler.*` from compile-time direct-call argument checking, so a mismatched
// call in such a module REACHES the interpreter. That exemption is precisely why the corpus
// had accumulated 33 of these; it is the population this wall guards.
const CALL_CONTRACT_SOURCE: &str = r#"
module v2.test.callcontract

import v2.std.logic { Bool }

fn takes_tag(tag: Bool) -> Bool { tag }
fn takes_unused(_ignored: Bool, keep: Bool) -> Bool { keep }

fn bad_label() -> Bool { takes_tag(nope: true) }
fn surplus_positional() -> Bool { takes_tag(true, true) }

fn ok_label() -> Bool { takes_tag(tag: true) }
fn ok_underscore_idiom() -> Bool { takes_unused(ignored: false, keep: true) }
"#;

fn assert_call_contract_resolved(resolved: &ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "contract source should resolve cleanly, got {:?} (graph present: {})",
        msgs,
        resolved.graph.is_some(),
    );
}

fn with_contract_ctx<R>(body: impl FnOnce(&v1_interpreter::InterpContext) -> R) -> R {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources =
        resolve_imports_transitively_with_source_roots("test.dag", CALL_CONTRACT_SOURCE, &roots);
    let resolved = compile_to_resolved(Arc::new(sources.into()));
    assert_call_contract_resolved(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    body(&ctx)
}

/// RED arm: a mismatched application site refuses, typed and located.
///
/// This is the arm that goes red if the wall is removed — without it `bad_label` binds `nope`
/// into the env, leaves `tag` unbound, and fails much later (or not at all).
#[test]
fn application_site_contract_mismatch_refuses() {
    with_contract_ctx(|ctx| {
        for (f, offending) in [("bad_label", "nope"), ("surplus_positional", "positional")] {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Err(InterpError::CallContractMismatch { callee, detail }) => {
                    assert_eq!(
                        callee, "takes_tag",
                        "{f}: the refusal must locate the CALLEE, not the caller"
                    );
                    assert!(
                        detail.contains(offending),
                        "{f}: detail should name what mismatched, got {detail:?}"
                    );
                }
                other => panic!(
                    "{f}: expected a typed CallContractMismatch at the application site — a \
                     silently-dropped argument or a late `undefined variable` is the fail-open \
                     this witness guards; got {other:?}"
                ),
            }
        }
    });
}

/// GREEN control: valid calls are untouched, INCLUDING the corpus idiom where a
/// deliberately-unused parameter is declared `_ignored` and labelled `ignored` at the call
/// site. Ignoring that idiom produced ~65 false positives; this arm pins it.
#[test]
fn valid_application_sites_are_unaffected() {
    with_contract_ctx(|ctx| {
        for f in ["ok_label", "ok_underscore_idiom"] {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Ok(Value::Bool(true)) => {}
                other => panic!(
                    "{f}: a well-formed call must be unaffected by the contract wall; the \
                     underscore idiom (`_ignored` declared, `ignored` supplied) is NOT a \
                     mismatch because the body cannot read the parameter; got {other:?}"
                ),
            }
        }
    });
}
