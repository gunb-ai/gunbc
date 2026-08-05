//! R1 closeout item TWO: discriminating bidirectional compile-fail evidence for the
//! roster<->handler-arm bijection `interpreter_dispatch_authority.rs` documents but does not
//! exercise. The construction wall lives in real macro-expanded Rust
//! (`v1_bridge_dispatch!`/`v1_bridge_family_arms!` in `v1_interpreter.rs`): a per-family generated
//! enum (one variant per roster row) is matched with NO wildcard arm against a hand-authored
//! `arm "<id>" { ... } => body` list, via `$arm_macro!($id)` where `$arm_macro` is itself generated
//! from the roster. Neither direction of the bijection is representable as a `.dag`-level check --
//! `.dag` has no ingest grammar for arbitrary Rust source, so reading v1_interpreter.rs text to
//! re-derive what rustc's own exhaustiveness/macro-expansion checker already decides would be a
//! second, fragile authority for a fact rustc already computes (DESIGN.md §6 "enforce with lenses,
//! not grep"; §3 single authority). So this fixture reproduces the exact shape -- generated enum +
//! generated arm-token macro + hand-authored match with no wildcard -- in an isolated temp-dir crate
//! and drives a real `cargo build`, following the one existing precedent for this pattern in the
//! tree (`cssl_seed_linked_closure_assembly.rs`'s `deliberate-red` cargo-subprocess controls). It
//! does not touch the real dispatch generator or `v1_interpreter.rs`.
//!
//! Not `#[ignore]`d, unlike the sibling `interpreter_dispatch_bijection_real_roster_red.rs`
//! (review 48433): that file's tests clone the real workspace and regenerate through the real
//! `gunbc` pipeline before a real `cargo build` of `v1-compiler` (~192s/test), so they are
//! `#[ignore]`d for cost, not because a cargo subprocess must never sit in per-PR CI discovery --
//! neither `cargo test` nor `cargo nextest` runs in per-PR CI at all (nextest removed 2026-07-11,
//! DESIGN.md `commit_gate_rust_suite_removed_disposition`; the local suite is the only consumer
//! either way). The fixtures here are a dependency-free temp-dir crate (no crates.io download, no
//! network) and build in a few seconds each, so leaving them in the default local `cargo test`
//! run is cheap and keeps the claim-A discriminator exercised without opt-in.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn temp_fixture_root(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "gunbc_dispatch_bijection_{name}_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).expect("temp root");
    base
}

fn write_fixture_crate(root: &PathBuf, lib_rs: &str) {
    let src = root.join("src");
    fs::create_dir_all(&src).expect("src dir");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"dispatch_bijection_red\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .expect("Cargo.toml");
    fs::write(src.join("lib.rs"), lib_rs).expect("lib.rs");
}

fn cargo_build_status(root: &PathBuf) -> std::process::Output {
    Command::new("cargo")
        .args(["build", "--lib"])
        .current_dir(root)
        .env("RUSTC_WRAPPER", "")
        .output()
        .expect("cargo build")
}

/// Positive control: the exact shape (generated 3-variant enum, generated arm macro, hand-authored
/// match, no wildcard) with the bijection intact compiles clean. Proves the fixture is
/// discriminating -- the two REDs below fail from the deliberate gap, not from the shape itself.
#[test]
fn w_control_bijection_intact_compiles() {
    let root = temp_fixture_root("control");
    write_fixture_crate(
        &root,
        r#"
// Roster-generated: one enum variant per roster row.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Arm { A, B, C }

// Roster-generated: token -> variant, one rule per roster row.
macro_rules! arm_macro {
    (A) => { Arm::A };
    (B) => { Arm::B };
    (C) => { Arm::C };
}

// Hand-authored family block: one `arm` entry per roster row, matched with NO wildcard.
pub fn dispatch(arm: Arm) -> &'static str {
    match arm {
        arm_macro!(A) => "a",
        arm_macro!(B) => "b",
        arm_macro!(C) => "c",
    }
}
"#,
    );
    let out = cargo_build_status(&root);
    assert!(
        out.status.success(),
        "control must compile clean; stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// RED — item TWO, direction 1: a roster row (generated enum variant `C`) with no corresponding
/// hand-authored `arm` block. The inner match has no pattern for `Arm::C` and carries no wildcard,
/// so rustc refuses with E0004 non-exhaustive-patterns. This is the real construction wall firing,
/// not a stub: deleting the `arm_macro!(C)` line is the only way to make it compile, and doing so
/// reproduces the exact defect item ONE's wall exists to make unwritable.
#[test]
fn w_red_roster_row_with_no_handler_arm_refuses_compile() {
    let root = temp_fixture_root("missing_handler");
    write_fixture_crate(
        &root,
        r#"
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Arm { A, B, C }

macro_rules! arm_macro {
    (A) => { Arm::A };
    (B) => { Arm::B };
    (C) => { Arm::C };
}

// Hand-authored family block is missing the `arm` entry for C (the defect item ONE's wall forbids).
pub fn dispatch(arm: Arm) -> &'static str {
    match arm {
        arm_macro!(A) => "a",
        arm_macro!(B) => "b",
    }
}
"#,
    );
    let out = cargo_build_status(&root);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "deliberate-red: a roster row with no handler arm must refuse cargo build"
    );
    assert!(
        stderr.contains("E0004") || stderr.contains("non-exhaustive"),
        "must refuse specifically on non-exhaustive-match, not some other defect; stderr:\n{stderr}"
    );
}

/// RED — item TWO, direction 2: a hand-authored `arm` block whose identity has no corresponding
/// roster row (the generated arm-token macro has no rule for it). The macro invocation
/// `arm_macro!(D)` itself fails to expand, so rustc refuses at the macro-invocation site rather than
/// at the match. This is the mirror defect: an orphan handler nobody's roster row names.
#[test]
fn w_red_handler_arm_with_no_roster_row_refuses_compile() {
    let root = temp_fixture_root("orphan_handler");
    write_fixture_crate(
        &root,
        r#"
// Roster only ever produced A and B; no roster row named D.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Arm { A, B }

macro_rules! arm_macro {
    (A) => { Arm::A };
    (B) => { Arm::B };
}

// Hand-authored family block carries an orphan `arm "D"` entry no roster row grounds.
pub fn dispatch(arm: Arm) -> &'static str {
    match arm {
        arm_macro!(A) => "a",
        arm_macro!(B) => "b",
        arm_macro!(D) => "d",
    }
}
"#,
    );
    let out = cargo_build_status(&root);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "deliberate-red: a handler arm with no roster row must refuse cargo build"
    );
    assert!(
        stderr.contains("no rules expected") || stderr.contains("unexpected token"),
        "must refuse specifically on macro-invocation failure for the orphan identity, not some other defect; stderr:\n{stderr}"
    );
}
