//! Ratchet for the canonical lens-name dispatch bridge in `test_runner.rs`.
//!
//! Pins the current bridge surface so any growth fails CI, and any reduction
//! can lower the pin in the same PR. Per `feedback_ratchet_only_down`: never
//! raise these constants.
//!
//! The bridge is described in `docs/briefs/r2-pb-canonical-lens-bridge-disposition.md`
//! (gate `bridge_canonical_lens_name_dispatch_retired`, R3 T-Bridge-Retirement
//! distributed bridge #3). Full retirement is gated on PB-Runtime
//! interpreter-as-data OR a typed lens-registry carrier substrate-introduction;
//! both are outside this PR's scope.
//!
//! Three categories are pinned:
//!   A. `include_str!("…/lenses/<name>.dag")` calls.
//!   B. `lens_decl.name.as_deref() == Some("<name>")` string-name dispatch arms.
//!   C. Generic `lens_decl.name.as_deref()` name-keyed lookups (ones not
//!      already counted in B).

use std::fs;
use std::path::PathBuf;

fn test_runner_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test_runner.rs");
    fs::read_to_string(&path).expect("read test_runner.rs source")
}

/// Category A: canonical-lens `include_str!` bridge constants in
/// `test_runner.rs`. Counts the named pub-const definitions that pull
/// canonical lens bytes for `apply_lens_declaration` cross-`Dag`
/// reflection-coherence (P2). Other `include_str!`-of-`/lenses/` cases
/// (e.g. `INFER_HELPERS_SOURCE`) are not part of this bridge surface;
/// they are not consumed via name-dispatch from a `lens_decl.name == …`
/// arm and are out of scope.
fn count_canonical_lens_include_str_bridges(src: &str) -> usize {
    let mut n = 0;
    if src.contains("pub const R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS: &str = include_str!") {
        n += 1;
    }
    if src.contains("pub const R1_CANONICAL_COMPLEXITY_LENS: &str = include_str!") {
        n += 1;
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

const EXPECTED_INCLUDE_STR_LENS_BYTES: usize = 2;
const EXPECTED_NAME_EQ_DISPATCH_ARMS: usize = 2;
const EXPECTED_GENERIC_NAME_LOOKUPS: usize = 2;

#[test]
fn canonical_lens_include_str_bridges_pinned() {
    let src = test_runner_source();
    let n = count_canonical_lens_include_str_bridges(&src);
    assert_eq!(
        n, EXPECTED_INCLUDE_STR_LENS_BYTES,
        "test_runner.rs canonical-lens `include_str!` count drifted: \
         expected {EXPECTED_INCLUDE_STR_LENS_BYTES}, found {n}. \
         If this is a *reduction* (bridge retired), lower the constant in \
         this file in the same PR. If this is *growth*, that is forbidden \
         per the disposition in \
         docs/briefs/r2-pb-canonical-lens-bridge-disposition.md — split \
         a structural lens-registry carrier brief instead of adding another \
         `include_str!` of canonical lens bytes."
    );
}

#[test]
fn canonical_lens_name_dispatch_arms_pinned() {
    let src = test_runner_source();
    let n = count_lens_name_eq_dispatches(&src);
    assert_eq!(
        n, EXPECTED_NAME_EQ_DISPATCH_ARMS,
        "test_runner.rs `lens_decl.name.as_deref() == Some(\"…\")` arm \
         count drifted: expected {EXPECTED_NAME_EQ_DISPATCH_ARMS}, found \
         {n}. If this is a *reduction*, lower the constant. If this is \
         *growth*, that is forbidden per the disposition — name-keyed \
         dispatch on lens identity must be retired via PB-Runtime \
         interpreter-as-data or a typed lens-registry carrier, not \
         extended."
    );
}

#[test]
fn canonical_lens_generic_name_lookups_pinned() {
    let src = test_runner_source();
    let n = count_lens_name_lookups(&src);
    assert_eq!(
        n, EXPECTED_GENERIC_NAME_LOOKUPS,
        "test_runner.rs generic `lens_decl.name.as_deref()` name-keyed \
         lookup count drifted: expected {EXPECTED_GENERIC_NAME_LOOKUPS}, \
         found {n}. Same rule as the equality-arm ratchet: ratchet only \
         down."
    );
}
