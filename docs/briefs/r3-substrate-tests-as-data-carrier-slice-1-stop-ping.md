# R3 Substrate — Tests-as-Data Carrier Slice 1 STOP+PING

**Status:** STOP+PING — slice 2 (`SymbolicCostExprEquals`) landed in this PR; slice 1 (`TestSuite.claims` migration to `List<SuiteClaim>`) STOPs for scope confirmation before authoring.

**Authority:** Verification audit #1659 / `docs/design-tests-as-data-completeness.md` §2 / §7. Dispatch on parent inbox #1130.

## What's in scope for slice 1

The design-doc-locked carrier additions:

```dag
type ProgramGenerator { generator: DeclarationRef }

type ProgramShape
  = LiteralProgram { source: String, file_name: String }

type Quantifier
  = ForAll
  | Exists

type QuantifiedTestClaim {
  name: String
  generator: ProgramGenerator
  quantifier: Quantifier
  predicate: TestPredicate
  requires: List<ResourceReference>
}

type SuiteClaim
  = Enumerated(TestClaim)
  | Quantified(QuantifiedTestClaim)

type TestSuite {
  name: String
  claims: List<SuiteClaim>   // was: List<TestClaim>
}
```

## Why STOP rather than land in this PR

The carrier additions themselves are ~25 lines. The migration that the dispatch goal requires — *"including migration of existing `TestSuite.claims` authoring to `Enumerated(...)` and bootstrap/manifest fallout"* — touches a much larger surface than a small Substrate slice:

1. **31 fixture files** declare `data <name>: TestSuite = { name: …, claims: [a, b, c] }` literals and would need every claim reference wrapped in `Enumerated(…)`. Inventory:
   - 6 dag-fixture authorities under `src/v3/compiler/tests/dag/`.
   - 9 fixture authorities under `src/v3/compiler/tests/fixtures/` (some auto-emitted from `*.template.dag` by `build.rs`).
   - 16 reference sites across `src/v3/compiler/tests/integration/*.rs` and other lane files that read `TestSuite.claims` shape.
2. **Runner shape change.** `test_runner::run_suite` walks `claims` as `FieldValue::List` of `FieldValue::Reference(id)` to bare `TestClaim` declarations (`src/v3/compiler/src/test_runner.rs:1896-1919`). Post-migration, each entry is `FieldValue::Variant { constructor: "Enumerated"|"Quantified", payload: [Reference(claim_id)] }`. The dispatch table reads the variant constructor, then projects to the wrapped declaration and routes:
   - `Enumerated(TestClaim)` → existing `run_claim` path.
   - `Quantified(QuantifiedTestClaim)` → new shape-only stub returning `NotYetImplemented` until Verification's `product_zero_linear` follow-up authors the eval (per dispatch's "runner evaluation remains Verification follow-up" allowance).
3. **Bootstrap snapshot regen + parse manifest refresh** ride along (mechanical fallout, not new authoring).
4. **No in-tree consumer of `QuantifiedTestClaim` exists yet.** Verification's `product_zero_linear` claim is the named first consumer (per dispatch §7 hand-off). Authoring carriers + migrating fixtures + introducing a runner stub *all at once with no concrete consumer to validate against* invites the same misshape risk #1634 review caught on L6 — far cheaper to confirm scope/shape with the consumer in mind first.

These together cross the dispatch's "small Substrate slice" line (parent inbox #1130: *"If you find the live suite surface makes this larger than a small Substrate slice, STOP+PING with exact blockers and a minimal first carrier PR"*).

## Minimal-first-carrier PR plan (what slice 1 would actually ship)

Single bounded PR, scope-confirmed before authoring:

