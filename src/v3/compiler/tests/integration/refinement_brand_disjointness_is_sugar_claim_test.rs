//! Claim #1 (§11): reproducible **measurement** for `refinement_brand_disjointness_is_sugar`.
//!
//! **Authoritative lock** — v3 `compile_to_dag` on real branded source. The v4 frontend
//! is NOT in the measurement loop today (see
//! `src/v4/test/claim/manual/refinement_brand_disjointness_is_sugar_scope.dag`).
//!
//! Tracked-red rows pin live measured `(refinement_accepts, desugared_accepts)` gaps while
//! sugar and desugared disagree. **DISSOLUTION:** when a discriminating row goes green,
//! flip `Some((true, false))` → `None` so the row becomes a permanent regression guard —
//! a reintroduced gap then fails (`control row must agree`).
//!
//! **P5 receipt (INVARIANTS.md §P5 Mechanism (b) — SG-0 `EXPECTED_HAND_AUTHORED_TEST`):**
//! explicit deferral to **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero` (ROADMAP.md:43,63); this file hosts
//! `compile_to_dag` program-matrix measurement until a v4 corpus compile primitive can
//! observe branded-program accept/reject directly.

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

/// Brand enforces nominal disjointness when refinement **rejects** the cross-brand call.
fn brand_enforces_disjointness() -> bool {
    !refinement_accepts_cross_brand()
}

/// Claim #1: sugar and desugared must agree, or the row must match a tracked-red gap.
///
/// `tracked_red_gap = None` — control / regression guard: disagreement fails closed.
/// `tracked_red_gap = Some((r, d))` — pins the live measured gap while shelved.
fn assert_claim1_agreement_or_tracked_gap(
    program: &str,
    refinement_accepts: bool,
    desugared_accepts: bool,
    tracked_red_gap: Option<(bool, bool)>,
) {
    if refinement_accepts == desugared_accepts {
        return;
    }
    let expected = tracked_red_gap.unwrap_or_else(|| {
        panic!(
            "Claim #1 regression guard: `{program}` — sugar/desugared disagree \
             (refinement={refinement_accepts}, desugared={desugared_accepts}); \
             control row must agree"
        )
    });
    assert_eq!(
        (refinement_accepts, desugared_accepts),
        expected,
        "Claim #1 tracked-red: unexpected measured gap for `{program}`"
    );
}

#[test]
fn claim1_refinement_brand_disjointness_cross_brand_matches_desugared() {
    // DISSOLUTION: when this row goes green, change Some((true, false)) -> None.
    assert_claim1_agreement_or_tracked_gap(
        "cross-brand call f(BrandB) where f: BrandA -> Int",
        refinement_accepts_cross_brand(),
        desugared_accepts_cross_brand(),
        Some((true, false)),
    );
}

#[test]
fn claim1_refinement_brand_disjointness_same_brand_matches_desugared() {
    assert_claim1_agreement_or_tracked_gap(
        "same-brand call f(BrandA) where f: BrandA -> Int",
        refinement_accepts_same_brand(),
        desugared_accepts_same_brand(),
        None,
    );
}

#[test]
fn claim1_refinement_brand_disjointness_raw_literal_matches_desugared() {
    // DISSOLUTION: when this row goes green, change Some((true, false)) -> None.
    assert_claim1_agreement_or_tracked_gap(
        "raw Int literal call f(1) where f: BrandA -> Int",
        refinement_accepts_raw_literal(),
        desugared_accepts_raw_literal(),
        Some((true, false)),
    );
}

#[test]
fn claim1_refinement_brand_disjointness_brand_enforces_disjointness() {
    // DISSOLUTION: when brand desugars, change assert_eq!(..., false) -> assert!(brand_enforces_disjointness()).
    assert_eq!(
        brand_enforces_disjointness(),
        false,
        "Claim #1: brand does not enforce nominal disjointness while shelved \
         (refinement accepts cross-brand)"
    );
}
