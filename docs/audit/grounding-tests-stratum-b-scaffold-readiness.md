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

**Post-#1465 refresh:** #1465 landed string-family axis vocabulary in
`src/v3/std/emit_model.dag`: `StringOwnershipAxis`, `StringLifetimeAxis`,
`StringGrowabilityAxis`, and `StringEncodingAxis`. This partially satisfies the
axis-vocabulary trigger for string-family rows, but it does not land per-target
string-family rows, program binding/use extraction, or production Coercion-Fold
projection. Quiet-otter's parallel #1442 refresh on the namespace-vs-shared
design call is pending at this audit refresh time; this document therefore
cross-references the merged #1442 pre-audit plus the #1465 ground truth directly.

The honest readiness split is narrow:

- **Already landed:** Slice 1 low-drift helper organization and typed
  prerequisite diagnostics (`#1468`).
- **Dispatch-ready now:** a narrow string-family-specific forward lockstep
  ratchet can be authored against #1465 axis sums without asserting fold outputs.
- **Substrate-gated:** any Stratum B test that claims a target inhabitance from
  Int or String examples, because the production fold still lacks declared Int
  projection rows, program-bound lowering, per-target string-family rows, and
  binding identity.

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
| `rust_method_template_contracts`, `python_method_template_contracts`, `go_method_template_contracts` | Row lists exist in `generated_full_bootstrap_dag()` and have Director-locked Phase 1 counts: Rust 13, Python 17, Go 13. |
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

### String-axis vocabulary ground truth after #1465

#1465 added four closed string-family axis sums under
`src/v3/std/emit_model.dag`:

| Axis | Variants | Notes |
|---|---|---|
| `StringOwnershipAxis` | `Owned`, `Borrowed` | Target-row property for string-family candidates, distinct from callable `ParameterDisposition`. |
| `StringLifetimeAxis` | `SelfContained`, `Caller` | `SelfContained` avoids leaking Rust's `Self_` spelling into substrate vocabulary. |
| `StringGrowabilityAxis` | `Growable`, `Fixed`, `NotApplicable` | `NotApplicable` is explicit and cannot be represented as field absence. |
| `StringEncodingAxis` | `Utf8FreeMonoidChar` | Names the `.dag String = FreeMonoid<Char>` / UTF-8 row shape for R2. |

The landing also added compiler integration ratchets:

- `string_diagnostic_ordering_axes_live_in_emit_model_authority` asserts all
  four axis declarations live in `src/v3/std/emit_model.dag`.
- `string_diagnostic_ordering_axes_are_closed_structural_values` asserts the
  exact closed variant lists.

Boundaries from #1465 remain load-bearing: axis vocabulary only; no
`TypeRealization` extension; no `std/lifetime.dag`; no per-target string-family
target rows; no Grounding projection reader.

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
`docs/audit/lifetime-axes-canonical-vocabulary-spec.md` show the same split,
updated by #1465's partial landing:

| Requirement | Readiness |
|---|---|
| Lifetime analyzer fixture logic | **Ready as fixture logic.** It can derive ownership, lifetime, growability, and encoding from `LifetimeProgram` test inputs. |
| `Dag` to `LifetimeProgram` extraction | **Gated.** Bootstrap extraction returns empty; runtime binding/use extraction is pending. |
| String-family axis substrate vocabulary | **Ready for string-family scope.** #1465 declares string-namespaced axes in `emit_model.dag`; this is enough for a string-specific forward lockstep ratchet. |
| General canonical lifetime-axis substrate decision | **Still pending.** #1465 chose string-namespaced axes, not a general `Ownership` / `LifetimeScope` / `Growability` vocabulary shared outside string-family diagnostic ordering. Quiet-otter's post-#1465 #1442 refresh should own that namespace-vs-shared design call. |
| String-family target rows | **Gated.** Rows for Rust `String`, `Box<str>`, `&str`, `Cow<str>` and analogous target facts are absent. |
| Binding identity contract | **Gated.** Coercion-Fold and Lifetime-Analyzer need a shared binding key before fold tests can consume reports structurally. |

Conclusion: Stratum B can now scaffold a **string-family-specific**
`string_axes_lockstep` forward ratchet over #1465's namespaced axes. It still
cannot honestly assert Example 3/4 fold selection over declared string rows.

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
| Create `stratum_b.rs` with only typed readiness/checklist helpers and no production assertions | Done in #1468 | Low | The module names gates and exposes helper seams without fabricating rows. |
| Add a Stratum B fixture contract doc / module comment naming `DeclaredLanguageSpecProjection` expectations | Done in #1468 | Low | The module documents readiness-only scope. |
| Add tests that assert current Stratum B gates are absent | Maybe | Medium | Useful only if phrased as readiness guards. Brittle if they key on exact missing declaration names likely to change in the substrate PR. |
| Add string-family axis lockstep now | Yes | Low | #1465 declares `StringOwnershipAxis`, `StringLifetimeAxis`, `StringGrowabilityAxis`, and `StringEncodingAxis`; a forward ratchet can assert future string-family Rust mirrors stay subsets of these sums. |
| Add general `lifetime_axes_lockstep.rs` now | No | High | General shared axis vocabulary remains a design question; do not turn #1465's string-namespaced axes into a global authority. |
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

