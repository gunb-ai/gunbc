# T-PB-B-2 — Predicate-gapped "G" list (Testgen backlog feed)

**Owner.** T-PB-B child (`session/neat-swift-804`).
**Status.** Non-landing brief. Parallel-safe vs all code PRs.
**Consumer.** Testgen manager (`docs/briefs/r1-testgen-manager.md`)
— backlog input for post-runner schema extensions.

## Scope

This brief enumerates the Rust-authored `v3` integration tests
whose semantics **cannot** be expressed today as `.dag` `TestClaim`
values because the `TestPredicate` schema lacks the needed shape.
These are the "G" (predicate-Gapped) bucket of the T-PB-B-1
inventory — i.e. tests *outside* the TESTING.md §Post-R2 residuals
(compiler-internal `#[cfg(test)]` helpers; rustc/go/python boundary
tests) that still cannot port on the currently-landed runner.

The "D" (Directly portable) bucket — predicates that reduce to
`Compiles` / `FailsWithDiagnostic` / `LensOutputEquals` /
`DifferentialEquals` / `ExecuteCommand` / `ForAllTargets` /
`AlgebraicLaw` — is being handled under T-PB-B-1 and is not the
subject of this doc.

## What ships today

Runner predicates on `main` (per r1-testgen-manager working state):

