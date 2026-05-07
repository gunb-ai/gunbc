//! **Layer:** integration
//!
//! Cementing test for `Lens<List<EmissionProvenance>>` per Director Q1
//! (a) RATIFIED at gunbc#1739 #issuecomment-4392562911 (per-Behavior
//! Lens<C>-compatible framing). Brief:
//! `docs/briefs/r3-substrate-emission-provenance-lens-worker.md`.
//!
//! **Scope of this slice — STRUCTURAL.** This test cements the
//! substrate-shape contract: the `EmissionProvenance` record carrier and
//! the lens fns/types compile, declare the Director-locked field set,
//! and the lens binds against the correct carrier under a fixture-bound
//! `data` declaration (matching the `mini_lens` precedent at
//! `e6_g1a_option3_static_lens_test.rs:66-73`).
//!
//! **Behavioral round-trip is BEHAVIORALLY DEFERRED** — emitter writes
//! field-path → lens recovers same field-path; per-Behavior list flatten
//! matches per-line view. Producer wiring lives in
//! `src/v3/compiler/src/emit/rust_target.rs`'s `render_named_template`
//! call sites and is a separate downstream slice (precedent: `lenses.cost`
//! BEHAVIORALLY PROXY).
//!
//! **Substrate-cascade receipt** — promoting the lens instance from a
//! fixture-bound `data` declaration to a top-level `data` item in
//! `src/v3/lenses/emission_provenance.dag` is currently blocked by
//! Class 5 Gap 3 (top-level `ValueBody` boundary; sum-variant literals
//! like `Empty` don't lower in `data` body context). Same constraint
//! that blocks the canonical generic `data list_monoid<element>:
//! Monoid<List<element>>` declaration documented at
//! `src/v3/std/list.dag:61-64`. Dissolution trigger: when Class 5 Gap 3
//! closes, this fixture promotes to a top-level `data` declaration in
//! the lens .dag file without lens-shape changes.

use std::collections::HashSet;

use crate::common::cached_compile_to_dag;
use v3_compiler::dag::{ArrowBody, Dag, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

fn conj_field_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from compiled dag"));
    match &decl.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        other => panic!("`{name}` is not a Conj (record): {other:?}"),
    }
}

fn disj_variant_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from compiled dag"));
    match &decl.connective {
        TypeConnective::Disj { variants } => variants.iter().map(|v| v.label.clone()).collect(),
        other => panic!("`{name}` is not a Disj (sum): {other:?}"),
    }
}

#[test]
fn emission_provenance_record_has_locked_three_field_shape() {
    // Director-locked record shape per brief Deliverable 1
    // (codex BLOCKING reshape at PR #1910 sha 2ed1046e Finding #1
    // applied: record form, not coproduct; mandatory `rule` enforces
    // structural fail-closed; optional `source_span` is structurally
    // legal absence).
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "EmissionProvenance")
        .into_iter()
        .collect();
    let expected: HashSet<&str> = ["emitted_line", "rule", "source_span"]
        .into_iter()
        .collect();
    let actual_refs: HashSet<&str> = labels.iter().map(String::as_str).collect();
    assert_eq!(
        actual_refs, expected,
        "EmissionProvenance must carry the Director-locked 3-field record shape \
         (mandatory `rule: EmissionRule`, optional `source_span: OptionalSourceSpan`, \
         positional `emitted_line: Int`); actual labels: {labels:?}"
    );
}

#[test]
fn emitted_line_is_positive_int_refinement_not_bare_int() {
    // Codex BLOCKING finding (sha 6c7c3d85): line numbers are positive
    // by construction (1-indexed). The carrier MUST type `emitted_line`
    // as `PositiveInt` (`dsl/std/integer.dag:137`) so the type system
    // enforces the invariant rather than leaving it as a comment.
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name("EmissionProvenance")
        .expect("EmissionProvenance missing from bootstrap");
    let TypeConnective::Conj { children } = &decl.connective else {
        panic!(
            "EmissionProvenance is not a Conj record: {:?}",
            decl.connective
        );
    };
    let emitted_line_field = children
        .iter()
        .find(|f| f.label == "emitted_line")
        .expect("`emitted_line` field missing from EmissionProvenance");
    let positive_int = dag
        .declaration_by_name("PositiveInt")
        .expect("PositiveInt missing from bootstrap");
    assert_eq!(
        emitted_line_field.ty, positive_int.id,
        "EmissionProvenance.emitted_line must be `PositiveInt`, not bare `Int` \
         (1-indexed line coordinates are positive by construction; type system \
         enforces the invariant per codex BLOCKING finding on PR #1928 sha 6c7c3d85)"
    );
}

