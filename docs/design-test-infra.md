> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 2 Stage 2c (test obligation materialization) | Status: **discussion draft — not locked**

# Design DB-15 (draft) — Minimal compositional test modeling

**Design blocker:** DB-15 (test infrastructure that rides on the compiler's dependency graph)
**Consumer:** Lane 2 Stage 2c (test obligation materialization) — forcing function. Also benefits every other lane.
**Status:** Draft for discussion. Sharp corners deliberate.

---

## Why a design doc now

Test cost is ballooning and the compiler's dependency-analysis machinery is not reaching the test framework. Concretely today:

- Each `#[test]` fn independently invokes `compile_to_dag(src, file)`. Bootstrap cached (#492); user-program lowering not cached across tests with identical source.
- m2 migration tests compile a roundtrip harness binary via rustc per test file (3× for three files). Harness bodies are near-identical.
- Fixture strings like `"fn add(a: Int, b: Int) -> Int = a + b"` appear hand-authored in multiple test files. Each tokenized, parsed, lowered independently.
- Every migration test writes its own hand-written Rust oracle that walks the DAG.

Lane 2 Stage 2c ("test obligation materialization") generates tests from compile-time proof dimensions. Without infrastructure that shares work, generation multiplies the per-test cost and the suite becomes intractable.

**Thesis claim that's load-bearing here:** gunbc's extensive dependency analysis should make tests *more efficient than typical*, not less. Today the opposite is true — because tests don't ride on the machinery.

---

## Design thesis

**Test fixtures are declared intent, not authored source.** Two tests expressing the same intent share the fixture by construction. Generation falls out of lens outputs.

The test framework is a consumer of the same substrate the compiler already has. What's missing is the contract that routes tests through the substrate instead of around it.

---

## Minimal model

### TestCase as a declaration

```dag
type TestCase {
  subject: DeclarationRef      // what's being tested — a reference, never a source string
  claim: DeclarationRef        // what's asserted about the subject
  expected: Expectation        // Holds | FailsWith(diagnostic_ref) | ProducesValue(...)
}

data test_ring_int_add_commutative: TestCase = {
  subject: ring_int_add
  claim: commutativity
  expected: Holds
}
```

Fixtures do not exist as inline string literals. `subject` is a reference to an existing declaration — either in the program under test, in `std/`, or in a test-module fixture file that itself composes existing declarations.

### Claims are declarations

```dag
type Claim {
  lens: DeclarationRef         // which lens proves this claim
  predicate: ClaimPredicate    // what the lens output must match
}

data commutativity: Claim = {
  lens: parallelism_lens
  predicate: IsCommutativeMonoid
}
```

A claim says "the named lens, run over the subject, produces an output matching this predicate." Running the test is: look up the lens, run it over the subject, compare the output.

### Compositional sharing (falls out of the model)

| Source of duplication today | Shared because | Cache key |
|---|---|---|
| Identical source strings across test files | Source comes from the referenced declaration — one declaration, many tests | `declaration_id` |
| Same program compiled twice | DAG compilation cached on source hash | `hash(source)` |
| Same lens run twice over equivalent subjects | Lens results cached on `(DAG hash, lens id)` | `(DAG_hash, lens_decl_id)` |
| Hand-written oracle walks | The lens IS the oracle; tests consume its output directly | n/a — duplication dissolves |
| Per-file harness compilation | One test-runner binary reads all `TestCase` declarations | n/a |

**Runtime cost invariant the framework establishes:**
> A test suite's cost is `O(distinct subjects × distinct lenses)`, not `O(test_count)`.

For a well-composed suite, those factors grow much more slowly than raw test count. This is where "more efficient than typical" cashes out.

### Generation from lenses (Lane 2 Stage 2c's case)

Lenses produce facts — `lens_provenance` labels every port's origin, `lens_cost` computes a cost map, `lens_unused_parameters` flags unused params. A test obligation materializes the inverse: for each fact the lens produces, emit a `TestCase` declaration that asserts it.

Stage 2c walks declared lenses and emits `data test_X: TestCase` declarations. The framework runs them without per-test compilation overhead because subjects are shared and lens caches are warm.

### Pipeline catches duplicate work — structurally

The user's core question: "can our pipeline catch duplicate work — uncached entries, unshared infra?"

Three properties the substrate already has that make caching sound:

1. **§8.9 inhabitance walk is deterministic on DAG structure.** Two programs with identical structural DAGs produce identical lens outputs. Hash-keyed caching is sound by construction, not by convention.
2. **Bind pass-through + `resolve_producer` dedupe at dispatch.** Two tests that compile `let x = 1 + 2` produce structurally-equal Transforms; the compiler already treats them as one thing internally — the cache just needs to observe it.
3. **Lens results are pure functions of (DAG, lens).** Cache lifetime is the test run; invalidation is trivial.

The CACHES do not exist yet. The SHAPES do. DB-15 wires the caches.

---

## Open questions (please tear into)

1. **Subject composition.** Can `subject` reference a *synthesized* declaration (e.g., `compose(ring_int_add, pipe_reverse)`) or only pre-existing ones? Synthesized would let tests declare "the subject is the composition of these two primitives," which is compositionally cleaner but adds a small lowering step. Preference: yes, allow synthesized — it's cheap and is exactly how fixtures should compose.

2. **Claim vocabulary.** `predicate: ClaimPredicate` needs a concrete type. Open-ended `String` is weak; a Disj over predicate shapes (`IsCommutativeMonoid | HasCost(n) | ProducesValue(v) | ...`) is typed but grows a coproduct. Middle option: claim declarations declare their own predicate shape as data, and the framework walks declared predicate shapes. Probably that.

3. **Expectation modeling.** `Holds | FailsWith(diag_ref) | ProducesValue(v)` — is that enough? Needs a case for "compilation fails with this diagnostic" vs "compilation succeeds but lens disagrees" vs "program runs and produces value." Currently three; might grow.

4. **Coexistence with existing Rust tests.** During transition, both worlds exist. What's the boundary? Probably: Rust tests that don't route through the framework are OK but must migrate when they touch generated tests. Explicit deprecation path once the framework proves out.

5. **Test runner shape.** Is the runner a minimal Rust binary that walks `TestCase` declarations (bootstrap) or itself a compiled `.dag` program (self-consistent but bigger bite)? Bootstrap path wins for landability.

6. **Failure diagnostics.** When a claim fails, does each `TestCase` declare its own failure template or does the framework synthesize a generic one from (subject, claim, actual output)? Generic first; per-case override if needed.

---

## Out of scope for DB-15

- Porting existing Rust tests. Transition is gradual; DB-15 defines the shape, not the migration.
- IDE integration, test selection UI, coverage metrics.
- Property-based / fuzzing generation. Different concept; layer later if useful.
- Performance ratchets. Those land with the cache implementation, not this design.

---

## Rejected alternatives

- **Sidecar caching layer on current Rust tests.** Memoize `compile_to_dag` calls in a `OnceLock<HashMap>` at the test harness level. Kills the crudest waste (identical-source cases) but doesn't address fixture authoring, oracle duplication, or Stage 2c's generation path. Rejected as insufficient even if useful short-term.
- **Tests as annotated Rust functions with shared fixtures.** Hand-authored fixtures shared across Rust test files via `mod common`. Current practice, partially. Doesn't participate in the dependency graph; fixture dedup is text-level, not structural.
- **Tests as external fixture files (JSON/YAML).** Fixtures as data, but not as *declarations*. Loses the structural-sharing property — two fixtures with identical compiled shape are distinct files.

---

## Acceptance (for when this graduates from draft)

- [ ] The six open questions above are locked with explicit answers.
- [ ] One existing test file (e.g., `m2_feature_parity_test.rs`'s 3a.2 tests) is re-expressed as `TestCase` declarations to prove the model works.
- [ ] A baseline audit of current test-suite waste (how much bootstrap, compile, lens work is duplicated) establishes the ratchet.
- [ ] Lane 2 Stage 2c's plan updates to generate `TestCase` declarations, not Rust test functions.
- [ ] Rust test-runner harness exists and can walk declarations. No cache implementation required at this milestone — just the shape.

---

## Where this lives

Cross-cutting; blocks Lane 2 Stage 2c. The design doc is DB-15. Implementation sequence (at full size) is probably:

1. Framework shape + runner harness (S)
2. Source-level compile cache (S)
3. Lens result cache (M)
4. First test migration — one file as proof (M)
5. Stage 2c generates TestCase declarations, not Rust fns (L — but this is Stage 2c's scope, not DB-15's)

Total for DB-15-owned items: M+ combined. Stage 2c then consumes.

---

## Associations

- **Lane 2 Stage 2c** ([lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md)) — forcing function and primary consumer
- **ROADMAP §Active deferrals** — DB-15 entry added tonight
- **PR #492** (cached bootstrapped DAG construction) — first cache level, lower than this design addresses
- **`src/v3/compiler/tests/m2_*_migration_test.rs`** — current pattern showing the duplication this design dissolves
- **THESIS §testgen / invariant lenses** — philosophical grounding
