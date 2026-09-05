// Execute the producer-emitted .dag controls against the native compiler.
// Scaffold: delete when the required native compiler test path discovers and
// invokes these .dag controls against the exact compiler artifact under test,
// preserving their positive/refusal assertions. Interpreted root coverage alone
// does not replace this observation.
use v1_compiler::{
    v1_tests_claim_algebra_application_evidence_test as evidence,
    v1_tests_claim_algebra_application_refusal_test as refusal,
};

#[test]
fn binding_preserves_application_argument_instead_of_variant_member() {
    assert!(evidence::binding_preserves_application_argument_instead_of_variant_member());
}

#[test]
fn structural_receiver_without_application_refuses_at_requested_position() {
    assert!(refusal::structural_receiver_without_application_refuses_at_requested_position());
}

#[test]
fn missing_application_diagnostic_prevents_method_resolution() {
    assert!(refusal::missing_application_diagnostic_prevents_method_resolution());
}
