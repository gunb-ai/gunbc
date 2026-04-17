> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 2 Stage 2c (test obligation materialization) | Consumes: compiler-as-dependency-analyzer thesis, `src/v3/std/verification.dag` (existing authority), `dsl/std/resources.dag` (dependency specialization)

# Design DB-15 R2 — Tests as declarations in the dependency DAG

**Design blocker:** DB-15 (test infrastructure that consumes the compiler's dependency-analysis machinery)
**Consumer:** Lane 2 Stage 2c (test obligation materialization) — forcing function
**Status:** Revision 2 (discussion draft). R1 was rejected — see Correction history below.
**Existing v3 authority being extended:** [`src/v3/std/verification.dag`](../src/v3/std/verification.dag) (`TestClaim`, `TestPredicate`, `TestSuite`)

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

### Extend `TestClaim` rather than forking

Keep the existing `TestClaim { name, source, file_name, predicate }` shape. R2 adds two things:

1. **New `TestPredicate` variants for behavioral / mock-backed claims** — the case where a property holds by observation, not by lens re-reading. Matches Lane 2 Stage 2c's mandate.
2. **A `requires: List<ResourceReference>` field (or equivalent)** — lets the claim declare what must be acquired to run it. This is NOT a new sharing mechanism; it's a declaration that the compiler's existing dependency walk reads to place acquires.

Shape (preliminary — open question #1 below on exact syntax):

```dag
// src/v3/std/verification.dag — extensions, not replacement
type TestClaim {
  name: String
  source: String
  file_name: String
  predicate: TestPredicate
  requires: List<ResourceReference>   // NEW — declared dependencies
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
      mock_transport: ResourceReference
      invariant: DeclarationRef
    }
```

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

**Blocker for R2's resource references:** v3 does not yet consume `dsl/std/resources.dag`. Grep confirms zero references to `Resource`/`acquire`/`release` under `src/v3/`.

This is a standalone dissolution-of-dual-representation item and belongs in ROADMAP §"Scheduled deletions" as its own row. Options:

1. **Port the declaration into `src/v3/std/resources.dag`** (direct v3 port; dsl/std/resources.dag remains v2 reference).
2. **Make `dsl/std/resources.dag` consumable by v3 bootstrap** (single authority across v2 and v3).

Preferring option (2) for the single-authority reason. Either way, the work is **separable from DB-15**. DB-15's `requires: List<ResourceReference>` field is authored but unconsumed until resources.dag lands in v3 — and the `TestClaim` scaffold for `requires` should carry a 🟡 dissolution marker with a named trigger (the resources-in-v3 port PR).

## Runtime cost invariant — derived, not asserted

R1 claimed `O(distinct subjects × distinct lenses)` as a load-bearing invariant. **R2 derives it from resource placement.** Because each test's dependencies are structurally declared and the compiler's walk places acquires at the outermost shared scope, the cost of a test suite is the cost of running each distinct resource acquire once across all consumers — exactly `O(distinct resources)`. The "subjects × lenses" phrasing was my own re-derivation; resources.dag's keying gives it for free.

No independent claim to enforce. If resource placement is wrong, the cost is wrong; if resource placement is right, the cost is right. Same fact, one source.

---

## Rejected alternatives

- **Fresh `TestCase`/`Claim`/`Expectation` schema (R1).** Forked `src/v3/std/verification.dag`. Single-authority violation. Rejected per codex round-1 review.

- **"Rerun the lens" test execution model (R1).** Tautological. A test that checks "what the lens says about X" by running the lens again doesn't verify anything. Rejected per codex round-1 review and INVARIANTS §no-tautological-tests.

- **Invent a new sharing mechanism (caches, fixture registries, runner coordinators).** The compiler's dependency walk + resources.dag's acquire placement IS the sharing mechanism. Reinventing it creates parallel schemas (chatgpt round-1 review). Rejected.

- **Declare DB-15's cost invariant as a standalone property.** Derived from resource placement; asserting it separately creates a fact that can drift from the underlying physics. Rejected per dependency-analyzer framing ("if it's a derived fact, don't name it as a primitive").

- **Host tests in Rust test functions with side-channel fixture sharing (status quo).** Rust tests don't participate in the compiler's dependency graph; fixture sharing is text-level (copy-pasted source strings) rather than structural. Doesn't scale to Lane 2 Stage 2c's generation-multiplied surface. Rejected.

---

## Open questions (for this draft)

1. **Exact syntax for `requires: List<ResourceReference>` on `TestClaim`.** Structural: should ResourceReference be a typed declaration reference (`DeclarationRef`) or a typed resource type (`ResourceHandle` in resources.dag terminology)? Probably the former — handles are runtime artifacts, not compile-time declarations. Verify at implementation time.

2. **Which existing `TestPredicate` variants need the `requires` declaration, and which are self-contained?** `PortStateExpectation` and `CostBounded` are compile-time assertions with no runtime resource needs. `BehavioralObservation` needs a test-runner resource. `MockBackedInvariant` needs both a test-runner AND a mock-transport resource. Open: is `requires` per-claim or per-predicate-variant?

3. **Tautology-avoidance enforcement.** The rule "predicate cannot rerun the producing lens" is currently prose. Can it be enforced structurally — e.g., the predicate variants are explicitly behavioral/observational by type, and "rerun lens X" is not even expressible? Needs a pass to confirm no variant sneaks in that permits the pattern.

4. **Lane 2 Stage 2c generation surface.** Stage 2c generates `TestClaim` declarations from lens outputs. What's the structural shape of "this lens's output, materialized into a `TestPredicate`"? Likely one generation rule per `(lens, predicate-variant)` pair, declared once per lens. Out of scope for DB-15's design; in scope for Stage 2c's implementation.

---

## Acceptance (for when this graduates from draft)

- [ ] Open questions 1–3 locked with explicit answers.
- [ ] Extensions to `src/v3/std/verification.dag` sketched with exact field shapes (`requires`, new `TestPredicate` variants).
- [ ] Resources-in-v3 reconciliation scheduled — named PR or ROADMAP row identifying the upstream path.
- [ ] One existing test file (e.g., `m2_feature_parity_test.rs`'s 3a.2 tests) re-expressed as `TestClaim` declarations, showing the structural form consuming the compiler's dependency walk.
- [ ] Lane 2 Stage 2c plan updates: generation emits `TestClaim` declarations via the R2 shape, not Rust functions.
- [ ] Cost invariant phrased as derived from resource placement, not as a standalone claim.

---

## Associations

- **Compiler-as-dependency-analyzer thesis** (tonight's framing) — DB-15 is the testing-scope consequence. Tests are declarations; the dependency walk handles them like anything else.
- **`src/v3/std/verification.dag`** — the existing authority DB-15 extends. `TestClaim`, `TestPredicate`, `TestSuite` stay as-authored.
- **`dsl/std/resources.dag`** — the existing acquire/release model DB-15 references via `requires: List<ResourceReference>`. Prerequisite for consumption: reconcile into v3.
- **Lane 2 Stage 2c** ([lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md)) — forcing function; generates `TestClaim` declarations from lens outputs.
- **`src/v3/compiler/pipeline.dag`** — analogous pattern for non-test declarations (compiler stages consume the dependency walk); DB-15 applies the same shape to test-scope declarations.
- **E-9 (INVARIANTS.md)** — sibling invariant. DB-15 doesn't need a new invariant; the rule "tests are declarations that consume the dependency walk" is implied by the thesis. If a future PR wants to bank it load-bearingly, it would be something like E-10 "tests as first-class declarations."
- **ROADMAP §Active deferrals** — DB-15's implementation (extending verification.dag + resources-in-v3) is a deferral under Lane 2 Stage 2c. ROADMAP row stays until the extension PR lands.

---

## Why R2 is smaller than R1

R1 added mechanism: TestCase type, Claim type, Expectation type, caches, runner coordination, six open questions.

R2 subtracts. `TestClaim` exists; `Resource` exists; the dependency walk exists. R2 names how they compose, adds two new `TestPredicate` variants for behavioral/mock claims, declares a `requires` edge, and points at one standalone prerequisite (resources-in-v3). The "six open questions" collapse to "does the predicate coproduct cover Lane 2 Stage 2c's needs" — one structural question.

This is the compiler-as-dependency-analyzer thesis cashing out at the testing surface: **don't reinvent mechanisms; name how things depend on each other; let the compiler's walk do the rest.**
