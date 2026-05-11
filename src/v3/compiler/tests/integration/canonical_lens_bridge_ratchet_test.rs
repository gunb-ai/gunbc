//! Zero-residual ratchet for the retired canonical lens-name dispatch bridge in `test_runner.rs`.
//!
//! Pins the retired bridge surface at zero so any reintroduction fails CI.
//!
//! The bridge was described in `docs/briefs/r2-pb-canonical-lens-bridge-disposition.md`
//! and closed by `docs/briefs/r3-pb-bridge-canonical-lens-name-dispatch-closure.md`
//! (gate `bridge_canonical_lens_name_dispatch_retired`, R3 T-Bridge-Retirement
//! distributed bridge #3).
//!
//! Three categories are pinned:
//!   A. `include_str!("…/lenses/<name>.dag")` calls.
//!   B. `lens_decl.name.as_deref() == Some("<name>")` string-name dispatch arms.
//!   C. Generic `lens_decl.name.as_deref()` name-keyed lookups (ones not
//!      already counted in B).
//!   D. Helper-shaped `DeclarationRef` to name-resolution dispatch for
//!      canonical lens identity.

use crate::common::strip_rust_comments;
use std::fs;
use std::path::PathBuf;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::{compile_to_dag, CompileError};

fn test_runner_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test_runner.rs");
    fs::read_to_string(&path).expect("read test_runner.rs source")
}

