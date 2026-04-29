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

/// Category A: `include_str!` references that pull canonical lens bytes.
///
/// Counts each `include_str!` invocation whose argument (possibly across
/// several lines via `concat!`) contains `/lenses/`. The whole-source scan
/// handles the multi-line `concat!(env!("CARGO_MANIFEST_DIR"), "/../lenses/…")`
/// form used in `test_runner.rs`.
fn count_lens_include_str(src: &str) -> usize {
    let mut n = 0;
    let needle = "include_str!";
    let mut rest = src;
    while let Some(idx) = rest.find(needle) {
        let after = &rest[idx + needle.len()..];
        // Scan the next ~256 bytes — enough to cover a multi-line `concat!(...)`
        // argument without straying into the next macro invocation.
        let window_end = after
            .char_indices()
            .nth(256)
            .map(|(i, _)| i)
            .unwrap_or(after.len());
        let window = &after[..window_end];
        if window.contains("/lenses/") {
            n += 1;
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

const EXPECTED_INCLUDE_STR_LENS_BYTES: usize = 2;
const EXPECTED_NAME_EQ_DISPATCH_ARMS: usize = 2;
const EXPECTED_GENERIC_NAME_LOOKUPS: usize = 2;

#[test]
fn canonical_lens_include_str_bridges_pinned() {
    let src = test_runner_source();
    let n = count_lens_include_str(&src);
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