#[test]
fn optional_source_span_carries_none_and_some_arms() {
    // v3 std has no generic Option<T>; the typed-sum pattern is the
    // idiom (compare `OptionalDiagnostic` at `v3.std.dimensions`).
    // NoSourceSpan must be a structurally legal absence (NOT a
    // diagnostic) per Director `feedback_no_textual_enforcement_bridges`.
    let dag = generated_full_bootstrap_dag();
    let variants: HashSet<String> = disj_variant_labels(&dag, "OptionalSourceSpan")
        .into_iter()
        .collect();
    let expected: HashSet<&str> = ["NoSourceSpan", "SomeSourceSpan"].into_iter().collect();
    let actual_refs: HashSet<&str> = variants.iter().map(String::as_str).collect();
    assert_eq!(
        actual_refs, expected,
        "OptionalSourceSpan must be the 2-variant typed-sum shape \
         (NoSourceSpan structurally legal; SomeSourceSpan {{ value: SourceSpan }}); \
         actual variants: {variants:?}"
    );
}

// Fixture-bound lens instance per the `mini_lens` precedent at
// `e6_g1a_option3_static_lens_test.rs:66-73`. The string source declares
// the carrier inline + the lens fns + the `data emission_provenance_lens:
// Lens<List<EmissionProvenance>>` instance. This is the cementing form
// for the Director-locked 6-field `Lens<C>` shape against the
// `List<EmissionProvenance>` carrier; promotion to a top-level `data`
// item in `src/v3/lenses/emission_provenance.dag` blocks on Class 5
// Gap 3 closure (sum-variant `Empty` literal in data body context).
const LENS_FIXTURE_SOURCE: &str = r#"
import std.integer { PositiveInt }
import std.list { List, concat }
import std.substrate { Dag, Behavior, LoopBound }
import v3.std.dimensions { Witness, OptionalDiagnostic }
import v3.std.diagnostics { Diagnostic }
import v3.std.lens { Lens }
import v3.std.substrate { SourceSpan }

type EmissionRule = String

type OptionalSourceSpan
  = NoSourceSpan
  | SomeSourceSpan { value: SourceSpan }

type EmissionProvenance {
  emitted_line: PositiveInt
  rule: EmissionRule
  source_span: OptionalSourceSpan
}

// Lowering seeds for `List<EmissionProvenance>.Empty` — same shape as
// the `_e6_seed_list_*_empty` pattern at e6_g1a_option3_static_lens_test.rs:45-48.
fn _seed_list_provenance_empty() -> List<EmissionProvenance> = Empty

fn read_provenance(d: Dag, b: Behavior) -> Witness<List<EmissionProvenance>> =
  Inhabits(Empty)

fn empty_provenance_list() -> List<EmissionProvenance> = Empty

fn concat_provenance(
  a: List<EmissionProvenance>,
  b: List<EmissionProvenance>
) -> List<EmissionProvenance> = concat(a, b)

fn branch_provenance(
  l: List<EmissionProvenance>,
  r: List<EmissionProvenance>
) -> List<EmissionProvenance> = concat(l, r)

fn iterate_provenance(
  c: List<EmissionProvenance>,
  bound: LoopBound
) -> List<EmissionProvenance> = c

fn validate_provenance(d: Dag, c: List<EmissionProvenance>) -> OptionalDiagnostic =
  NoDiagnostic

data emission_provenance_lens: Lens<List<EmissionProvenance>> = {
  name: "EmissionProvenance",
  read: read_provenance,
  sequential: { op: concat_provenance, identity: empty_provenance_list },
  branch: branch_provenance,
  iterate: iterate_provenance,
  validate: validate_provenance
}
"#;

#[test]
#[ignore = "blocked on Class 5 Gap 3 (top-level ValueBody boundary at \
            src/v3/compiler/src/dag.rs:259-287): `data emission_provenance_lens: \
            Lens<List<EmissionProvenance>>` rejects on two facets of the same \
            gap — bare sum-variant identity (`identity: Empty`) doesn't lower \
            in data body context, AND nested-fn identity (`identity: \
            empty_provenance_list`) hits the opaque-body rejection for \
            fn-typed fields in data bodies. Same constraint blocks the \
            canonical `data list_monoid<element>: Monoid<List<element>>` at \
            `src/v3/std/list.dag:61-64`. Dissolution trigger: when both facets \
            close (top-level List/sum-variant ValueBody carriers AND fn-typed \
            data-body fields validate structurally), un-ignore and verify this \
            test passes without lens-shape changes. See lens .dag file header \
            for the unified two-facet receipt."]
fn emission_provenance_lens_binds_against_locked_carrier() {
    // Structural cement: the `Lens<List<EmissionProvenance>>` instance
    // compiles against the Director-locked 6-field `Lens<C>` carrier at
    // `src/v3/std/lens.dag` under the fixture-bound pattern.
    //
    // Currently `#[ignore]`d on the substrate-cascade STOP described in
    // the test attribute. The fixture source IS authored in
    // `LENS_FIXTURE_SOURCE` so the dissolution trigger is the simple
    // act of removing this `#[ignore]` once Class 5 Gap 3 closes.
    let _dag = cached_compile_to_dag(
        LENS_FIXTURE_SOURCE,
        "tests/integration/emission_provenance_lens_test.rs:LENS_FIXTURE_SOURCE",
    );
}
