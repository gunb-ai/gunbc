> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 2 Stage 2c (test obligation materialization) | Consumes: compiler-as-dependency-analyzer thesis, `src/v3/std/verification.dag` (existing authority), `dsl/std/resources.dag` (dependency specialization)

# Design DB-15 R2 — Tests as declarations in the dependency DAG

**Design blocker:** DB-15 (test infrastructure that consumes the compiler's dependency-analysis machinery)
**Consumer:** Lane 2 Stage 2c (test obligation materialization) — forcing function
**Status:** R2 **locked for schema** — `src/v3/std/verification.dag` implements `TestClaim.requires`, `BehavioralObservation`, `MockBackedInvariant`, `TestObligation`, and `materialize_test_obligations`; `src/v3/std/resources.dag` supplies `ResourceReference` / `ResourceHandle` (including `cap: Secret` aligned with `dsl/std/resources.dag`). Test-runner wiring and doc-only checkboxes below remain follow-ups. R1 was rejected — see Correction history below.
**Existing v3 authority being extended:** [`src/v3/std/verification.dag`](../src/v3/std/verification.dag) — quoted inline in §"What DB-15 extends" below.

**Two verification.dag files exist in the repo** — this is important:
- `dsl/std/verification.dag` (v2-era): `AssertKind`, `TestClaim { kind, label }`, `TestCase { name, claims, ignored }`. Older behavioral-assertion model.
- `src/v3/std/verification.dag` (v3, extended by DB-15): `TestPredicate`, `TestClaim { name, source, file_name, predicate }`, `TestSuite`.

Per the v3 file's own header comment: *"`dsl/std/verification.dag` remains the older v2-era behavioral-assertion model; it is not silently superseded here. Convergence trigger: once v2 retires and the shared std tree can host the v3 verification surface directly, dissolve the duplicate definitions back to one `std.verification`."* DB-15 lives in the v3 surface; convergence with v2 is a separate deferral.

---

## Correction history

**Revision 1** proposed a fresh `TestCase`/`Claim`/`Expectation` schema. Reviewers (codex + chatgpt) converged on two structural concerns:

1. **Single-authority violation.** `src/v3/std/verification.dag` already declares `TestClaim { name, source, file_name, predicate }` and `TestSuite { name, claims }`, explicitly named as "the structural authority for generated tests." R1's fresh schema forked this authority.
2. **Tautology.** R1 defined test execution as "rerun the lens that produced the fact" (e.g., a claim asserts commutativity by rerunning `parallelism_lens`). That violates INVARIANTS § no-tautological-tests: if the test just re-reads what the lens says, it doesn't verify anything.
3. **Missing substrate reference.** Both reviewers pointed at `dsl/std/resources.dag` as the existing model for "shared acquired thing" (compiler-inserted acquire/release, keyed for conflict detection). R1 reinvented the sharing mechanism by gesturing at structural equality.

**Revision 2 (this doc)** consumes the compiler-as-dependency-analyzer thesis. Tests are declarations like any other; fixture sharing, caching, incremental execution, and oracle selection all fall out of the compiler's existing dependency walk. `TestClaim` stays the authority (DB-15 extends it, doesn't replace it). Tautology avoidance is a structural rule, not a testing convention. Resources are a named specialization of "depends on," not a parallel mechanism.

---

## The framing this consumes

**The compiler IS a dependency analyzer.** Every declaration names its dependencies via typed edges to other declarations. The compiler walks that DAG — compile-time verification, runtime composition, emission into targets — all consume the same walk.

**Resources are one specialization.** `dsl/std/resources.dag` models acquirable capabilities with compiler-inserted acquire/release, keyed for conflict detection. That's *one flavor* of dependency relation: the one with lifecycle. The general relation is just "declaration → declaration via typed edge"; resources have the additional acquire/release discipline layered on.

**Tests are declarations.** `TestClaim` already exists in `src/v3/std/verification.dag`. A test declaration names its dependencies structurally:

- *What is being tested* — a `DeclarationRef` to the subject (a fn, a type, a module, whatever).
- *What property must hold* — the `TestPredicate`, which is itself a structural description (e.g., `PortStateExpectation`, `CostBounded`, plus new variants for behavioral claims).
- *What must be acquired to run the test* — resources in the `dsl/std/resources.dag` sense: bootstrap DAG, compilation output, mock backends, test runner state.

Sharing, caching, incremental execution, and oracle selection all fall out of the compiler's dependency walk:

