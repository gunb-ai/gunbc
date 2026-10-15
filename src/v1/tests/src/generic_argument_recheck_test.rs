//! Structural controls for sparse generic-argument recheck
//! (`v1.compiler.infer` `argument_needs_generic_recheck`).
//!
//! The nested left-bound chain is qualitative: T is bound before each lambda, so the
//! missing-context set is empty and replay is not demanded. The right-bound case is the
//! discriminating red if that demand is omitted.

use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::compile_to_resolved;
use v1_compiler::v1_compiler_infer::argument_needs_generic_recheck;
use v1_compiler::v1_rt;
use v1_compiler::v1_std_core;

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

fn empty_subst() -> Rc<im::HashMap<String, Rc<v1_std_core::Node>>> {
    v1_rt::rc_empty_map()
}

fn subst_t() -> Rc<im::HashMap<String, Rc<v1_std_core::Node>>> {
    v1_rt::rc_map_insert(empty_subst(), "T".to_string(), v1_std_core::error_type())
}

#[test]
fn left_bound_empty_missing_set_does_not_demand_recheck() {
    assert!(!argument_needs_generic_recheck(
        Rc::new(im::vector![]),
        subst_t()
    ));
}

#[test]
fn right_bound_missing_generic_demands_recheck_once_bound() {
    assert!(argument_needs_generic_recheck(
        Rc::new(im::vector!["T".to_string()]),
        subst_t()
    ));
}

#[test]
fn unbound_missing_generic_does_not_demand_recheck() {
    assert!(!argument_needs_generic_recheck(
        Rc::new(im::vector!["T".to_string()]),
        empty_subst()
    ));
}

const NESTED_LEFT: &str = r#"
module test.gar_nested_left_rs
import std.types { Int }
fn gar_bind<T, R>(o: T, f: fn(T) -> R) -> R { f(o) }
fn probe() -> Int {
  gar_bind(o: 1, f: fn(a) {
    gar_bind(o: a, f: fn(b) {
      gar_bind(o: b, f: fn(c) { c })
    })
  })
}
"#;

#[test]
fn nested_left_bound_chain_still_infers() {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources = resolve_imports_transitively_with_source_roots(
        "test.gar_nested_left_rs.dag",
        NESTED_LEFT,
        &roots,
    );
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .filter(|d| cli_run::compile_clean_diagnostic_is_hard(d))
        .map(|d| v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "nested left-bound bind chain must infer, got {msgs:?}"
    );
}
