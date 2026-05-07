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
use v3_compiler::dag::TypeConnective;
use v3_compiler::generated_full_bootstrap_dag;

#[test]
fn emission_provenance_record_has_locked_three_field_shape_with_locked_field_types() {
    // Director-locked record shape per brief Deliverable 1
    // (codex BLOCKING reshape at PR #1910 sha 2ed1046e Finding #1
    // applied: record form, not coproduct; mandatory `rule` enforces
    // structural fail-closed; optional `source_span` is structurally
    // legal absence).
    //
    // Per openai-pro NON-BLOCKING TESTING finding (sha 6251a1e5): also
    // assert each field's *type* (not just label) so the test fails if
    // `rule` stops being `EmissionRule` or `source_span` stops being
    // `OptionalSourceSpan`.
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
    let labels: Vec<&str> = children.iter().map(|f| f.label.as_str()).collect();
    // Arity check first — `HashSet` collapses duplicates, so a record with
    // (e.g.) `rule` declared twice would pass the set comparison below.
    // Assert exact arity to catch duplicate / extra fields explicitly.
    assert_eq!(
        children.len(),
        3,
        "EmissionProvenance must declare exactly 3 fields (emitted_line, rule, \
         source_span); actual labels: {labels:?}"
    );
    let expected_labels: HashSet<&str> = ["emitted_line", "rule", "source_span"]
        .into_iter()
        .collect();
    let actual_labels: HashSet<&str> = labels.iter().copied().collect();
    assert_eq!(
        actual_labels, expected_labels,
        "EmissionProvenance must carry the Director-locked 3-field record shape; \
         actual labels: {labels:?}"
    );

    // Each field's type must resolve to the locked carrier — not a
    // structural look-alike. If the carrier downstream weakens (e.g.,
    // `rule: String` instead of `rule: EmissionRule`), this fails.
    let emission_rule = dag
        .declaration_by_name("EmissionRule")
        .expect("EmissionRule missing from bootstrap");
    let optional_source_span = dag
        .declaration_by_name("OptionalSourceSpan")
        .expect("OptionalSourceSpan missing from bootstrap");
    let positive_int = dag
        .declaration_by_name("PositiveInt")
        .expect("PositiveInt missing from bootstrap");

    for field in children {
        match field.label.as_str() {
            "emitted_line" => assert_eq!(
                field.ty, positive_int.id,
                "EmissionProvenance.emitted_line must resolve to `PositiveInt`"
            ),
            "rule" => assert_eq!(
                field.ty, emission_rule.id,
                "EmissionProvenance.rule must resolve to `EmissionRule`"
            ),
            "source_span" => assert_eq!(
                field.ty, optional_source_span.id,
                "EmissionProvenance.source_span must resolve to `OptionalSourceSpan`"
            ),
            other => panic!("unexpected EmissionProvenance field: {other}"),
        }
    }
}

// Note: the standalone `emitted_line_is_positive_int_refinement_not_bare_int`
// test (codex BLOCKING finding sha 6c7c3d85) is now subsumed by
// `emission_provenance_record_has_locked_three_field_shape_with_locked_field_types`
// above, which asserts each field's type explicitly (including
// `emitted_line: PositiveInt`).

#[test]
fn production_lens_module_compiles_cleanly() {
    // Per openai-pro NON-BLOCKING TESTING finding (sha 4553d4e8): the
    // fixture-bound `#[ignore]`d test below mirrors the production lens
    // file but doesn't actively guard it — `src/v3/lenses/emission_provenance.dag`
    // could drift syntactically or type-wise without any active test
    // failing. This smoke compiles the production .dag file directly
    // (it lives outside the bootstrap, like other `src/v3/lenses/*.dag`
    // entries — compare `lens_apply.rs:1020` for `named_function_count.dag`),
    // so syntactic / type drift fails this test instead of slipping
    // through.
    let _dag = cached_compile_to_dag(
        include_str!("../../../lenses/emission_provenance.dag"),
        "src/v3/lenses/emission_provenance.dag",
    );
}

#[test]
fn optional_source_span_carries_none_and_some_arms_with_locked_payload() {
    // v3 std has no generic Option<T>; the typed-sum pattern is the
    // idiom (compare `OptionalDiagnostic` at `v3.std.dimensions`).
    // NoSourceSpan must be a structurally legal absence (NOT a
    // diagnostic) per Director `feedback_no_textual_enforcement_bridges`.
    //
    // Per openai-pro NON-BLOCKING TESTING finding (sha 6251a1e5):
    // assert that the `SomeSourceSpan` variant payload resolves to the
    // canonical `SourceSpan` carrier so the test fails if the payload
    // gets weakened or renamed.
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name("OptionalSourceSpan")
        .expect("OptionalSourceSpan missing from bootstrap");
    let TypeConnective::Disj { variants } = &decl.connective else {
        panic!(
            "OptionalSourceSpan is not a Disj sum: {:?}",
            decl.connective
        );
    };
    // Arity first — `HashSet` collapses duplicates; assert exact variant
    // count to catch a duplicate `SomeSourceSpan` declaration etc.
    assert_eq!(
        variants.len(),
        2,
        "OptionalSourceSpan must declare exactly 2 variants (NoSourceSpan, \
         SomeSourceSpan); actual variants: {:?}",
        variants.iter().map(|v| &v.label).collect::<Vec<_>>()
    );
    let actual_labels: HashSet<&str> = variants.iter().map(|v| v.label.as_str()).collect();
    let expected_labels: HashSet<&str> = ["NoSourceSpan", "SomeSourceSpan"].into_iter().collect();
    assert_eq!(
        actual_labels, expected_labels,
        "OptionalSourceSpan must be the 2-variant typed-sum shape \
         (NoSourceSpan structurally legal; SomeSourceSpan {{ value: SourceSpan }})"
    );

    // The `SomeSourceSpan` variant's payload type must resolve to the
    // canonical `SourceSpan` from `dsl/std/types.dag` (re-exported via
    // `v3.std.substrate`), not a structural look-alike. The
    // `value` field on the variant payload is what carries the
    // SourceSpan reference.
    let some_variant = variants
        .iter()
        .find(|v| v.label == "SomeSourceSpan")
        .expect("SomeSourceSpan variant missing");
    let payload_decl = dag.declaration(some_variant.ty);
    if let TypeConnective::Conj { children } = &payload_decl.connective {
        let value_field = children
            .iter()
            .find(|f| f.label == "value")
            .expect("SomeSourceSpan payload missing `value` field");
        let source_span = dag
            .declaration_by_name("SourceSpan")
            .expect("SourceSpan missing from bootstrap");
        assert_eq!(
            value_field.ty, source_span.id,
            "SomeSourceSpan.value must resolve to `SourceSpan`, not a structural look-alike"
        );
    } else {
        // If the payload isn't a Conj-with-`value`, the variant shape
        // has drifted; fail loudly.
        panic!(
            "SomeSourceSpan variant payload should be a record with a `value: SourceSpan` field; \
             actual payload connective: {:?}",
            payload_decl.connective
        );
    }
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
import std.list { List, Empty, concat }
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
  Inhabits(_seed_list_provenance_empty())

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
