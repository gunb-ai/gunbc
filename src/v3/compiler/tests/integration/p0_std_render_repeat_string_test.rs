//! **Layer:** integration
//!
//! R1C-B / T-P0: `repeat_string` lower-time fold via the v3 `TestRunner`
//! (`p0_repeat_string_correct_gate` in `tests/fixtures/r1_gates.dag`).
//! Live v2 interpreter oracle retired (T-V2-Retirement); semantics are covered
//! by the gate + `lower::fold_repeat_string_semantics` unit tests.

/// R1C-B — structural `p0_repeat_string_correct` suite through the v3 `TestRunner`.
#[test]
fn p0_repeat_string_correct_gate_passes_through_test_runner() {
    crate::common::assert_p0_repeat_string_correct_gate_passes();
}
