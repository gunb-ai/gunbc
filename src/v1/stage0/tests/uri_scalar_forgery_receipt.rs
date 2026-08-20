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
//! not close it. The positive control (`cp = 65`) is asserted to succeed on BOTH
//! sides so a wholesale serde failure could not masquerade as a pass. Repair
//! (omit `Deserialize`, emit a validating `Deserialize`, or seal the field behind
//! `TryFrom`) is a separate design decision per the ARM 3 brief and is not made
//! here. DISSOLVE-ON: `sole_constructor` completeness audit
//! (compiler-guarantee-recovery-gap-analysis.md §11 item 1a) extends the
//! unforgeable-construction wall through Rust emission for sealed carriers; at
//! that point the `serde`/struct-literal assertions below flip from "admits" to
//! "refuses" and this receipt becomes the regression control for that repair.

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