- Two tests with the same subject share the compile output because they reference the same `DeclarationRef`.
- Two tests with the same lens-computed fact share the lens evaluation because the dependency walk sees the same `(DAG, lens)` pair.
- Two tests that need the same resource share its acquire, at the outermost scope across them, because `resources.dag`'s acquire placement IS the compiler's dependency walk.

There is no new mechanism to invent. DB-15 names what test-scope declarations ARE, in terms of existing authorities.

---

## Minimal model (what DB-15 adds)

### What DB-15 extends — current `src/v3/std/verification.dag` shapes quoted inline

Per INVARIANTS.md (claimed existing substrate forms must be verified against current code), the shapes being extended, quoted from `src/v3/std/verification.dag` on `main` as of this revision:

```dag
// src/v3/std/verification.dag:69-81
type TestPredicate
  = DiagnosticExpected { ... }
  | PortState {
      bind_name: String
      state: PortStateExpectation
    }
  | CostBounded {
      bind_name: String
      comparator: ComparisonOp
      bound: Int
    }

// src/v3/std/verification.dag:83-88
type TestClaim {
  name: String
  source: String
  file_name: String
  predicate: TestPredicate
}

// src/v3/std/verification.dag:90-93
type TestSuite {
  name: String
  claims: List<TestClaim>
}
```

**This is the authority DB-15 R2 extends.** The older `dsl/std/verification.dag` (v2-era with `AssertKind`, `TestClaim { kind, label }`, `TestCase`) is a separate model noted above; DB-15 doesn't touch it. If the convergence noted in the v3 file's own header (collapse both back to one `std.verification`) lands first, DB-15 re-aligns against whatever the merged shape is; that's a mechanical rebase, not a redesign.

### Extend `TestClaim` rather than forking

R2 keeps the above shapes and adds two things:

1. **New `TestPredicate` variants for behavioral / mock-backed claims** — the case where a property holds by observation, not by lens re-reading. Matches Lane 2 Stage 2c's mandate.
2. **A `requires: List<ResourceReference>` field (or equivalent)** — lets the claim declare what must be acquired to run it. This is NOT a new sharing mechanism; it's a declaration that the compiler's existing dependency walk reads to place acquires.

Shape (**implemented** in `src/v3/std/verification.dag`):

```dag
// src/v3/std/verification.dag — extensions, not replacement
type TestClaim {
  name: String
  source: String
  file_name: String
  predicate: TestPredicate
  requires: List<ResourceReference>   // declared dependencies (per-claim)
}

type TestPredicate
  = DiagnosticExpected { ... }        // existing
  | PortState { ... }                 // existing
  | CostBounded { ... }               // existing
  | BehavioralObservation {           // NEW — tautology-avoiding
      subject: DeclarationRef
      input_sample: SampleRef
      expected_output: ValueRef
    }
  | MockBackedInvariant {             // NEW — for Lane 2 Stage 2c
      subject: DeclarationRef
      invariant: DeclarationRef
    }
```

For mock-backed tests, declare mock `ResourceReference` targets only on `TestClaim.requires` (obligation authority) — not again inside `MockBackedInvariant`.

`BehavioralObservation` encodes "the test runs the subject on a sample and compares to an independently-declared expected output." That's not rerunning a lens; it's running the subject and checking a separately-declared fact.

`MockBackedInvariant` encodes "the test runs the subject against a mocked resource (e.g., a mock HTTP backend) and checks that a separately-declared invariant holds." That's the runtime-mock Lane 2 Stage 2c requires.

### Resources are consumed, not reinvented

`ResourceReference` points at `dsl/std/resources.dag` (once that file is reconciled into v3 — see Prerequisite below). Tests declare the resources they need; the compiler's existing acquire/release machinery places acquires at the outermost shared scope across tests with matching resource keys.

No new "fixture cache," no new "lens result cache," no new "runner coordinator." The compiler's dependency walk is the cache, the coordinator, and the scope manager.

### Tautology avoidance is a structural rule, not a convention

**Rule:** a `TestPredicate` cannot verify a lens's fact by rerunning the same lens. Verification must go through an independent path.

Enforceable structurally: the predicate variants above (`BehavioralObservation`, `MockBackedInvariant`) verify by OBSERVATION of the subject, not by RE-EVALUATION of the producing lens. Predicate variants that would be tautological (e.g., "what does lens X say about subject Y?") are not in the coproduct.