1. **Substrate (`src/v3/std/verification.dag`):** add `ProgramGenerator`, `ProgramShape` (1-variant `LiteralProgram { source, file_name }`), `Quantifier` (`ForAll | Exists`), `QuantifiedTestClaim`, `SuiteClaim`. Change `TestSuite.claims` to `List<SuiteClaim>`.
2. **Fixture migration** (~31 files, mechanical sed `claims: [a, b, c]` → `claims: [Enumerated(a), Enumerated(b), Enumerated(c)]`). Auto-emitted fixtures are derived from `*.template.dag`; touch templates only.
3. **Runner (`src/v3/compiler/src/test_runner.rs`):** widen `run_suite`'s `FieldValue::List` walker to dispatch on the `SuiteClaim` variant constructor. `Enumerated(_)` → existing `run_claim`. `Quantified(_)` → shape-only stub (`NotYetImplemented` naming the missing eval).
4. **Bootstrap regen + parse manifest refresh** via the documented helpers (`regen_bootstrap` + `refresh_handwritten_parse_snapshot_manifest`).
5. **Ratchet tests** (mirror existing `bootstrap_loads_verification_authority_types` shape):
   - `bootstrap_loads_quantified_test_claim_carrier` — asserts the new types exist with the design-doc-locked field shape.
   - `every_test_suite_uses_enumerated_wrapper` — walks every `TestSuite` declaration in the bootstrap and confirms each `claims` entry's variant constructor is `Enumerated` (no bare `TestClaim` references survive). Single-shot ratchet that pins the migration.
   - `quantified_test_claim_runner_returns_not_yet_implemented` — focused fixture authoring a 1-element `LiteralProgram` generator + `QuantifiedTestClaim` with `quantifier: ForAll, predicate: Compiles`; runner returns `NotYetImplemented` with the documented missing-eval message. This pins the runner stub shape.
6. **No `coverage.rs`-style downstream rewrite.** Verification's `product_zero_linear` follow-up authors the runner eval; this PR only lands the substrate + the shape-only stub.

## Open sub-questions for sign-off before slice 1 PR

The §2 of the design doc resolves carrier shape; the §3 migration mechanics need scope confirmation:

### 1.A — Auto-emitted fixtures

`r1_gates.dag`, `r1_release_acceptance.dag`, `r1_pb_census_gates.dag`, etc. are emitted from `*.template.dag` by `build.rs`. Migration target: edit only the templates. Confirmation request: any template-versus-emitted-fixture check beyond the existing `r1_gates.template.dag → r1_gates.dag` build-time splice that would silently drift if I edit only templates?

### 1.B — Verification-side parallel reads

Some `tests/integration/*.rs` files synthesize `TestSuite { claims: [...] }` Rust-side via `FieldValue::List` literals (e.g., `r1_release_acceptance_test.rs`, `tc1_substrate_lens_eta_equivalence_deferred_test.rs`). Migration target: same wrapping rule applies in Rust constructors. Confirmation request: do those Rust-side synthesizers belong to the slice 1 PR (Substrate scope) or to the Verification follow-up (Verification scope)? My read: Substrate, since they are migration-required to keep the runner happy and the migration is mechanical.

### 1.C — `Quantified(_)` runner-stub shape

The runner stub for `SuiteClaim::Quantified` returns `NotYetImplemented` with a typed message naming the missing eval (per dispatch's allowance). Confirmation request: any preference for failing closed (`ClaimResult::Fail`) instead of `NotYetImplemented`? Existing scaffolds (`BinaryDimensionReportEquals`, `CensusBoundCheck`, `FixedPointConverges`, etc.) all use `NotYetImplemented`, so my proposed shape mirrors precedent.

### 1.D — `ProgramShape` 1-variant bootstrap

Per design doc §8.2, the substrate ships `ProgramShape` with `LiteralProgram` only and `feedback_groundedness_gates_lenses` covers extension via lens framework. The single-variant coproduct is structurally honest. Confirmation request: any preference for shipping the variant flat-recorded (`ProgramShape { source: String, file_name: String }`) instead of a 1-variant Disj? Design doc §8.2 already resolves this in favor of the 1-variant Disj — confirming nothing changed since.

## Hand-off chain (post sign-off)

- **Substrate (slice 1 PR):** carriers + migration + runner shape stub + ratchets per §1.A–§1.D resolutions.
- **Verification (`product_zero_linear` follow-up PR):** authors a `data product_zero_linear: QuantifiedTestClaim = …` value, defines the generator, and authors the runner eval (`eval_quantified_test_claim_for_all`) once the substrate slice lands.
- **Verification (heuristic-cost-function 5th-gate testgen PR):** consumes both `QuantifiedTestClaim` and the `SymbolicCostExprEquals` predicate landed in this PR.

## Out of scope (for slice 1 PR)

- Generated target-language test code path (design doc §4) — owned by Verification per §7.
- Cementing test discipline migration (design doc §5) — separate dispatch.
- Lens capability register migration to `.dag` (design doc §8.3) — separate dispatch.

---

— sent from tidy-tern-769 (inbox #1288); reply at #1130
