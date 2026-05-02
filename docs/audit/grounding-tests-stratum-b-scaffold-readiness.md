# Grounding Tests Stratum B Scaffold Readiness

**Date:** 2026-05-02  
**Lane:** T-Ground-Tests  
**Scope:** audit / dispatch spec only. No code changes.

## Summary

`v3-grounding-tests` already has a real Stratum A scaffold: it walks the
embedded bootstrap `Dag`, projects `MethodTemplateContract` row lists, enforces
closed record schemas, resolves `MethodRef` through the method registry, and
checks deterministic row digests for the Phase 1 row authorities. Stratum B,
per `docs/briefs/t-ground-tests.md`, should add algebra-homomorphism routing
assertions over production Coercion-Fold outputs plus Q4 structural
certification.

The honest readiness split is narrow:

- **Dispatch-ready now:** low-drift helper organization and ratchets that only
  name existing or already-specified surfaces.
- **Substrate-gated:** any Stratum B test that claims a target inhabitance from
  Int or String examples, because the production fold still lacks declared
  projection rows, program-bound lowering, string-family rows, and canonical
  lifetime-axis substrate vocabulary.

This means a useful pre-scaffold can land, but it must stop before introducing
mock Stratum B candidate rows or reifying `ScratchIntExamples` as if it were the
production fold.

## Stratum A Current State

Files:

- `src/v3/grounding_tests/src/stratum_a.rs`
- `src/v3/grounding_tests/src/diagnostic.rs`
- `src/v3/grounding_tests/src/emission_diagnostic_lockstep.rs`
- `src/v3/grounding_tests/src/integer_diagnostic_order.rs`
- `src/v3/grounding_tests/src/lib.rs`

Current facts asserted:

| Surface | Current assertion |
|---|---|
| `rust_method_template_contracts`, `python_method_template_contracts`, `go_method_template_contracts` | Row lists exist in `generated_full_bootstrap_dag()` and have Director-locked Phase 1 counts: Rust 13, Python 18, Go 14. |
| `MethodTemplateContract` rows | Each row is a closed record with exactly `dag_method`, `emit_template`, `placeholder_convention`, `runtime_template`, and `wraps_result`. Duplicate, missing, and extra fields fail closed. |
| `MethodRef` | `dag_method` must be a `MethodRef { decl }` record, and `decl` must reference a declaration that instantiates `MethodDeclaration`. |
| Templates | `runtime_template` must be a string, `emit_template` must be a known `MethodEmitTemplate` variant, `PlaceholderConvention` must be a known nullary variant, and `wraps_result` must be bool. |
| Determinism | Row fingerprints are keyed through `BTreeMap`; tests compare forward vs reverse row walks and re-run digests over the full bootstrap. |
| Diagnostics | Failures are typed as `GroundingTestsDiagnostic`, lane-local because the payloads are test-outcome coordinates rather than fold/emission diagnostics. |
| Lockstep ratchets | `EmissionDiagnostic` mirrors must stay subsets of the substrate sum, and integer diagnostic order rows must lower to the expected `DeclarationRef` chains. |

Discipline that already holds:

- Stratum A reads substrate authority through `v3_compiler::generated_full_bootstrap_dag()`, not text includes of row files.
- It is fail-closed on schema drift.
- It does not touch `src/v3/compiler/`.
- It keeps test-side diagnostics lane-local pending a specific carrier decision.

What Stratum A does **not** assert today:

- It does not call the production Coercion-Fold body.
- It does not prove algebra-homomorphism target selection.
- It does not consume `BoundDeclaration` or lifetime facts.
- It does not walk Q4 `Lens<C>` witnesses.

## Stratum B Intended Scope

Per `docs/briefs/t-ground-tests.md`, Stratum B is the
algebra-homomorphism extension of Stratum A:

1. For programs with no name-keyed registry entry, the fold selects the unique
   `(algebra x refinement)` target inhabitance.
2. Selection is deterministic under non-substrate-state perturbation.
3. Overlaps between Stratum A registry entries and Stratum B structural
   candidates agree.
4. Q4 structural witnesses certify Faithful, Correct, Minimal, and Performant
   properties per selected inhabitance.
5. Under-refinement and no-inhabitant cases surface typed diagnostics rather
   than skipped tests.

The seed examples are the design-emission-model examples already tracked by the
Coercion-Fold and Lifetime-Analyzer audits:

- Int-family Examples 1, 2, 5, 6, and 8.
- String/lifetime Examples 3 and 4.
- Later collection/map/string operation surfaces after LanguageSpec operation
  ontology is collapsed through `MethodTemplateContract` rows.

