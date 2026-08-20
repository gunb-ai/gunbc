//! ARM 3 executed receipt (findings-only; no repair — see
//! docs/plans/compiler-guarantee-recovery-gap-analysis.md §11 item 1a).
//!
//! `extdeps.uri` `UriValidatedScalar` is declared `sole_constructor` in the `.dag`
//! source with a single field `admitted_cp: Int`; its only sanctioned mint is
//! `uri_validated_scalar_construction`, whose fixed law refuses surrogate code
//! points (`UriValidatedScalarSurrogateRefused`) and out-of-range code points
//! (`UriValidatedScalarOutOfRangeRefused`) — each refusal covers a whole RANGE,
//! not a finite set. `uri_percent_encode_admitted_scalar_wire` accepts
//! `UriValidatedScalar` only and reads `admitted_cp` directly without re-running
//! the mint, so a forged scalar reaches percent-encoding output — executed below,
//! not inferred from the signature.
//!
//! The `.dag` `sole_constructor` wall refuses a cross-module record literal for this
//! carrier at compile time (`SoleConstructorViolation`, see
//! `dag/test/claim/guarantee_probe_corpus_witness_test.dag`
//! `sole_ctor_forged_source`). This receipt establishes, by execution against the
//! emitted stage0 Rust mirror, that the wall does not survive emission: the carrier
//! is emitted as `pub struct UriValidatedScalar { pub admitted_cp: i64 }` deriving
//! `serde::Serialize, serde::Deserialize` (`extdeps_uri.rs`), so it is freely
//! constructible both as a Rust struct literal (compile-time, no constructor call)
//! and via `serde_json::from_value` — for a representative of each of the three
//! refusal partitions the `.dag` mint enforces (negative, surrogate, above-max),
//! not for every value in those infinite ranges: three representative points are
//! executed evidence of the class of gap, not a universal claim discharged by a
//! finite test.
//!
//! KNOWN HOLE, asserted as it stands today — this test documents the gap, it does
//! not close it. The positive control (`cp = 65`) controls the *mint's* behavior,
//! not serde's: it is admitted on both sides, so on its own it cannot show this
//! harness is capable of observing an `Err`. `serde_shape_control_rejects_malformed_input`
//! is the discriminator that closes that hole — a value serde itself must reject
//! regardless of the `admitted_cp` predicate (wrong JSON type, and the field
//! missing entirely) — so every "ADMITTED" verdict elsewhere in this file is
//! load-bearing: the harness has demonstrated it can and does report `Err`.
//!
//! Repair closes TWO INDEPENDENT doors, and is a conjunction, not a menu of
//! alternatives: the field must stop being publicly constructible (a Rust
//! struct literal needs no constructor and does not go through `serde` at
//! all) AND `Deserialize` must be absent or validating (removing only the
//! field's public visibility does not stop the derived `Deserialize` impl,
//! which constructs the struct from inside its own module, where privacy
//! does not apply). Any repair that leaves one door open leaves the carrier
//! forgeable. This is a separate design decision per the ARM 3 brief and is
//! not made here. DISSOLVE-ON: `sole_constructor` completeness audit
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
    uri_percent_encode_admitted_scalar_wire, uri_validated_scalar_construction,
    UriPercentEncodeFoldState, UriPercentEncodeRefusalCause, UriValidatedScalar,
    UriValidatedScalarConstruction,
};

const SURROGATE_CP: i64 = 55_296; // 0xD800, first code point in the surrogate range
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
fn known_hole_serde_admits_representatives_from_every_mint_refusal_partition() {
    // The mint refuses whole RANGES (negative, surrogate, above-max), each an
    // infinite set. This test executes one representative from each of the
    // three refusal partitions -- it does not and cannot discharge a claim
    // over every value in those ranges.
    assert!(
        serde_admits(SURROGATE_CP),
        "known hole: serde forges a representative of the surrogate refusal partition"
    );
    assert!(
        serde_admits(NEGATIVE_CP),
        "known hole: serde forges a representative of the negative refusal partition"
    );
    assert!(
        serde_admits(ABOVE_MAX_CP),
        "known hole: serde forges a representative of the above-max refusal partition"
    );
    // Positive control: must also succeed, or the harness is void.
    assert!(
        serde_admits(ADMITTED_CP),
        "positive control: serde must admit a value the .dag mint also admits"
    );
}

#[test]
fn known_hole_forged_scalars_reach_percent_encode_output() {
    // Executes the actual downstream consumer, uri_percent_encode_admitted_scalar_wire,
    // on struct literals built directly from the three refusal-partition
    // representatives -- not an inference from the function's signature. The
    // consumer reads admitted_cp directly and does not re-run the scalar
    // mint, so a forged value is not merely constructible: it produces
    // percent-encoded OUTPUT.
    let surrogate = UriValidatedScalar {
        admitted_cp: SURROGATE_CP,
    };
    match &*uri_percent_encode_admitted_scalar_wire(surrogate) {
        UriPercentEncodeFoldState::UriPercentEncodeBuilding { wire } => {
            assert_eq!(
                wire, "%ED%A0%80",
                "known hole: forged surrogate scalar percent-encodes to invalid UTF-8 output"
            );
        }
        UriPercentEncodeFoldState::UriPercentEncodeRefused { cause } => {
            panic!("expected the forged surrogate to reach encoded output, got refusal: {cause:?}");
        }
    }

    let above_max = UriValidatedScalar {
        admitted_cp: ABOVE_MAX_CP,
    };
    match &*uri_percent_encode_admitted_scalar_wire(above_max) {
        UriPercentEncodeFoldState::UriPercentEncodeBuilding { wire } => {
            assert_eq!(
                wire, "%F4%90%80%80",
                "known hole: forged above-max scalar percent-encodes to invalid UTF-8 output"
            );
        }
        UriPercentEncodeFoldState::UriPercentEncodeRefused { cause } => {
            panic!("expected the forged above-max scalar to reach encoded output, got refusal: {cause:?}");
        }
    }

    // The negative representative does NOT behave uniformly with the other
    // two: uri_percent_encode_admitted_scalar_wire routes cp < 128 through
    // uri_utf8_octet_construction(cp) directly, which refuses byte < 0. So
    // the negative forgery is caught here, downstream of admission -- a
    // different failure mode from the other two partitions, not a
    // contradiction of the finding.
    let negative = UriValidatedScalar {
        admitted_cp: NEGATIVE_CP,
    };
    match &*uri_percent_encode_admitted_scalar_wire(negative) {
        UriPercentEncodeFoldState::UriPercentEncodeRefused { cause } => {
            assert!(
                matches!(
                    **cause,
                    UriPercentEncodeRefusalCause::UriPercentEncodeUtf8OctetOutOfRangeRefused {
                        value: NEGATIVE_CP
                    }
                ),
                "expected the negative forgery to be refused as an out-of-range UTF-8 octet, got: {cause:?}"
            );
        }
        UriPercentEncodeFoldState::UriPercentEncodeBuilding { wire } => {
            panic!("expected the forged negative scalar to be refused downstream, got output: {wire:?}");
        }
    }
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
