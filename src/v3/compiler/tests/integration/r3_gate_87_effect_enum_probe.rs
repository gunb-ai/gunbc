//! **Layer:** integration
//!
//! Temporary probe for authoring `EffectEnumerationReport` literals in
//! `tests/dag/t_r3_gate_87_cementing_regen_effect_enumeration.dag`.
//! Run once with:
//!   cargo test -p v3-compiler emit_effect_enumeration_debug -- --ignored --nocapture
//! then delete this module after copying the printed shape.

use v3_compiler::compile_to_dag;
use v3_compiler::lens_effect_enumeration::enumerate_effects;

#[test]
#[ignore = "one-shot probe for gate #87 .dag literals"]
fn emit_effect_enumeration_debug() {
    let dag = compile_to_dag("let lit: Int = 7", "r3_gate_87_effect_enum_probe.v3").expect("compile");
    let report = enumerate_effects(&dag);
    panic!("{report:#?}");
}
