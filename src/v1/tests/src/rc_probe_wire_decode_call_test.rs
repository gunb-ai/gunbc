//! ctrl#1476 B3-Gap2 — SG-RC probe wire-decode call-site arg fix + detection test.
//!
//! `target_reference_layer_probe_from_emitted_type` (src/v2/std/compilers/target_model.dag) decodes the
//! emitted type wire via `target_type_expr_emitted_wire_decode`, whose signature is
//! `(node:, projection:)`. The original call passed the wrong named arg (`emitted:`) and omitted
//! `projection:`, leaving `node` unbound at runtime ("undefined variable: node"). The bug was
//! invisible at compile time because v2 has no call-site arg-name/arity check yet (enforcement-B
//! lane). This test reds if the wrong call shape reappears at the probe call-site.
//!
//! Runtime greenness (`sg_rc_f6_round_trip_owned` passes; `sg_rc_f6_round_trip_rc` is a separate
//! tracked-red read-path bug) is asserted via the v2 roster pilot claim-run, not here.

use crate::helpers::read_v2_file;

const TARGET_MODEL_DAG: &str = "src/v2/std/compilers/target_model.dag";

/// Drop `//` comment lines so the guard checks live code, not the explanatory mark.
fn live_source(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn probe_wire_decode_call_uses_node_and_projection_args() {
    let source = read_v2_file(TARGET_MODEL_DAG);
    let live = live_source(&source);

    // Correct call shape must be present: decode takes (node:, projection:).
    assert!(
        live.contains(
            "target_type_expr_emitted_wire_decode(node: emitted, projection: projection)"
        ),
        "SG-RC probe must call target_type_expr_emitted_wire_decode with (node:, projection:) \
         (ctrl#1476 B3-Gap2). target_model.dag:target_reference_layer_probe_from_emitted_type."
    );

    // The wrong named-arg shape must NOT reappear in live code (regression guard).
    assert!(
        !live.contains("target_type_expr_emitted_wire_decode(emitted:"),
        "regression: target_type_expr_emitted_wire_decode called with `emitted:` — its signature is \
         (node:, projection:); the misnamed arg leaves `node` unbound at runtime \
         ('undefined variable: node'). ctrl#1476 B3-Gap2."
    );
}
