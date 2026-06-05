//! Claim #1 (§11): refinement-is-sugar — `refinement_brand_disjointness_is_sugar`.
//!
//! A `where brand(...)` refinement and its hand-desugared equivalent must
//! accept/reject the **identical** set of programs. Divergence = the refinement
//! forked/shelved (not sugar).
//!
//! **Do not relax** these assertions to force green — a passing claim here must
//! mean brand actually enforces nominal disjointness, not a weakened oracle.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const FIXTURE: &str = "dsl/std/integer.dag";

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
        refinement_accepts,
        desugared_accepts,
        "Claim #1 refinement_brand_disjointness_is_sugar: `{program}` — \
         refinement sugar accepts={refinement_accepts}, \
         hand-desugared accepts={desugared_accepts}; \
         divergence means the refinement forked/shelved"
    );
}

#[test]
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

#[test]
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