**Lens outputs are one source of oracles, but not the verification mechanism.** A lens can supply the EXPECTED fact (e.g., the expected cost bound the lens computed statically). The test then verifies the fact behaviorally (runs the subject and compares) or via mock (runs with a mocked dependency and checks the invariant). The lens isn't rerun at test time; its output is PRE-COMPUTED and embedded in the predicate's declaration.

This is the difference between `CostBounded { bind_name, comparator, bound }` (existing, structural: "the compile-time cost lens's output must satisfy this comparator") and a hypothetical tautological `LensSays { lens, subject, expected }` (which would be "rerun the lens and check it says what we think" — redundant with the first).

---

## Prerequisite: `dsl/std/resources.dag` → v3 reconciliation

**Update:** `src/v3/std/resources.dag` (module `v3.std.resources`) provides `ResourceHandle` (including `cap: Secret` per `dsl/std/resources.dag`) and `ResourceReference { target: DeclarationRef }` for v3 bootstrap so `TestClaim.requires` has typed carriers. Full `resource { }` syntax, acquire/release insertion, and loading `dsl/std/resources.dag` in the same bootstrap pass as v3-only files remain ROADMAP-tracked dissolution work — see [ROADMAP.md](../ROADMAP.md) Stage 2c / resources.

## Runtime cost — three sharing classes, all derived from dependency placement

R1 claimed `O(distinct subjects × distinct lenses)` as a load-bearing invariant. R2's first draft oversimplified to `O(distinct resources)`. The reviewer correctly flagged this as silently dropping the other sharing classes.

**The corrected accounting.** A test suite has three sharing classes, all handled by the compiler's dependency walk but with different shapes:

1. **Subject compilation output.** Compiling a source (the `source: String` on `TestClaim`) to a DAG is a pure function of the source. Two tests referencing the same subject share the compile output — keyed on `(source, file_name)`.
2. **Lens evaluation output.** A lens result over a compiled subject is a pure function of `(DAG, lens)`. Two tests invoking the same lens over the same subject share the lens evaluation — keyed on `(DAG_hash, lens_decl_id)`.
3. **Resource acquire/release.** Runtime-backed test observation needs resources (test runner, mock transports, etc.). Acquires are placed at the outermost shared scope across consumers — keyed on the resource's own key fields per `dsl/std/resources.dag`.

Each class is a CACHING fact, derived from the compiler's dependency walk (the first two) or resource placement (the third). None of them is an independent invariant DB-15 asserts; they're all consequences of the walk and the existing resources model.

**Aggregate cost** for a suite with N test claims:
- If claims share subjects: `distinct_subjects` compile invocations, not `N`.
- If claims share lens applications: `distinct (DAG, lens)` lens invocations, not `N × lenses`.
- If claims share runtime resources: `distinct resources` acquires, not `N × resource_needs`.

The "more efficient than typical" intuition cashes out from ALL THREE collapses, not just the third. Whether Stage 2c's payoff is real depends on generated claims sharing enough subjects/lenses/resources that these three collapses materialize — a structural property of the generation rules, not something DB-15 asserts separately.

**No independent claim to enforce.** If the dependency walk is complete and resource placement is correct, all three collapses happen by construction; if not, they don't. DB-15's scope is to name the shapes that let the walk see the sharing — it doesn't add a new invariant.

---

## Rejected alternatives

- **Fresh `TestCase`/`Claim`/`Expectation` schema (R1).** Forked `src/v3/std/verification.dag`. Single-authority violation. Rejected per codex round-1 review.

- **"Rerun the lens" test execution model (R1).** Tautological. A test that checks "what the lens says about X" by running the lens again doesn't verify anything. Rejected per codex round-1 review and INVARIANTS §no-tautological-tests.

- **Invent a new sharing mechanism (caches, fixture registries, runner coordinators).** The compiler's dependency walk + resources.dag's acquire placement IS the sharing mechanism. Reinventing it creates parallel schemas (chatgpt round-1 review). Rejected.

- **Declare DB-15's cost invariant as a standalone property.** Derived from resource placement; asserting it separately creates a fact that can drift from the underlying physics. Rejected per dependency-analyzer framing ("if it's a derived fact, don't name it as a primitive").

- **Host tests in Rust test functions with side-channel fixture sharing (status quo).** Rust tests don't participate in the compiler's dependency graph; fixture sharing is text-level (copy-pasted source strings) rather than structural. Doesn't scale to Lane 2 Stage 2c's generation-multiplied surface. Rejected.

