//! G2/G3: alias field access through parametric carriers (Measure / ByteSize).
//!
//! G2 (single-hop): ByteSize = Measure<Memory, One>, NatBox = Box<Nat>.
//! G3 (multi-hop): MoneyMicros = MoneyAmount<Micro> = Measure<Currency, Micro>
//! resolves through the alias chain; a param-DEPENDENT field threaded through a
//! parametric-alias chain fails closed (boundary witness); and all of
//! `dsl/std/measure.dag` loads with zero hard diagnostics. Tests pin the
//! contract end-to-end and assert zero hard diagnostics (TESTING.md:
//! behavior-driven, one claim per test).

use v1_compiler::v1_std_core::diagnostic_to_message;

use crate::helpers::compile_dag_resolved;

fn hard_diagnostic_messages(
    resolved: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult,
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

// Fail-closed boundary (G3 condition #1): a field whose TYPE is a generic param
// threaded THROUGH a parametric-alias chain cannot be substituted (the alias
// param list is dropped on the resolved binding), so it MUST error -- never
// silently resolve to the raw type variable. This case is uninhabited in dsl
// today; the test pins the honest fail so a future regression to silent-wrong
// is caught. `Wrap<T> = Box<T>` then `IntWrap = Wrap<Int>`; `w.value` is `T`,
// unsubstitutable through the parametric-alias hop.
//
// By-execution at this head (probe in expand_alias_chain_for_field_access): the
// chain reaches the structural record `Wrap` (NOT a fully-instantiated `Box<Int>`
// with `value: Int`) with `lossy=true` and `record_has_unresolved_param_field=true`,
// so the `lossy && record_has_unresolved_param_field` branch (04_infer.dag:2793)
// fires and returns `nominal_type_ref(origin_name="IntWrap")`. We therefore assert
// the SPECIFIC origin-nominal diagnostic, not merely non-emptiness, so the test
// can only pass when the documented fail-closed branch fired (not for an unrelated
// diagnostic, and not if generic instantiation silently resolved it). Its twin
// `param_independent_field_through_parametric_alias_chain_resolves` is the positive
// control: the identical chain shape with a phantom (param-independent) field
// resolves cleanly, isolating that fail-closed is driven by the param-DEPENDENT
// field, not by the chain being unreachable.
#[test]
fn g3_param_dependent_field_through_parametric_alias_chain_fails_closed() {
    let src = r#"
module m

type Box<T> {
  value: T
}

type Wrap<T> = Box<T>
type IntWrap = Wrap<Int>

fn unwrap(w: IntWrap) -> Int {
  w.value
}
"#;
    let msgs = hard_diagnostic_messages(&compile_dag_resolved(src));
    assert!(
        msgs.iter()
            .any(|m| m.contains("no field 'value' on type 'IntWrap'")),
        "param-dependent field through a parametric-alias chain must fail closed with the \
         origin-nominal `no field 'value' on type 'IntWrap'` diagnostic (proving \
         nominal_type_ref(origin) fired at the lossy && unresolved-param boundary), got: {msgs:?}"
    );
}

// Positive control twin for the fail-closed boundary above: the IDENTICAL
// parametric-alias chain shape (parametric record <- parametric alias <- concrete
// alias) but with a phantom, param-INDEPENDENT field resolves cleanly. This proves
// the chain genuinely REACHES the structural record through the lossy hop -- so the
// fail-closed in the twin is caused specifically by the param-DEPENDENT field, not
// by the chain being broken or unreachable. `Tagged<T>` carries `tag: Nat` (T is
// phantom), so `record_has_unresolved_param_field` is false and field access
// resolves end-to-end. Mirrors the MoneyMicros (count: Nat) shape minimally.
#[test]
fn param_independent_field_through_parametric_alias_chain_resolves() {
    let src = r#"
module m

type Nat

type Tagged<T> {
  tag: Nat
}

type WrapTagged<T> = Tagged<T>
type IntTagged = WrapTagged<Int>

fn get_tag(w: IntTagged) -> Nat {
  w.tag
}
"#;
    let msgs = hard_diagnostic_messages(&compile_dag_resolved(src));
    assert!(
        msgs.is_empty(),
        "param-independent (phantom) field through the same parametric-alias chain shape should \
         resolve, isolating fail-closed to param-dependent fields, got: {msgs:?}"
    );
}

#[test]
fn g3_repro_multihop_alias_field_access() {
    let src = r#"
module m

type Nat

type Quantity = Memory | Count | Currency | Frequency
type Scale = One | Micro

type Measure<Q, S> {
  count: Nat
}

type MoneyAmount<S> = Measure<Currency, S>
type MoneyMicros = MoneyAmount<Micro>

fn money_micros_count(m: MoneyMicros) -> Nat {
  m.count
}
"#;
    let msgs = hard_diagnostic_messages(&compile_dag_resolved(src));
    assert!(
        msgs.is_empty(),
        "G3 multi-hop alias field access should resolve, got: {msgs:?}"
    );
}

// G3 landing site (moved here from the parked G1+G2 NOTE): the whole of
// `dsl/std/measure.dag` loads on v2 with zero hard diagnostics. Before G3 this
// failed with `no field 'count' on type 'MoneyMicros'` -- the multi-hop chain
// MoneyMicros = MoneyAmount<Micro> = Measure<Currency, Micro>. With multi-hop
// alias field-access it resolves end-to-end (transitive import closure).
//
// Emit regression (G1 phantom enum type-args): parametric aliases such as
// `MoneyAmount<S> = Measure<Currency, S>` must terminate under rust-emit — the
// phantom Quantity/Scale tags are not standalone type items and must short-circuit
// the alias RHS authority walk (see 05_emit_rust.dag is_phantom_unit_variant_type_arg).
#[test]
fn money_amount_parametric_alias_rust_emit_terminates() {
    let src = r#"
module m
type Nat
type Quantity = Time | Memory | Currency
type Scale = One | Micro
type Measure<Q, S> { count: Nat }
type MoneyAmount<S> = Measure<Currency, S>
"#;
    let result = crate::helpers::compile_dag(src);
    crate::helpers::assert_no_diagnostics(&result);
    assert!(
        !result.files.is_empty(),
        "parametric Measure alias with phantom enum type-arg should emit Rust"
    );
}

#[test]
fn measure_dag_rust_emit_terminates() {
    use std::rc::Rc;
    use v1_compiler::cli_run;
    use v1_compiler::v1_compiler_artifact::RenderTarget;
    use v1_compiler::v1_compiler_compile::compile_sources;
    use v1_compiler::v1_std_core::diagnostic_to_message;

    let ws = crate::helpers::workspace_root();
    let roots: Vec<String> = vec![
        ws.join("dsl").to_string_lossy().to_string(),
        ws.join("src/v2").to_string_lossy().to_string(),
    ];
    let entry = crate::helpers::workspace_root().join("dsl/std/measure.dag");
    let entry = entry.to_string_lossy().to_string();
    let sources = cli_run::load_sources_for_entry(&roots, &entry)
        .unwrap_or_else(|e| panic!("failed to load {entry}: {e}"));
    let result = compile_sources(Rc::new(sources), RenderTarget::Rust);
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty(),
        "measure.dag rust emit should complete with zero hard diagnostics, got: {msgs:?}"
    );
    assert!(
        !result.files.is_empty(),
        "measure.dag rust emit should produce files"
    );
}

#[test]
fn measure_dag_v2_loads_without_field_errors() {
    use std::rc::Rc;
    let ws = crate::helpers::workspace_root();
    let roots: Vec<String> = vec![
        ws.join("dsl").to_string_lossy().to_string(),
        ws.join("src/v2").to_string_lossy().to_string(),
    ];
    let entry = crate::helpers::workspace_root().join("dsl/std/measure.dag");
    let entry = entry.to_string_lossy().to_string();
    let sources = v1_compiler::cli_run::load_sources_for_entry(&roots, &entry)
        .unwrap_or_else(|e| panic!("failed to load {entry}: {e}"));
    let resolved = v1_compiler::v1_compiler_compile::compile_to_resolved(Rc::new(sources));
    let msgs = hard_diagnostic_messages(resolved.as_ref());
    assert!(
        msgs.is_empty(),
        "measure.dag should load on v2 with zero hard diagnostics, got: {msgs:?}"
    );
}
