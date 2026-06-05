//! Claim #1 (§11): reproducible **evidence** for `refinement_brand_disjointness_is_sugar`.
//!
//! The tracked-red **lock** lives in the v4 claim corpus
//! (`src/v4/test/claim/manual/refinement_brand_disjointness_is_sugar.dag` +
//! `workflow/refinement_brand_disjointness_is_sugar_eval.dag`). These Rust tests
//! print load-bearing compile facts and host explicit lock reruns — they must not
//! hard-fail the shared `cargo test -p v3-compiler --test integration` suite.
//!
//! **P5 receipt (INVARIANTS.md §P5 Mechanism (b) — SG-0 `EXPECTED_HAND_AUTHORED_TEST`):**
//! explicit deferral to **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero` (ROADMAP.md:43,63); this file hosts
//! `compile_to_dag` program-matrix evidence until the v4 corpus eval runner is the
//! sole consumer. Dissolves when `.dag` `TestClaim` rows or generated harness
//! coverage own the compile-time matrix without this hand-Rust receipt.

use v3_compiler::compile_to_dag;

/// Did this source compile cleanly (Accepted)?
fn program_compiles(source: &str, file: &str) -> bool {
    compile_to_dag(source, file).is_ok()
}

// ── Refinement (sugar) program matrix ─────────────────────────────────────

const SUGAR_CROSS_BRAND: &str = "\
type BrandA = Int where brand(\"A\")\n\
type BrandB = Int where brand(\"B\")\n\
fn f(x: BrandA) -> Int = x\n\
data b: BrandB = 1\n\
fn prog() -> Int = f(b)\n";

const SUGAR_SAME_BRAND: &str = "\
type BrandA = Int where brand(\"A\")\n\
fn f(x: BrandA) -> Int = x\n\
data a: BrandA = 1\n\
fn prog() -> Int = f(a)\n";

const SUGAR_RAW_LITERAL: &str = "\
type BrandA = Int where brand(\"A\")\n\
fn f(x: BrandA) -> Int = x\n\
fn prog() -> Int = f(1)\n";

// ── Hand-desugared: genuinely distinct nominal wrappers (no brand) ────────

const DESUGAR_CROSS_BRAND: &str = "\
type WrapA { value: Int }\n\
type WrapB { value: Int }\n\
fn f(x: WrapA) -> Int = x.value\n\
fn make_b() -> WrapB = { value: 1 }\n\
fn prog() -> Int = f(make_b())\n";

const DESUGAR_SAME_BRAND: &str = "\
type WrapA { value: Int }\n\
fn f(x: WrapA) -> Int = x.value\n\
fn make_a() -> WrapA = { value: 1 }\n\
fn prog() -> Int = f(make_a())\n";

const DESUGAR_RAW_LITERAL: &str = "\
type WrapA { value: Int }\n\
fn f(x: WrapA) -> Int = x.value\n\
fn prog() -> Int = f(1)\n";

fn refinement_accepts_cross_brand() -> bool {
    program_compiles(SUGAR_CROSS_BRAND, "claim1_sugar_cross_brand.v3")
}

fn refinement_accepts_same_brand() -> bool {
    program_compiles(SUGAR_SAME_BRAND, "claim1_sugar_same_brand.v3")
}

fn refinement_accepts_raw_literal() -> bool {
    program_compiles(SUGAR_RAW_LITERAL, "claim1_sugar_raw_literal.v3")
}

fn desugared_accepts_cross_brand() -> bool {
    program_compiles(DESUGAR_CROSS_BRAND, "claim1_desugar_cross_brand.v3")
}

fn desugared_accepts_same_brand() -> bool {
    program_compiles(DESUGAR_SAME_BRAND, "claim1_desugar_same_brand.v3")
}

fn desugared_accepts_raw_literal() -> bool {
    program_compiles(DESUGAR_RAW_LITERAL, "claim1_desugar_raw_literal.v3")
}

/// Claim #1 lock: refinement sugar and hand-desugared must agree on every
/// program in the matrix. Mismatch = fork/shelve, not sugar.
fn assert_refinement_matches_desugared(
    program: &str,
    refinement_accepts: bool,
    desugared_accepts: bool,
) {
    assert_eq!(
        refinement_accepts, desugared_accepts,
        "Claim #1 refinement_brand_disjointness_is_sugar: `{program}` — \
         refinement sugar accepts={refinement_accepts}, \
         hand-desugared accepts={desugared_accepts}; \
         divergence means the refinement forked/shelved"
    );
}

/// Claim #1 lock — RED until brand enforces nominal disjointness; run explicitly
/// to verify: `cargo test -p v3-compiler claim1_refinement_brand_disjointness_cross_brand -- --ignored`
#[test]
#[ignore = "claim #1 lock — RED until brand enforces nominal disjointness; tracked in v4 claim corpus"]
fn claim1_refinement_brand_disjointness_cross_brand_matches_desugared() {
    assert_refinement_matches_desugared(
        "cross-brand call f(BrandB) where f: BrandA -> Int",
        refinement_accepts_cross_brand(),
        desugared_accepts_cross_brand(),
    );
}

#[test]
fn claim1_refinement_brand_disjointness_same_brand_matches_desugared() {
    assert_refinement_matches_desugared(
        "same-brand call f(BrandA) where f: BrandA -> Int",
        refinement_accepts_same_brand(),
        desugared_accepts_same_brand(),
    );
}

/// Claim #1 lock — RED until brand enforces nominal disjointness; run explicitly
/// to verify: `cargo test -p v3-compiler claim1_refinement_brand_disjointness_raw_literal -- --ignored`
#[test]
#[ignore = "claim #1 lock — RED until brand enforces nominal disjointness; tracked in v4 claim corpus"]
fn claim1_refinement_brand_disjointness_raw_literal_matches_desugared() {
    assert_refinement_matches_desugared(
        "raw Int literal call f(1) where f: BrandA -> Int",
        refinement_accepts_raw_literal(),
        desugared_accepts_raw_literal(),
    );
}

/// Empirical verdict reporter — always runs, prints load-bearing facts for
/// the operator realism gate (does not affect pass/fail of the locks above).
#[test]
fn claim1_refinement_brand_disjointness_empirical_verdict() {
    eprintln!("=== Claim #1 empirical verdict (refinement_brand_disjointness_is_sugar) ===");
    eprintln!(
        "cross-brand:  refinement_accepts={} desugared_accepts={}",
        refinement_accepts_cross_brand(),
        desugared_accepts_cross_brand()
    );
    eprintln!(
        "same-brand:   refinement_accepts={} desugared_accepts={}",
        refinement_accepts_same_brand(),
        desugared_accepts_same_brand()
    );
    eprintln!(
        "raw-literal:  refinement_accepts={} desugared_accepts={}",
        refinement_accepts_raw_literal(),
        desugared_accepts_raw_literal()
    );
    eprintln!(
        "brand_enforces_disjointness={}",
        !refinement_accepts_cross_brand() && desugared_accepts_cross_brand() == false
            || refinement_accepts_cross_brand() == desugared_accepts_cross_brand()
    );
}