fn compile_clean(source: &str, file: &str) -> v3_compiler::dag::Dag {
    match compile_to_dag(source, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => panic!(
            "{file} should compile cleanly, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(err) => panic!("{file} should compile cleanly, got {err:?}"),
    }
}

/// Category A: canonical-lens `include_str!` bridge constants in
/// `test_runner.rs`. Counts pub-const definitions of the shape
/// `pub const R1_CANONICAL_<NAME>_LENS: &str = include_str!` — any new
/// canonical-lens `include_str!` const with this naming is caught by the
/// ratchet, not just the two current names. Lines/blocks of comment text
/// are stripped before scanning so docstring examples can't fabricate or
/// mask growth.
///
/// Other `include_str!`-of-`/lenses/` cases (e.g. `INFER_HELPERS_SOURCE`)
/// are not part of this bridge surface — they are not consumed via
/// name-dispatch from a `lens_decl.name == …` arm and do not match the
/// `R1_CANONICAL_*_LENS` naming.
fn count_canonical_lens_include_str_bridges(src: &str) -> usize {
    // Anchor on live syntax: scan tokens between "pub const " and "=".
    let mut n = 0;
    let mut rest = src;
    let needle = "pub const ";
    while let Some(idx) = rest.find(needle) {
        let after = &rest[idx + needle.len()..];
        // Bound the inspection window to the const's signature/initializer.
        let window_end = after.find(';').map(|p| p + 1).unwrap_or(after.len());
        let window = &after[..window_end];
        // Match the canonical-lens const naming + include_str! initializer.
        if let Some(eq_pos) = window.find('=') {
            let (name_part, init_part) = window.split_at(eq_pos);
            let name = name_part.trim().trim_end_matches(": &str").trim();
            if name.starts_with("R1_CANONICAL_")
                && name.ends_with("_LENS")
                && init_part.contains("include_str!")
            {
                n += 1;
            }
        }
        rest = after;
    }
    n
}

/// Category B: `lens_decl.name.as_deref() == Some("...")` string-name
/// dispatch arms (the literal-equality comparisons).
fn count_lens_name_eq_dispatches(src: &str) -> usize {
    let mut n = 0;
    let needle = "lens_decl.name.as_deref()";
    let mut rest = src;
    while let Some(idx) = rest.find(needle) {
        let after = &rest[idx + needle.len()..];
        // Count only the equality-to-Some("...") pattern, not the generic
        // `if let Some(name) = lens_decl.name.as_deref()` form.
        let trimmed = after.trim_start();
        if trimmed.starts_with("== Some(\"") {
            n += 1;
        }
        rest = after;
    }
    n
}

/// Category C: generic `if let Some(name) = lens_decl.name.as_deref()`
/// name-keyed lookups.
fn count_lens_name_lookups(src: &str) -> usize {
    src.matches("if let Some(name) = lens_decl.name.as_deref()")
        .count()
}

/// Category D: helper-shaped `DeclarationRef` identity checks that resolve
/// the referenced declaration back to a spelling such as `cost_of`.
fn count_declaration_ref_name_dispatch_helpers(src: &str) -> usize {
    src.matches("declaration_ref_resolves_to_name").count()
        + src.matches("declaration_by_name(\"cost_of\")").count()
}

const EXPECTED_INCLUDE_STR_LENS_BYTES: usize = 0;
const EXPECTED_NAME_EQ_DISPATCH_ARMS: usize = 0;
const EXPECTED_GENERIC_NAME_LOOKUPS: usize = 0;
const EXPECTED_DECLARATION_REF_NAME_DISPATCH_HELPERS: usize = 0;

#[test]
fn canonical_lens_include_str_bridges_pinned() {
    let src = strip_rust_comments(&test_runner_source());
    let n = count_canonical_lens_include_str_bridges(&src);
    assert_eq!(
        n, EXPECTED_INCLUDE_STR_LENS_BYTES,
        "test_runner.rs canonical-lens `include_str!` count drifted: \
         expected {EXPECTED_INCLUDE_STR_LENS_BYTES}, found {n}. \
         The canonical lens-name dispatch bridge is retired; any nonzero \
         count reintroduces a production canonical-lens byte authority. \
         Keep lens execution routed through the resolved fixture \
         DeclarationRef, not through runner-owned `include_str!` bytes."
    );
}

#[test]
fn canonical_lens_name_dispatch_arms_pinned() {
    let src = strip_rust_comments(&test_runner_source());
    let n = count_lens_name_eq_dispatches(&src);
    assert_eq!(
        n, EXPECTED_NAME_EQ_DISPATCH_ARMS,
        "test_runner.rs `lens_decl.name.as_deref() == Some(\"…\")` arm \
         count drifted: expected {EXPECTED_NAME_EQ_DISPATCH_ARMS}, found \
         {n}. The canonical lens-name dispatch bridge is retired; any \
         nonzero count reintroduces name-keyed semantic dispatch on lens \
         identity."
    );
}

#[test]
fn canonical_lens_generic_name_lookups_pinned() {
    let src = strip_rust_comments(&test_runner_source());
    let n = count_lens_name_lookups(&src);
    assert_eq!(
        n, EXPECTED_GENERIC_NAME_LOOKUPS,
        "test_runner.rs generic `lens_decl.name.as_deref()` name-keyed \
         lookup count drifted: expected {EXPECTED_GENERIC_NAME_LOOKUPS}, \
         found {n}. The canonical lens-name dispatch bridge is retired; \
         lens body selection must not route through declaration spelling."
    );
}

#[test]
fn canonical_lens_declaration_ref_name_dispatch_helpers_pinned() {
    let src = strip_rust_comments(&test_runner_source());
    let n = count_declaration_ref_name_dispatch_helpers(&src);
    assert_eq!(
        n, EXPECTED_DECLARATION_REF_NAME_DISPATCH_HELPERS,
        "test_runner.rs DeclarationRef-to-name lens dispatch helper count \
         drifted: expected {EXPECTED_DECLARATION_REF_NAME_DISPATCH_HELPERS}, \
         found {n}. The canonical lens-name dispatch bridge is retired; \
         semantic dispatch must not route by resolving a lens DeclarationRef \
         back to declaration spelling."
    );
}

#[test]
fn bridge_retirement_demonstration() {
    // R3 gate #69: execute one retired-bridge replacement path. The
    // `lens_output_equals_gate` fixture carries `LensOutputEquals(
    // named_function_count, ProgramInput {}, expected)`; the runner resolves
    // the lens as a `DeclarationRef` in the fixture Dag and applies it through
    // production `apply_lens_declaration`, not via canonical-lens
    // `include_str!` bytes or `lens_decl.name` dispatch.
    let src = strip_rust_comments(&test_runner_source());
    assert_eq!(count_canonical_lens_include_str_bridges(&src), 0);
    assert_eq!(count_lens_name_eq_dispatches(&src), 0);
    assert_eq!(count_lens_name_lookups(&src), 0);
    assert_eq!(count_declaration_ref_name_dispatch_helpers(&src), 0);

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let gate = manifest_dir.join("tests/fixtures/r1_gates.dag");
    let source = fs::read_to_string(&gate).unwrap_or_else(|err| panic!("read {gate:?}: {err}"));
    let dag = compile_clean(&source, "src/v3/compiler/tests/fixtures/r1_gates.dag");
    let results = TestRunner::new(&dag).run_suite("r1_lens_output_equals_suite");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].claim_name, "lens_output_equals_gate");
    assert_eq!(results[0].result, ClaimResult::Pass);
}

#[test]
fn count_canonical_lens_catches_new_r1_canonical_const() {
    // A future bridge addition under a different name MUST be caught by
    // the ratchet — the prior exact-name predicate would have returned 2,
    // missing the third bridge.
    let src = r#"
pub const R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS: &str = include_str!("a.dag");
pub const R1_CANONICAL_COMPLEXITY_LENS: &str = include_str!("b.dag");
pub const R1_CANONICAL_FUTURE_BRIDGE_LENS: &str = include_str!("c.dag");
pub const NOT_A_LENS_BRIDGE: &str = include_str!("d.dag");
    "#;
    assert_eq!(count_canonical_lens_include_str_bridges(src), 3);
}
