//! ARM 3 executed receipt (findings-only; no repair — see
//! docs/plans/compiler-guarantee-recovery-gap-analysis.md §11 item 1a).
//!
//! `extdeps.uri` `UriValidatedScalar` is declared `sole_constructor` in the `.dag`
//! source with a single field `admitted_cp: Int`; its only sanctioned mint is
//! `uri_validated_scalar_construction`, whose fixed law refuses surrogate code
//! points (`UriValidatedScalarSurrogateRefused`) and out-of-range code points
//! (`UriValidatedScalarOutOfRangeRefused`). `uri_percent_encode_admitted_scalar_wire`
//! accepts `UriValidatedScalar` only, so a forged scalar reaches percent-encoding
//! output.
//!
//! The `.dag` `sole_constructor` wall refuses a cross-module record literal for this
//! carrier at compile time (`SoleConstructorViolation`, see
//! `dag/test/claim/guarantee_probe_corpus_witness_test.dag`
//! `sole_ctor_forged_source`). This receipt establishes, by execution against the
//! emitted stage0 Rust mirror, that the wall does not survive emission: the carrier
//! is emitted as `pub struct UriValidatedScalar { pub admitted_cp: i64 }` deriving
//! `serde::Serialize, serde::Deserialize` (`extdeps_uri.rs`), so it is freely
//! constructible both as a Rust struct literal (compile-time, no constructor call)
//! and via `serde_json::from_value` — for every value the `.dag` mint refuses, not
//! only the admitted ones.
//!
//! KNOWN HOLE, asserted as it stands today — this test documents the gap, it does
//! not close it. The positive control (`cp = 65`) controls the *mint's* behavior,
//! not serde's: it is admitted on both sides, so on its own it cannot show this
//! harness is capable of observing an `Err`. `serde_shape_control_rejects_malformed_input`
//! is the discriminator that closes that hole — a value serde itself must reject
//! regardless of the `admitted_cp` predicate (wrong JSON type, and the field
//! missing entirely) — so every "ADMITTED" verdict elsewhere in this file is
//! load-bearing: the harness has demonstrated it can and does report `Err`.
//! Repair (omit `Deserialize`, emit a validating `Deserialize`, or seal the field
//! behind `TryFrom`) is a separate design decision per the ARM 3 brief and is not
//! made here. DISSOLVE-ON: `sole_constructor` completeness audit
//! (compiler-guarantee-recovery-gap-analysis.md §11 item 1a) extends the
//! unforgeable-construction wall through Rust emission for sealed carriers; at
//! that point the `serde`/struct-literal assertions below flip from "admits" to
//! "refuses" and this receipt becomes the regression control for that repair.
//! **THAT FAILURE IS THE SUCCESS SIGNAL, NOT A REGRESSION IN THIS RECEIPT.** Per
//! DESIGN.md §4b, a climb dissolves the obsolete production machinery it
//! obsoletes but never the evidence: when a repair lands and these assertions
//! start failing, the correct response is to flip the assertions from "admits" to
//! "refuses" so this file becomes the permanent regression control — never to
//! delete it.
//!
//! **UNENROLLED, by standing operator ruling, not oversight.** This is a Rust
//! test; the Rust suite was removed from CI 2026-07-11 (`gunbc.commit_workflow`
//! `commit_gate_rust_suite_removed_disposition`) and runs locally only — no
//! `.github/workflows/*` invokes `cargo test`. These assertions are executed
//! evidence for this PR's receipt, not a guard on any future PR or on main; this
//! PR does not re-add `cargo test` to CI, since reversing that ruling is not this
//! change's to make.

use v1_compiler::extdeps_uri::{
    uri_validated_scalar_construction, UriValidatedScalar, UriValidatedScalarConstruction,
};

const SURROGATE_CP: i64 = 55_296; // 0xD800, first low surrogate
const NEGATIVE_CP: i64 = -1;
const ABOVE_MAX_CP: i64 = 1_114_112; // 0x110000, one past the last scalar value
const ADMITTED_CP: i64 = 65; // 'A' -- positive control, admitted by the .dag mint

#[test]
fn dag_mint_refuses_surrogate_negative_and_above_max_admits_control() {
    assert!(matches!(
        *uri_validated_scalar_construction(SURROGATE_CP),
        UriValidatedScalarConstruction::UriValidatedScalarSurrogateRefused { cp } if cp == SURROGATE_CP
    ));
    assert!(matches!(
        *uri_validated_scalar_construction(NEGATIVE_CP),
        UriValidatedScalarConstruction::UriValidatedScalarOutOfRangeRefused { cp } if cp == NEGATIVE_CP
    ));
    assert!(matches!(
        *uri_validated_scalar_construction(ABOVE_MAX_CP),
        UriValidatedScalarConstruction::UriValidatedScalarOutOfRangeRefused { cp } if cp == ABOVE_MAX_CP
    ));
    assert!(matches!(
        *uri_validated_scalar_construction(ADMITTED_CP),
        UriValidatedScalarConstruction::UriValidatedScalarConstructed(ref s)
            if s.admitted_cp == ADMITTED_CP
    ));
}

fn serde_admits(cp: i64) -> bool {
    let value = serde_json::json!({ "admitted_cp": cp });
    serde_json::from_value::<UriValidatedScalar>(value).is_ok()
}

#[test]
fn known_hole_serde_admits_every_value_the_dag_mint_refuses() {
    assert!(
        serde_admits(SURROGATE_CP),
        "known hole: serde forges the surrogate scalar the .dag mint refuses"
    );
    assert!(
        serde_admits(NEGATIVE_CP),
        "known hole: serde forges the negative scalar the .dag mint refuses"
    );
    assert!(
        serde_admits(ABOVE_MAX_CP),
        "known hole: serde forges the above-max scalar the .dag mint refuses"
    );
    // Positive control: must also succeed, or the harness is void.
    assert!(
        serde_admits(ADMITTED_CP),
        "positive control: serde must admit a value the .dag mint also admits"
    );
}

#[test]
fn serde_shape_control_rejects_malformed_input() {
    // Discriminator: values serde itself must reject, independent of the
    // admitted_cp predicate entirely -- a wrong JSON type and a missing
    // required field. Without this, every ADMITTED verdict above is
    // unfalsifiable: nothing would show the harness can observe or report a
    // deserialization Err at all, and "ADMITTED" could just be what
    // serde_admits prints regardless of outcome.
    let wrong_type = serde_json::json!({ "admitted_cp": "not-an-int" });
    assert!(
        serde_json::from_value::<UriValidatedScalar>(wrong_type).is_err(),
        "shape control: serde must refuse a non-integer admitted_cp"
    );

    let missing_field = serde_json::json!({});
    assert!(
        serde_json::from_value::<UriValidatedScalar>(missing_field).is_err(),
        "shape control: serde must refuse a missing admitted_cp field"
    );
}

#[test]
fn known_hole_direct_struct_literal_forges_a_refused_surrogate() {
    // Compiles at all only because `admitted_cp` is a public field with no
    // construction wall enforced by the Rust type itself -- no constructor
    // call, no refusal possible. The assertion is trivial; the finding is that
    // this function is well-typed.
    let forged = UriValidatedScalar {
        admitted_cp: SURROGATE_CP,
    };
    assert_eq!(forged.admitted_cp, SURROGATE_CP);
}