**Landed in #1468.**

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

Post-#1465 update for the landed readiness scaffold:

- `StratumBPrerequisite::LifetimeAxisVocabulary` should **split** in a small
  follow-up if code is amended:
  - `StringFamilyAxisVocabulary` = ready when #1465 axis sums are present;
  - `GeneralLifetimeAxisVocabulary` = pending until the namespace-vs-shared
    design call lands.
- `StringFamilyRows` detail should mention that future rows should reference
  `StringOwnershipAxis`, `StringLifetimeAxis`, `StringGrowabilityAxis`, and
  `StringEncodingAxis` values structurally.
- Keeping the single `LifetimeAxisVocabulary` as `PendingExternal` is now too
  coarse for dispatch planning, but it is not behaviorally wrong because no
  production Stratum B assertion consumes it yet.

### Slice 2a: String-family Axis Lockstep Ratchet

**Dispatchable now.**

Add a forward ratchet in `v3-grounding-tests` that reads #1465 substrate sums
from `generated_full_bootstrap_dag()`:

- `StringOwnershipAxis`
- `StringLifetimeAxis`
- `StringGrowabilityAxis`
- `StringEncodingAxis`

The ratchet should compare only string-family mirrors that actually exist. At
HEAD, `grounding_lifetime/src/facts.rs` has conceptually aligned but differently
named program-side enums (`Ownership`, `LifetimeScope`, `Growability`,
`Encoding`), not string-family target-row mirrors. Therefore the first ratchet
should be a **forward structural ratchet**:

- assert #1465 axes exist under `emit_model.dag` with the closed values from the
  compiler integration test;
- expose the future mirror mapping table in code comments or test names;
- avoid requiring `LifetimeScope::Self_` to be a literal subset of
  `StringLifetimeAxis::SelfContained`, because that would conflate Rust enum
  escaping with substrate naming.

This is useful because it gives Stratum B a local gate that will bite if #1465
axis authority moves or reshapes before target rows arrive, without pretending
that program-side lifetime facts are generated from substrate yet.

### Slice 2b: General Lifetime Axis Lockstep Ratchet

**Deferred until the canonical/shared vocabulary decision lands.**

If the quiet-otter #1442 refresh or Substrate Manager later authorizes a general
axis vocabulary outside string-family diagnostic ordering, add
`lifetime_axes_lockstep.rs` in `v3-grounding-tests`:

- read substrate shared `Ownership`, `LifetimeScope`, `Growability`, and
  `Encoding` sum variants from `generated_full_bootstrap_dag()`;
- parse lane-local `grounding_lifetime/src/facts.rs` enum variants;
- assert mirror variants are subsets of substrate labels;
- include a synthetic negative control matching the existing
  `emission_diagnostic_lockstep.rs` pattern.

This remains low drift after a shared vocabulary lands and high drift before it.

### Slice 3: Real Stratum B Assertions

**Deferred until Coercion-Fold Slice C prerequisites land.**

After program-bound lowering, per-target Int rows, algebra intent facts,
string-family rows, and shared binding identity are present:

- call the production fold through a declared projection arm;
- assert Int Examples 1, 2, 5, 6, and 8 by selected row identity or typed
  diagnostic;
- assert String Examples 3 and 4 by selected row identity plus propagated
  lifetime facts; the target rows should reference the #1465
  `String*Axis` values structurally unless a later Substrate decision replaces
  them with a shared vocabulary;
- add cross-stratum consistency only for rows that have both registry-backed
  and structural candidates;
- add Q4 witness checks only after the relevant `Lens<C>` witnesses are
  declared.

This is the slice that can eventually close the Stratum B portion of
`routing_correctness_l4_verified`.

## Recommendation

Proceed with **Slice 2a only** if a code PR is desired today. It is a narrow
string-family-specific forward ratchet over the #1465 axis sums and should not
reshape when target rows land.

Also consider a small Slice 1 amendment to split the readiness prerequisite into
`StringFamilyAxisVocabulary` (ready) and `GeneralLifetimeAxisVocabulary`
(pending). That amendment is not required before Slice 2a because Slice 2a can
read #1465 directly.

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
- PR #1465, `feat(v3): add string diagnostic axis vocabulary`
- PR #1468, `feat(grounding-tests): Stratum B readiness scaffold — typed prerequisite diagnostics (#1464 audit Slice 1)`
