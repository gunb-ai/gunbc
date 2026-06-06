//! **Layer:** integration
//!
//! R3 gate #83 structural ratchet over `std.verification` `lens_capability_register_rows`.
//! The retired prose mirror `docs/v3-lens-capability-register.md` (removed #4192 public
//! cleanup) is not a second authority; markdown↔structural correspondence tests were
//! deleted as stale doc consumers in the frozen-dir drift triage.

use v3_compiler::cementing_dispatch;
use v3_compiler::dag::Dag;

const R3_LENS_BEHAVIORAL_PARITY_SCOPE: &[&str] = &[
    "complexity.dag",
    "cost.dag",
    "parallelism.dag",
    "effect_enumeration.dag",
];

#[test]
fn r3_gate_83_lens_capability_register_std_verification_behavioral_axis_has_zero_stub() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap Dag should load cleanly for std.verification lens_capability_register_rows \
         behavioral-axis ratchet, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    let labels = cementing_dispatch::lens_capability_register_behavioral_label_by_basename(&dag)
        .expect("read behavioral axis from lens_capability_register_rows");
    let mut blockers = Vec::new();
    for basename in R3_LENS_BEHAVIORAL_PARITY_SCOPE {
        let Some(label) = labels.get(*basename) else {
            panic!(
                "R3 gate #83: `lens_capability_register_rows` must include `{basename}` — add the row \
                 in `src/v3/std/verification.dag`."
            );
        };
        if label == "LensCapabilityBehavioralStub" {
            blockers.push(format!("{basename}: {label}"));
        }
    }
    assert!(
        blockers.is_empty(),
        "R3 gate #83 — `std.verification` `lens_capability_register_rows` behavioral-axis ratchet \
         (field `behavioral: LensCapabilityBehavioralStatus`): the four T-Lens-Behavioral-Parity \
         basenames must not carry `LensCapabilityBehavioralStub`. Remaining STUB blocker(s): \
         {blockers:?}."
    );
}
