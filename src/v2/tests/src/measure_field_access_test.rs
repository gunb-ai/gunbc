//! G2: single-hop alias field access through parametric carriers (Measure / ByteSize).
//!
//! Both tests pin the contract end-to-end via `compile_dag_resolved` and assert
//! zero hard diagnostics (TESTING.md: behavior-driven, hermetic, one claim per
//! test). The multi-hop alias case (MoneyMicros = MoneyAmount<Micro> =
//! Measure<Currency, Micro>) is a G3 follow-up; see the note at the bottom.

use v2_compiler::v2_std_core::diagnostic_to_message;

use crate::helpers::compile_dag_resolved;

fn hard_diagnostic_messages(
    resolved: &v2_compiler::v2_compiler_compile::ResolvedPipelineResult,
) -> Vec<String> {
    resolved
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect()
}

// Generic parametric alias (Box<Nat> via NatBox), field access through the
// expanded RHS.
#[test]
fn generic_alias_field_access_resolves_through_expansion() {
    let src = r#"
module m

import std.nat { Nat }

type Box<T> {
  value: T
}

type NatBox = Box<Nat>

fn get(b: NatBox) -> Nat {
  b.value
}
"#;
    let msgs = hard_diagnostic_messages(&compile_dag_resolved(src));
    assert!(
        msgs.is_empty(),
        "generic alias field access should resolve, got: {msgs:?}"
    );
}

// The named ByteSize use case for this PR: ByteSize = Measure<Memory, One>
// requires G1 (phantom type-arg resolution of Memory/One) and then G2 (single-hop
// alias field expansion) so `b.count` resolves. End-to-end, zero diagnostics.
#[test]
fn bytesize_alias_field_access_resolves_end_to_end() {
    let src = r#"
module m

type Nat

type Quantity = Memory | Count | Currency | Frequency
type Scale = One | Micro

type Measure<Q, S> {
  count: Nat
}

type ByteSize = Measure<Memory, One>

fn byte_size_count(b: ByteSize) -> Nat {
  b.count
}
"#;
    let msgs = hard_diagnostic_messages(&compile_dag_resolved(src));
    assert!(
        msgs.is_empty(),
        "ByteSize single-hop alias field access should resolve, got: {msgs:?}"
    );
}

// NOTE: a `measure_dag_v2_loads_without_field_errors` test (asserting all of
// measure.dag loads on v2 with zero diagnostics) is deliberately NOT in this
// proven-G1+G2 slice. It currently fails with `no field 'count' on type
// 'MoneyMicros'`: MoneyMicros = MoneyAmount<Micro> = Measure<Currency, Micro>
// is a MULTI-HOP alias chain, and G2 alias-field expansion only resolves the
// single-hop case so far (ByteSize.count, covered above). Multi-hop alias
// field access lands with the G3 follow-up; the full-load test moves there
// where it goes green.