---

## Open questions — lock state (2026-04)

Questions **1–3** from R2 draft are **resolved** by the shipped `src/v3/std/verification.dag` coproduct and `ResourceReference` shape:

1. **`requires` syntax.** `TestClaim.requires: List<ResourceReference>` with `ResourceReference { target: DeclarationRef }` — compile-time declaration edges, not raw `ResourceHandle` literals in claims (handles remain the runtime minted carrier in `dsl/std/resources.dag` / `v3.std.resources`).

2. **Per-claim vs per-predicate.** `requires` is **per `TestClaim`** (one list on the claim). Predicates that need runtime backing declare resources at the claim level; compile-time-only predicates (`PortHasState`, `CostBounded`, etc.) may use empty `requires` where applicable.

3. **Tautology avoidance.** Enforced by **construction**: behavioral/mock variants (`BehavioralObservation`, `MockBackedInvariant`) point at `DeclarationRef` edges for subject / (for mocks: invariant, with mock carriers on `requires` only); there is no `TestPredicate` variant meaning “invoke lens L and compare.” Prose rule matches the expressible surface.

4. **Lane 2 Stage 2c generation surface.** Still open for **implementation** — how each lens materializes into `TestPredicate` (generation rules). Out of scope for this design doc’s schema lock; tracked under Stage 2c / testgen.

---

## Acceptance — schema locked; execution follow-ups

**R2 schema** (verification + minimal resources carriers) is **locked** — this section tracks **test-runner / generation** work, not unresolved design questions.

- [x] Open questions 1–3 locked with explicit answers (see section above).
- [x] Extensions to `src/v3/std/verification.dag` with field shapes (`requires`, `BehavioralObservation`, `MockBackedInvariant`, obligations).
- [x] Minimal resources surface in v3 (`src/v3/std/resources.dag`); full dsl merge / `resource { }` lowering still ROADMAP-tracked.
- [ ] One existing test file (e.g., `m2_feature_parity_test.rs`'s 3a.2 tests) re-expressed as `TestClaim` declarations, showing the structural form consuming the compiler's dependency walk.
- [ ] Lane 2 Stage 2c plan updates: generation emits `TestClaim` declarations via the R2 shape, not Rust functions.
- [x] Cost / sharing narrative: derived from dependency walk (§Runtime cost); no standalone O(…) claim as primitive.

---

## Associations

- **Compiler-as-dependency-analyzer thesis** (tonight's framing) — DB-15 is the testing-scope consequence. Tests are declarations; the dependency walk handles them like anything else.
- **`src/v3/std/verification.dag`** — the existing authority DB-15 extends. `TestClaim`, `TestPredicate`, `TestSuite` stay as-authored.
- **`dsl/std/resources.dag`** — acquire/release authority; **`src/v3/std/resources.dag`** supplies bootstrap `ResourceHandle` / `ResourceReference` with **matching `ResourceHandle` field labels** until full `resource { }` / merged bootstrap is ROADMAP-tracked.
- **Lane 2 Stage 2c** ([lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md)) — forcing function; generates `TestClaim` declarations from lens outputs.
- **`src/v3/compiler/pipeline.dag`** — analogous pattern for non-test declarations (compiler stages consume the dependency walk); DB-15 applies the same shape to test-scope declarations.
- **E-9 (INVARIANTS.md)** — sibling invariant. DB-15 doesn't need a new invariant; the rule "tests are declarations that consume the dependency walk" is implied by the thesis. If a future PR wants to bank it load-bearingly, it would be something like E-10 "tests as first-class declarations."
- **ROADMAP §Active deferrals** — DB-15's implementation (extending verification.dag + resources-in-v3) is a deferral under Lane 2 Stage 2c. ROADMAP row stays until the extension PR lands.

---

## Why R2 is smaller than R1

R1 added mechanism: TestCase type, Claim type, Expectation type, caches, runner coordination, six open questions.

R2 subtracts. `TestClaim` exists; `Resource` exists; the dependency walk exists. R2 names how they compose, adds two new `TestPredicate` variants for behavioral/mock claims, declares a `requires` edge, and points at one standalone prerequisite (resources-in-v3). The "six open questions" collapse to "does the predicate coproduct cover Lane 2 Stage 2c's needs" — one structural question.

This is the compiler-as-dependency-analyzer thesis cashing out at the testing surface: **don't reinvent mechanisms; name how things depend on each other; let the compiler's walk do the rest.**
