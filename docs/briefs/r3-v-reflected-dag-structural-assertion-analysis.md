# R3 Verification — Reflected-Dag Structural Assertion Analysis

**Status:** PROPOSAL / research-only. This brief does not introduce a
`TestPredicate`, edit substrate, or assign implementation. Substrate-fact
introduction remains routed through `INVARIANTS.md` §P1 and the Substrate
Manager. It records Verification analysis for Director routing after #828
c4356309499.

## Scope

Director surfaced a repeated acceptance-test pattern: new bootstrap carriers
land with hand-Rust tests that inspect the regenerated Dag for type, field,
variant, reference, and row-list shape. SG-0 already names the dissolution
target for several files as `.dag` `TestClaim` coverage for reflected-Dag
structural assertions over `std/` types. This brief asks what capability would
let those checks move out of bespoke Rust without pretending the substrate
change is already authorized.

The adjacent reflection input is `reflect_program_dag_nodes_in_file` in
`lens_apply.rs`, locked by `docs/design-reflection-completeness.md`. That
function reflects program-Dag `Behavior` nodes into `FieldValue`; it is useful
input precedent, not proof that structural assertions are already first-class
`.dag` claims.

## Candidate Capability Shape

A research-tier capability could be named `ReflectedDagAssert`,
`DagShapeAssert`, or similar. The important shape is not the name:

- **Observed carrier:** a reflected bootstrap Dag or source-file-scoped Dag
  carrier, produced by the same authority family as
  `reflect_program_dag_nodes_in_file` once reflection is generated rather than
  hand-Rust.
- **Assertion program:** structural atoms such as `type X exists`, `type X is
  Conj with exactly fields [a, b]`, `field X.a points at declaration Y`,
  `type S is Disj with variants [A, B]`, `data row R instantiates T`, `row
  field f is a Reference`, and `no declaration in file F has value_body`.
- **Result:** pass/fail with diagnostics naming the missing or surplus
  structural element, e.g. `Operation.inputs expected Map<String, InputField>`
  or `MethodTemplateContract row 3 duplicate dag_method`.
- **Non-goal:** evaluating the reflected program. This is static Dag-shape
  inspection, not L4 target-vs-eval output parity and not L5 cross-target
  runtime behavior.

The atoms need exact-set forms, not only existence checks. Most current Rust
tests are ratchets against unauthorized growth as much as unauthorized absence.

## Reuse Versus New Variant

This does **not** fold cleanly into the newly ratified
`BinaryDimensionReportEquals` path as currently understood. That predicate is
a binary structural-equality carrier for `DimensionReport<C>` outputs, with
reflection-aware modifiers for TC1/TC2/TC3 strict-fire surfaces. Reflected-Dag
shape assertions are usually unary queries against a Dag's declarations and
rows. They could be forced into `BinaryDimensionReportEquals` only by first
defining a dimension that projects "actual Dag shape report" and an expected
shape report. That would reuse a comparison mechanism, but the missing
capability would still be the reflected-Dag query/report producer.

Therefore the likely routing is: either define a reflected-Dag shape report
producer that `BinaryDimensionReportEquals` can compare, or authorize a
specialized assertion predicate. Verification should not choose that substrate
shape autonomously.

## RustDagIsomorphism Adjacency

The dispatch cites a Substrate-side `RustDagIsomorphism` proposal from the
ROADMAP 2026-04-30 debt row. The live `ROADMAP.md` on this branch exposes the
older 2026-04-25 reflection debt section and SG-0 comments naming reflected-Dag
structural assertions; it does not contain the exact `RustDagIsomorphism` token
or 2026-04-30 heading.

Conceptually, these are adjacent but not identical:

- `RustDagIsomorphism` sounds like whole-Dag equivalence between a Rust-emitted
  or Rust-reflected Dag and a `.dag` authority.
- `ReflectedDagAssert` is a query/assertion surface over selected declarations,
  fields, variants, rows, and absence constraints.

They can collapse if Substrate's isomorphism work exposes reusable expected
shape reports and partial structural predicates. If `RustDagIsomorphism` is
only whole-Dag equality, then this remains a distinct consumer need that should
share reflection/query atoms with it rather than fork another walker.

## Migration Audit

