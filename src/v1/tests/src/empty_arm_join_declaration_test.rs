use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_std_core::{
    diagnostic_disposition, diagnostic_to_message, DiagnosticGateDisposition,
};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

// A NULLARY ARM'S TYPE CARRIES THE DeclarationRef OF ITS OWNER, STAMPED WHERE THE ARM IS TYPED
// (decided_owner_with_declaration, at expected_decided_variant_type and at the local and global
// variant_reference_inferred_node fallbacks). An `if` joins `List<Int>` with a bare `Empty` by that
// identity, never by the authored name: with one of the three sites unstamped the join refuses
// ("if branches resolve to incompatible types: Container(List,Int) vs Coproduct(FreeMonoid)"), so each
// claim below goes red when a stamp site regresses.
//
// Hand Rust because the join is the v1 seed typechecker's: a .dag claim compiles through the native
// route, which does not run this code. Dissolves with src/v1 at v2 self-host.

fn blocking(src: &str) -> Vec<String> {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources = resolve_imports_transitively_with_source_roots("test.dag", src, &roots);
    let r: Rc<ResolvedPipelineResult> = compile_to_resolved(Rc::new(sources.into()));
    r.diagnostics
        .iter()
        .filter(|d| {
            matches!(
                diagnostic_disposition(d.diagnostic.clone()).gate.as_ref(),
                DiagnosticGateDisposition::GateBlocking
            )
        })
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

const HEAD: &str = "module zz_empty_join\n\
import std.optional { Optional, Absent, Present, optional_present }\n\
import std.types { Int, List }\n\
import std.algebra { Empty }\n\
fn ident<T>(x: T) -> T { x }\n";

#[test]
fn a_bare_empty_arm_joins_a_list_by_its_declaration() {
    let src = format!("{HEAD}fn u(c: Bool, xs: List<Int>) -> List<Int> {{ if c {{ xs }} else {{ ident(Empty) }} }}\n");
    assert_eq!(blocking(&src), Vec::<String>::new());
}

#[test]
fn a_bare_empty_payload_joins_an_optional_list_on_either_side() {
    let t = format!("{HEAD}fn t(c: Bool, xs: List<Int>) -> Optional<List<Int>> {{ if c {{ Present {{ value: xs }} }} else {{ optional_present(value: Empty) }} }}\n");
    let a = format!("{HEAD}fn a(c: Bool, xs: List<Int>) -> Optional<List<Int>> {{ if c {{ optional_present(value: Empty) }} else {{ Present {{ value: xs }} }} }}\n");
    assert_eq!(blocking(&t), Vec::<String>::new());
    assert_eq!(blocking(&a), Vec::<String>::new());
}

// The refusal is live: branches of different types still refuse, so the acceptances above are not vacuous.
#[test]
fn incompatible_branches_still_refuse() {
    let src = format!("{HEAD}fn f() -> Int {{ if true {{ 1 }} else {{ \"x\" }} }}\n");
    assert_eq!(blocking(&src).len(), 1);
}
