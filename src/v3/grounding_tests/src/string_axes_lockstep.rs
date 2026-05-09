//! Forward ratchet: #1465 string-family axis sums stay structural and closed.
//!
//! This is intentionally narrower than a general lifetime-axis mirror ratchet.
//! `StringOwnershipAxis`, `StringLifetimeAxis`, `StringGrowabilityAxis`, and
//! `StringEncodingAxis` are target-row vocabulary for string-family candidates
//! under `src/v3/std/emit_model.dag`. They are not a global replacement for
//! `grounding_lifetime/src/facts.rs` program-side enums.
//!
//! Future mirror mapping notes (not asserted here):
//! - `grounding_lifetime::Ownership::{Owned,Borrowed}` maps conceptually to
//!   `StringOwnershipAxis::{Owned,Borrowed}` once target rows consume substrate
//!   references.
//! - `grounding_lifetime::Growability::{Yes,No,NotApplicable}` maps
//!   conceptually to `StringGrowabilityAxis::{Growable,Fixed,NotApplicable}`.
//! - `grounding_lifetime::LifetimeScope::Self_` must not be asserted as a
//!   literal subset of `StringLifetimeAxis::SelfContained`; `Self_` is Rust enum
//!   escaping, while `SelfContained` is substrate vocabulary.

use v3_compiler::dag::{Dag, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

#[derive(Clone, Copy, Debug)]
struct StringAxisExpectation {
    name: &'static str,
    variants: &'static [&'static str],
}

const STRING_AXIS_EXPECTATIONS: &[StringAxisExpectation] = &[
    StringAxisExpectation {
        name: "StringOwnershipAxis",
        variants: &["Owned", "Borrowed"],
    },
    StringAxisExpectation {
        name: "StringLifetimeAxis",
        variants: &["SelfContained", "Caller"],
    },
    StringAxisExpectation {
        name: "StringGrowabilityAxis",
        variants: &["Growable", "Fixed", "NotApplicable"],
    },
    StringAxisExpectation {
        name: "StringEncodingAxis",
        variants: &["Utf8FreeMonoidChar"],
    },
];

fn string_axis_variant_labels(dag: &Dag, axis_name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(axis_name)
        .unwrap_or_else(|| panic!("bootstrap Dag must declare `{axis_name}`"));
    assert_eq!(
        decl.span.file, "src/v3/std/emit_model.dag",
        "`{axis_name}` must stay under emit-model string-family axis authority"
    );
    match &decl.connective {
        TypeConnective::Disj { variants } => variants.iter().map(|v| v.label.clone()).collect(),
        other => panic!("`{axis_name}` must be a Disj sum; got {other:?}"),
    }
}

fn variant_mismatch(actual: &[String], expected: &[&str]) -> Option<String> {
    let expected_owned: Vec<String> = expected.iter().map(|label| label.to_string()).collect();
    if actual == expected_owned {
        None
    } else {
        Some(format!(
            "expected closed variant set {expected_owned:?}, got {actual:?}"
        ))
    }
}

#[test]
fn string_family_axes_are_closed_structural_values() {
    let dag = generated_full_bootstrap_dag();

    for expectation in STRING_AXIS_EXPECTATIONS {
        let actual = string_axis_variant_labels(&dag, expectation.name);
        assert!(
            variant_mismatch(&actual, expectation.variants).is_none(),
            "{} drifted: {}",
            expectation.name,
            variant_mismatch(&actual, expectation.variants).unwrap_or_default()
        );
    }
}

/// Negative control: the same mismatch path must detect a missing substrate-axis variant.
#[test]
fn string_axis_ratchet_detects_synthetic_missing_variant() {
    let actual = vec!["Growable".to_string(), "Fixed".to_string()];
    let mismatch = variant_mismatch(&actual, &["Growable", "Fixed", "NotApplicable"])
        .expect("synthetic missing NotApplicable variant must be detected");

    assert!(
        mismatch.contains("NotApplicable"),
        "mismatch should name the missing explicit variant; got {mismatch}"
    );
}
