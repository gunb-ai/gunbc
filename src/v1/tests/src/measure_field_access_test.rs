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

#[test]
fn g3_param_dependent_field_through_parametric_alias_chain_resolves() {
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
        msgs.is_empty(),
        "param-dependent field through a parametric-alias chain should resolve once type args \
         are preserved across hops, got: {msgs:?}"
    );
}

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

    let roots: Vec<String> = crate::helpers::source_roots()
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
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
    let roots: Vec<String> = crate::helpers::source_roots()
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
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