## Substrate Gap Analysis

### Int-family Stratum B

`docs/audit/scratch-int-examples-dissolution-spec.md` and
`docs/audit/scratch-int-examples-slice-c-prep.md` agree on the current state:

| Requirement | Readiness |
|---|---|
| `BoundDeclaration` carrier | **Ready.** #1449 landed `StaticBound(Interval<Int>) | PlatformDependent`. |
| Program-bound lowering | **Gated.** `Int(lo..hi)`, `Int(any)`, and `Int(platform)` are not lowered into `BoundDeclaration` facts for the fold. |
| Per-target Int inhabitance rows | **Gated.** Rust/Python/Go do not yet carry the required full integer-family rows with algebra and `BoundDeclaration` facts. |
| Algebra intent | **Gated.** Example 5 needs declared Semiring vs OrderedRing intent or a structural ambiguity diagnostic. |
| Production declared projection reader | **Gated after rows.** Grounding owns this reader only after the rows and program-bound facts exist. |

Conclusion: Stratum B must not add tests that assert `RustU32`, `RustI32`,
`PythonInt`, or `GoInt32` from declared facts yet. A test around
`ScratchIntExamples` would only certify the transitional selector enum and would
reshape when Slice C lands.

### String / Lifetime Stratum B

`docs/audit/coercion-fold-lifetime-analyzer-convergence.md` and
`docs/audit/lifetime-axes-canonical-vocabulary-spec.md` show the same split:

| Requirement | Readiness |
|---|---|
| Lifetime analyzer fixture logic | **Ready as fixture logic.** It can derive ownership, lifetime, growability, and encoding from `LifetimeProgram` test inputs. |
| `Dag` to `LifetimeProgram` extraction | **Gated.** Bootstrap extraction returns empty; runtime binding/use extraction is pending. |
| Canonical lifetime-axis substrate vocabulary | **Gated.** The audit specifies substrate sums, but no `.dag` authority has landed. |
| String-family target rows | **Gated.** Rows for Rust `String`, `Box<str>`, `&str`, `Cow<str>` and analogous target facts are absent. |
| Binding identity contract | **Gated.** Coercion-Fold and Lifetime-Analyzer need a shared binding key before fold tests can consume reports structurally. |

Conclusion: Stratum B can scaffold a future `lifetime_axes_lockstep` ratchet
shape after the substrate vocabulary lands, but today it cannot honestly assert
Example 3/4 fold selection over declared string rows.

### Operation-template Stratum B

`docs/audit/collection-ops-string-ops-map-ops-duplicate-fact.md` is adjacent
rather than a direct gate for Int/String routing tests.

Readiness:

- The `MethodTemplateContract` bridge exists and Stratum A already consumes it.
- Missing method identities, missing per-target rows, construction-syntax
  classification, and optional algebra/profile coordinates remain open.
- A #1424-style ratchet against new consumers of legacy `CollectionOps`,
  `StringOps`, and `MapOps` operation fields is dispatchable after the method
  identity / row-population subset is chosen.

Conclusion: operation-template Stratum B work should not be bundled into the
first algebra-homomorphism scaffold. It is a later LanguageSpec/consumer
migration slice, with Stratum A row-walking patterns reusable.

## Pre-scaffold Work Dispatchable Now

| Item | Dispatchable today? | Drift risk | Notes |
|---|---:|---:|---|
| Create `stratum_b.rs` with only typed readiness/checklist helpers and no production assertions | Yes | Low | Helpful if it names gates and exposes helper seams without fabricating rows. Must not call `ScratchIntExamples` as production. |
| Add a Stratum B fixture contract doc / module comment naming `DeclaredLanguageSpecProjection` expectations | Yes | Low | Mirrors Slice C prep; can be doc-only or test-helper TODOs. |
| Add tests that assert current Stratum B gates are absent | Maybe | Medium | Useful only if phrased as readiness guards. Brittle if they key on exact missing declaration names likely to change in the substrate PR. |
| Add `lifetime_axes_lockstep.rs` now | No | High | No substrate axis sums exist yet. A textual-only ratchet would invent the authority it is meant to follow. |
| Add Stratum B tests over `ScratchIntExamples` | No | High | Would certify hardcoded selector variants and then reshape under declared projection. |
| Add in-memory fake Int/String candidate rows for algebra-homomorphism selection | No | High | Risks authoring a parallel LanguageSpec row shape in Rust before Substrate lands the real row schema. |
| Generalize Stratum A closed-record helpers for reuse | Yes | Low | Pure helper extraction is safe if it does not change behavior. |
| Add a #1424-style legacy-operation consumer ratchet | Not in this slice | Medium | Dispatchable after operation identity and row-population subset is chosen; not a Stratum B core prerequisite. |