| Hand-Rust acceptance | Fit | Notes |
|---|---:|---|
| `anthropic_schema_lockstep_test.rs` | Partial | V3 mirror checks over `Anthropic*` records/sums fit. The v2-source extraction and lockstep comparison against `dsl/extdeps/llm/anthropic.dag` is cross-source schema diffing, not just reflected bootstrap shape. The "no data rows or fns" leak check fits. |
| `canonical_lens_bridge_ratchet_test.rs` | No | This is a Rust-source ratchet over `test_runner.rs`: `include_str!` constants, `lens_decl.name` equality arms, and generic name lookups. It dissolves via PB-Runtime or a lens registry carrier, not Dag-shape assertion. |
| `lens_substrate_carrier_test.rs` | Strong | Exact field sets for `Lens`, `Diagnostic.kind`, closed-sum variants for `CompilerDiagnosticKind` / `AnyDiagnosticKind`, and single-field `LensInstanceKindWitness` are canonical structural atoms. |
| `method_registry_test.rs` | Partial-strong | `MethodDeclaration` and `MethodRef` shapes fit directly. Registry coverage over names scraped from `dsl/std/algebra.dag` requires either a structural source for those template names or a cross-source expected-set input; otherwise the current lexical extraction remains outside the capability. |
| `method_template_contract_test.rs` | Strong | Distinct declaration IDs, disjoint field sets, forbidden fields, `MethodEmitTemplate` variants, per-target row-list uniqueness, and higher-order row payload shapes are good examples for exact-set, reference, variant, and list uniqueness atoms. |
| `services_carrier_shape_test.rs` | Strong | `Operation`, `InputField`, `RestEndpointBinding`, `CallableRef`, `Map<String, InputField>`, path authority, and "no data rows in services.dag" are direct structural assertions. |
| `workflow_root_port_test.rs` | Partial | The `WorkflowRoot` carrier shape would fit, but the live tests assert `Dag::workflow_root_port()` behavior over compiled fixtures. That is accessor semantics, not merely reflected declaration shape. A future claim might express compile-and-fold structure, but not this basic shape predicate alone. |

TC1's deferred fixture test is adjacent but separate: it verifies
`SubstrateResearchDeferredClaim` runner scoping, not a generic Dag-shape claim.

## Research Finding

The highest-value slice is not "add one more bespoke predicate"; it is a
shared reflected-Dag query/report substrate that can feed either:

1. `BinaryDimensionReportEquals`, if the project wants all structural equality
   claims to route through the unified binary report predicate; or
2. a specialized Dag-shape assertion predicate, if diagnostics and exact-set
   query syntax are too different from dimension reports.

The first path is attractive for single-carrier discipline, but only if the
report producer is first-class and reusable. Without that producer, calling the
capability `BinaryDimensionReportEquals` would hide the real missing substrate
fact behind an equality envelope.

## Coordination Signal

Verification should surface this as a Substrate/Director decision point:

- If `RustDagIsomorphism` already intends reusable reflected-Dag shape reports,
  collapse this dispatch into that queue.
- If unified `BinaryDimensionReportEquals` becomes the comparison shell, define
  the reflected-Dag shape report producer as the load-bearing carrier.
- If neither is true, ratify a separate `ReflectedDagAssert`/`DagShapeAssert`
  capability before any `.dag` TestClaim authoring.

## Reflection-Completeness Residual

Lane 1 consumers under T-V-L4-L7-Direct should not treat post-#1170 reflection
completeness as a full substrate proof. The live implementation still routes
through hand-Rust `lens_apply.rs::substrate_reflection::reflect_behavior_list`,
even though `docs/design-reflection-completeness.md` locks the intended full
`Behavior` shape.

That matters for L4/L7 standby implementation: a claim that consumes
`reflect_program_dag_nodes_in_file` can rely on the documented structural
contract, but closure should wait for a generated conformance walker or
Evaluator-backed substrate projection that proves every nested `FieldValue`
tree follows the declared substrate shape. Until then, reflection completeness
is a strong hand-Rust acceptance surface, not a mechanical theorem.

The live ROADMAP reflection row is `ROADMAP.md` §"Post-merge debt
(2026-04-25 reflective + exploratory analyses)" / "Lossy user-lens reflection
vs full substrate"; Director's #828 c4356309499 records the newer residual
routing. This note is consumer-side awareness only.

## Non-Claims

- No new `TestPredicate` is proposed here.
- No substrate edit is authorized here.
- No hand-Rust test is declared obsolete until the generated `.dag` claim path
  exists and each migrated acceptance surface has an equivalent diagnostic.
- No L4/L5/L7 lane is closed by this research note.