- `Compiles` / `FailsWithDiagnostic` — outcome on whole program.
- `ExecuteCommand`, `ForAllTargets` — target-toolchain boundary.
- `LensOutputEquals`, `DifferentialEquals` — lens-level equality.
- `AlgebraicLaw` (declared; runner evaluation WIP).
- `MockBackedInvariant` (PR #722 in review).

All of these treat the compiled artifact as a **black box** or
compare **lens outputs**. None of them can pose the question
"after compile, does the resulting `Dag` contain a `Bind(y)` whose
value is a `Transform` whose `TransformTarget::Callable` resolves
to the declaration named `negate`, with a literal-5 input on the
injected port?" — which is the characteristic shape of the G set.

## Canonical gap — `pipe_desugar.rs`

`src/v3/compiler/tests/integration/pipe_desugar.rs` (5 tests,
~22 helpers) is the prototype G module. Each test:

1. Compiles a short source string.
2. Walks `Dag.nodes()` for a `Bind` by name.
3. Follows that bind's value to the producing `Transform`.
4. Asserts the `TransformTarget` kind and the resolved-declaration
   identity (`Callable`, `FieldProject { field_label }`,
   `Operator(op_kind)`).
5. Follows input ports back to producer `Value` nodes and asserts
   on `LiteralBits`.

A `.dag` `TestClaim` for `pipe_desugars_unary_call_by_injecting_
the_left_value` would need to say, in schema: *compile S, then
prove path-of-Behaviors exists with these labels/identities.*
That is a **structural query over the post-compile substrate**,
not an outcome predicate.

## Needs-schema list (Testgen backlog)

Each item names a `TestPredicate` shape we cannot currently
evaluate and the G-bucket it unlocks. These are proposal shapes;
exact names/fields are Testgen's call.

1. **`BindExists { program, name }`** — closes the "find a named
   Bind in the compiled Dag" step used by every pipe_desugar test
   and by the `m1_lens_structural_resolution`, `m2_field_access_
   binding`, and `m2_lens_*_migration` modules.

2. **`BindValueIsTransformTo { program, bind_name, target }`**
   with `target ∈ { CallableNamed(String) | FieldProjection{label}
   | Operator(OpKind) }`. Closes the `assert_target_name` helper
   pattern — the dominant assertion in pipe_desugar and in the
   four `m2_lens_*_migration` suites.

3. **`TransformInputIsLiteral { program, bind_name, port_index,
   literal }`** — closes the `literal_input` helper; needed for
   pipe-left-injection, comparison-feeding, and the field-access
   binding tests.

4. **`NodeCountByBehavior { program, behavior_kind, count_rel }`**
   where `count_rel ∈ { Equals | AtLeast | AtMost }`. Closes the
   substrate-walk assertions in `m1_substrate_test.rs` (91 tests)
   and `lane2_stage_2b_db18_test.rs`. Without this, those 91
   imperative walks block port.

5. **`DeclarationResolvedByStructure { program, name }`** — closes
   the `AtomPayload::ResolvedByStructure(..)` / `ResolvedByName(..)`
   follow-through in `pipe_desugar::assert_target_name` and in
   `m1_fn_external_body_reconciliation_test.rs`.

6. **`PortShapeEquals { program, bind_name, port_index,
   type_shape }`** — generalizes `primitive_shape` and is the
   port-typing analog of #2. Closes type-resolution claims in
   `m1_5_verification_test.rs` and the structural-resolution
   migration modules.

7. *(retracted after review, 2026-04-24)* A prior draft proposed
   `CompileErrorAtPhase` to pin the compile phase of a failing
   diagnostic. That would create a second authority for phase:
   `DiagnosticReference.kind` already discriminates
   `ParseError` / later-phase variants (see
   `src/v3/compiler/src/test_runner.rs:402`), and
   `FailsWithDiagnostic` already matches on that kind. G tests
   that pin "fails at parse" (e.g.
   `invalid_pipe_target_fails_closed_at_parse_time`) are in the
   **D bucket**, not here. If a real residual appears — span
   coverage or diagnostic-code identity beyond the kind tag —
   it should be filed as a distinct shape naming that specific
   gap, not as a phase-pinning predicate.

## G-modules this list unlocks (non-exhaustive)

Integration modules whose predicate needs are fully or partially
covered by the seven shapes above:

- `pipe_desugar.rs` (5) — #1 #2 #3 #5
- `m1_substrate_test.rs` (~91) — #4 #6
- `m1_lens_structural_resolution_test.rs` + its `m2_*_migration`
  pair — #1 #2 #6
- `m2_field_access_binding_test.rs` — #1 #2 #3
- `m2_lens_cost_migration_test.rs` / `m2_lens_idempotency_*` /
  `m2_lens_unused_parameters_migration_test.rs` /
  `m2_lens_variant_payload_migration_test.rs` — #1 #2 #6
- `m1_fn_external_body_reconciliation_test.rs` — #2 #5
- `m1_5_verification_test.rs` — #6
- `sg1_tokenize_authority_test.rs`, `sg2_parse_authority_test.rs`,
  `sg2c1_parse_tables_authority_test.rs`, `sg3_surface_reflection_
  consumer_test.rs` — #4

Rough order-of-magnitude: the six shapes above, if landed,
unblock >60% of the current integration-test file count for
`.dag` port. The remainder is the D bucket (already covered by
landed predicates) plus the Post-R2 Rust residuals.

## Non-asks

- No new `TestClaim` *variant* beyond the current kind; the six
  live shapes are new `TestPredicate` variants carrying a
  substrate query against the compiled Dag.
- No new carrier type in user-facing surface — `TypeShape`,
  `OpKind`, `Behavior` kind names already exist in v3.
- No parallel "test DSL" for Dag traversal: predicates should
  name **what** to prove, never **how** to walk (see
  `feedback_lenses_not_passes` and `feedback_no_textual_
  enforcement_bridges`).

## Hand-off

- File this brief as the Testgen backlog input referenced in
  `r1-selfhosting-manager.md` working-state T-PB-B row 3
  ("Identify and scope the two TESTING.md residual categories
  per-test") — the G list is the **complement** of that residual
  plus the D bucket.
- Testgen manager decides sequencing of the six live shapes against
  `MockBackedInvariant` (in-review) and `AlgebraicLaw` runner
  evaluation already on their working list.
- No Rust deletion, no `.dag` drafting against these shapes until
  Testgen lands (or explicitly pre-approves) the schema.