## Drift Risk Assessment

High-drift pre-scaffold items share one shape: they encode the missing substrate
row schema locally. That includes fake candidate structs, fake bound payloads,
string axis labels, and tests over `TargetInhabitance` enum variants. Those
would become parallel authorities as soon as Substrate lands the real rows.

Low-drift items are purely organizational or fail-closed:

- module boundaries;
- diagnostics for "Stratum B prerequisite missing";
- helper contracts that keep identity `DeclarationId` / `DeclarationRef` keyed;
- reuse of closed-record parsing discipline;
- doc comments that point back to the five upstream audit files.

The first implementation PR should therefore be deliberately small. It should
make future Stratum B insertion easier without asserting target-routing behavior
until declared facts exist.

## Slice Spec

### Slice 1: Stratum B Readiness Scaffold

**Dispatchable now.**

Add a `stratum_b` module that exports a small readiness API and typed diagnostic
variant(s) for missing prerequisites. The module may:

- document the required declared projection contract;
- expose a `StratumBPrerequisite` enum for the known gates;
- provide a function that reports gates still absent at HEAD by checking for
  substrate declarations that are already expected to exist, such as
  `BoundDeclaration`, and by naming still-external gates as not-yet-checkable;
- reuse Stratum A closed-record helper patterns only if moved without behavior
  change.

It must not:

- build mock target inhabitance rows;
- assert Examples 1-8 outcomes from `ScratchIntExamples`;
- normalize lifetime axes through strings;
- touch `src/v3/compiler/`.

Tests should be limited to the readiness API and deterministic formatting of
diagnostics. They should not close `routing_correctness_l4_verified`.

### Slice 2: Lifetime Axis Lockstep Ratchet

**Deferred until Substrate lands canonical axis sums.**

When the vocabulary from
`docs/audit/lifetime-axes-canonical-vocabulary-spec.md` lands, add
`lifetime_axes_lockstep.rs` in `v3-grounding-tests`:

- read substrate `Ownership`, `LifetimeScope`, `Growability`, and `Encoding`
  sum variants from `generated_full_bootstrap_dag()`;
- parse lane-local `grounding_lifetime/src/facts.rs` enum variants;
- assert mirror variants are subsets of substrate labels;
- include a synthetic negative control matching the existing
  `emission_diagnostic_lockstep.rs` pattern.

This is low drift after substrate lands and high drift before it.

### Slice 3: Real Stratum B Assertions

**Deferred until Coercion-Fold Slice C prerequisites land.**

After program-bound lowering, per-target Int rows, algebra intent facts,
string-family rows, and shared binding identity are present:

- call the production fold through a declared projection arm;
- assert Int Examples 1, 2, 5, 6, and 8 by selected row identity or typed
  diagnostic;
- assert String Examples 3 and 4 by selected row identity plus propagated
  lifetime facts;
- add cross-stratum consistency only for rows that have both registry-backed
  and structural candidates;
- add Q4 witness checks only after the relevant `Lens<C>` witnesses are
  declared.

This is the slice that can eventually close the Stratum B portion of
`routing_correctness_l4_verified`.

## Recommendation

Proceed with **Slice 1 only** if a code PR is desired today. It is small,
mechanical, and will not reshape when Substrate lands.

Do not land target-selection Stratum B tests yet. They would either read
`ScratchIntExamples` and certify the wrong surface, or they would invent local
candidate rows that duplicate the LanguageSpec authority still being queued
through Substrate / `#1130`.

## References

- `docs/briefs/t-ground-tests.md`
- `src/v3/grounding_tests/src/stratum_a.rs`
- `src/v3/grounding_tests/src/emission_diagnostic_lockstep.rs`
- `src/v3/grounding_tests/src/integer_diagnostic_order.rs`
- `docs/audit/scratch-int-examples-dissolution-spec.md`
- `docs/audit/scratch-int-examples-slice-c-prep.md`
- `docs/audit/coercion-fold-lifetime-analyzer-convergence.md`
- `docs/audit/lifetime-axes-canonical-vocabulary-spec.md`
- `docs/audit/collection-ops-string-ops-map-ops-duplicate-fact.md`
