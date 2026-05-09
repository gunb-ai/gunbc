> Part of: [`docs/r3-structure.md`](r3-structure.md) lane T-Tests-As-Data-Completeness, [`docs/design-test-infra.md`](design-test-infra.md), [`TESTING.md`](../TESTING.md), [`THESIS.md`](../THESIS.md) facet 3, [`../INVARIANTS.md`](../INVARIANTS.md), [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md)
>
> **Purpose:** specify the substrate carriers and migration path that close THESIS facet 3 ("tests are data too") to full coverage. The R3 lane T-Tests-As-Data-Completeness has three deliverables: (1) every Rust test ports to a `.dag` `TestClaim` or to generated target-language test code; (2) the property-based testing surface (`ForAll`/`Exists` quantifiers + `ProgramGenerator` carrier); (3) cementing test discipline for `.dag` lenses (per-lens v2-oracle equivalence on the same source). This doc resolves the structural design questions blocking lane dispatch.
>
> **Authority discipline:** R3 design doc. Implementation lane is **T-Tests-As-Data-Completeness** (R3 lane 15; Verification Manager). Substrate Manager assists on carrier authoring per §7. All §8 design questions resolved in-doc per `feedback_design_before_implement` — no Director ratification required before lane dispatch (only standard cascade gates: R2-Evaluator landed; existing TestClaim infrastructure from DB-15 R2).

## What this document is

[`docs/design-test-infra.md`](design-test-infra.md) (DB-15 R2) locked the schema for `TestClaim { name, source, file_name, predicate, requires }` plus `BehavioralObservation` / `MockBackedInvariant` predicate variants. That schema covers **enumerated** test cases — one fixture per `TestClaim`. THESIS facet 3 retracted the "two-residual" Rust-test carve-out (under 0-floor: helper unit tests vanish with their hand-Rust subjects; external-toolchain tests migrate to `ExecuteCommand`-based `TestClaim`s). The R3 lane closes the residual gap:

- **Facet 3 full coverage** — every existing Rust test under `src/v3/compiler/tests/` either ports to a `.dag` `TestClaim` (boundary tests via `ExecuteCommand`; lens / behavioral assertions via predicate variants) **or** is replaced by generated target-language test code that the compiler emits from `TestClaim` declarations into Rust/Python/Go test runners.
- **Property-based surface** — today every claim names one `source: String`. Some properties are not "this single program has this property" but "every program in family `F` satisfies property `P`" (∀) or "some program in family `F` satisfies property `P`" (∃). The substrate has no carrier for that quantification today.
- **Lens cementing discipline** — `TESTING.md` §"Cementing tests (Band C — lens subsumption)" + the dispatch test at `src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs` already enumerate cementing as a discipline. The lane closes the "every `LensRegistryEntry` whose row reads `BEHAVIORALLY COMPLETE` with a real v2 counterpart has a cementing module" ratchet to **zero gaps** at R3 close, alongside T-Lens-Behavioral-Parity flipping the four PROXY/STUB/PARTIAL rows to COMPLETE.

This document specifies the substrate shape for the property-based surface, the migration path for Rust→`TestClaim`, the generated-test-code path, and the cementing discipline closure.

## §1. Problem framing

### §1.1 What "tests as data" means at full coverage

THESIS facet 3 is one sentence: *"The test suite equivalent of v2's hand-authored `pipeline.rs` exists only as `.dag` `TestClaim` declarations and generated target-language test code."* Full coverage decomposes into three structural commitments:

1. **No hand-authored Rust test files in `src/v3/compiler/tests/` post-R3.** Today the SG-0 census `EXPECTED_HAND_AUTHORED_TEST` ratchet (`src/v3/compiler/tests/integration/sg0_census_test.rs:243`) lists ~87 hand-authored Rust test files. Under 0-floor those shrink to 0; under T-Tests-As-Data-Completeness, the lane verifies **0 remain at R3 close** (the SG-0 census ratchet reaches 0 for the test partition).
2. **Every property a Rust test asserts is expressible as a `TestPredicate`.** Today's `TestPredicate` coproduct (per `src/v3/std/verification.dag:109-278`) covers 22 variants. **Maturity is mixed**: a subset is 🟢 TERMINAL (`Compiles`, `FailsWithDiagnostic`, `OutputEquals`, `PortHasState`, `DeclarationHasRefinement`, `CostBounded`, `BehavioralObservation`, `MockBackedInvariant`, `BridgeLedgerZero`); the remaining variants are 🟡 Scaffold per the verification.dag inline annotations (`ExecuteCommand`, `ForAllTargets`, `LensOutputEquals`, `DifferentialEquals`, `BinaryDimensionReportEquals`, `AlgebraicLaw`, `ReleaseDeferredClaim`, `SubstrateResearchDeferredClaim`) — each carries a named dissolution trigger in its variant comment (collapse with paired variants once substrate facets land). Census/ratchet/fixedpoint variants (`CensusBoundCheck`, `CensusSubsetCount`, `FixedPointConverges`, `RatchetZero`, `GeneratedFromDag`) are TERMINAL within their R1C-A scope. Rust-test migration may target either TERMINAL or 🟡 Scaffold variants; ports landing on a 🟡 Scaffold are inherently scoped by that variant's named dissolution trigger (when the trigger fires, the test ports forward to the dissolved replacement). The migration audit in §3 catalogs each Rust-test class against these variants and identifies the (small) residual that needs new carriers — those new carriers are scope of T-Tests-As-Data-Completeness, not assumed live.
3. **Quantifier surface for property-based claims.** A claim of the form *"every program shaped like X satisfies P"* requires a carrier for *"every program shaped like X"*. That is the `ProgramGenerator` substrate introduction.

### §1.2 Why the existing `TestClaim` substrate is insufficient

`TestClaim.source: String` enumerates one fixture per claim. Property-based testing requires the runner to walk a *family* of programs (a generator's output) and apply the predicate to each. The substrate must let a single declaration name "the family of programs and the predicate that must hold for every member" without textually enumerating each member — otherwise property-based claims degenerate to enumeration and lose their mathematical content.

The minimal substrate shift: introduce a sibling carrier `QuantifiedTestClaim` (or reshape `TestClaim` to admit a quantified mode) that names a `ProgramGenerator` instead of a `source: String`, plus a `ForAll` / `Exists` quantifier discriminating "every member" from "some member."

### §1.3 What "generated target-language test code" means

The thesis-facet-3 sentence pairs `.dag TestClaim` declarations with **generated target-language test code**. These are the same data with two emission paths:

- **Path A — runner-evaluated.** The `.dag` test runner (per `src/v3/compiler/src/test_runner.rs`) reads `TestClaim` declarations, evaluates the predicate against the compiled `source`, and reports Pass/Fail. This is how today's `.dag` tests already run.
- **Path B — emitted target test runner.** For cross-target equivalence (and for users who want to ship gunbc-validated tests as part of a Rust/Python/Go test suite), the same `TestClaim` declaration emits to a target-language test function (e.g., a `#[test] fn ...` in Rust, a `def test_...` in Python, a `func Test... (t *testing.T)` in Go). The emitted test asserts the same predicate against the same `source` (compiled to that target), giving a target-native receipt.

Path B is the existing emission infrastructure (`emit_rust_module` / `emit_python_module` / `emit_go_module`) extended to consume `TestClaim` declarations. There is no new compiler stage; emission already walks `Declaration`s and `TestClaim` is one. The lane delivers the per-target rendering tables for `TestPredicate` variants.

## §2. Substrate carrier shape

The new substrate lives in `src/v3/std/verification.dag` (extending the existing authority — single-authority discipline per P2) and is keyed off a `Quantifier` discriminator + a `ProgramGenerator` carrier.

### §2.1 `ProgramGenerator` carrier

`ProgramGenerator` is a structural reference to a generator declaration. It is **not** a roster of "shape kinds" — that would be the same failure class flagged by [`docs/lens-library-design.md`](lens-library-design.md) §1.5 (closed roster on a category meant to be lens-extensible). Instead, a generator is itself a `.dag` declaration whose body produces a `List<ProgramShape>` (or, equivalently, an iterator-shaped value), and `ProgramGenerator` references it structurally:

```dag
// src/v3/std/verification.dag — additions

import v3.std.lookup { Lookup }

// A reference to a declaration whose body produces program shapes
// (one shape per generated program, or, equivalently, an iterable
// `List<ProgramShape>`). The named declaration is itself the
// authority for the family — a fresh family is a new declaration,
// not a new variant in a roster.
type ProgramGenerator {
  generator: DeclarationRef
}

// A program shape is a `.dag` source program with optional bound holes.
// Today the bootstrap can ship a degenerate carrier whose only
// inhabitant is `LiteralProgram { source, file_name }` matching the
// existing `TestClaim` source/file_name pair. Lens-extensible richer
// shapes (parameterized programs, programs derived from a typed
// substrate walk) are user-extensible per `feedback_groundedness_gates_lenses`
// — the lens framework's structural inhabitance is the namespacing.
type ProgramShape
  = LiteralProgram { source: String, file_name: String }
  // Future-extensible via DAG-ancestor inheritance (per INVARIANTS §P1
  // Step 1 of the substrate-fact-introduction procedure): a new shape
  // attaches as a sibling here when the closed system genuinely needs
  // a new primitive form. Initial bootstrap: LiteralProgram only.
```

**Why a single-variant coproduct, not a record**: per `INVARIANTS.md#p1-modeling-faithfulness` Step 2 (coproduct-vs-coordinate check), shapes are alternatives — a generated program is *one kind* at a time (a `LiteralProgram` OR, eventually, a `ParameterizedProgram`, OR a `SubstrateDerivedProgram`). The variants are not coordinates. Single-variant-at-bootstrap is structurally honest (it admits future variants without restructuring) and matches `feedback_state_space_vs_behavioral_invariants` (a coproduct is the right shape because illegal combinations like "both LiteralProgram fields and ParameterizedProgram fields populated simultaneously" become unrepresentable).

### §2.2 `Quantifier` and `QuantifiedTestClaim`

`Quantifier` is a closed two-variant sum — `ForAll` and `Exists` are the only structurally meaningful quantifications over a `ProgramGenerator`'s output:

```dag
// Quantifier over generated program family. Two variants exhaust the
// cases — universal ("predicate holds for every member") vs existential
// ("predicate holds for at least one member"). Per INVARIANTS §P1 Step 2,
// this is a true coproduct (a single quantification is one or the other).
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
```

`QuantifiedTestClaim` lives **alongside** `TestClaim`, not as a replacement. The two carriers cover orthogonal axes:

- `TestClaim`: one named program (`source` / `file_name`); predicate evaluated once. Today's surface; not deprecated.
- `QuantifiedTestClaim`: a generator-produced family; predicate evaluated over `ForAll`/`Exists` of the family.

**Why two carriers, not a unified one with an `Option<Generator>`**: per `feedback_optional_models_recovery_as_exception`, `Option<Generator>` would treat "no generator" as the exceptional case — but enumerated single-program tests are the common case, not the exception. Two carriers preserve the structural distinction (one program vs a family) and make the difference visible in every reading site. Per `feedback_state_space_vs_behavioral_invariants`, this also avoids the illegal state "both `source` populated AND `generator` populated" being expressible.

### §2.3 `TestSuite` extension

`TestSuite.claims: List<TestClaim>` becomes a sum-typed list admitting both shapes:

```dag
type SuiteClaim
  = Enumerated(TestClaim)
  | Quantified(QuantifiedTestClaim)

type TestSuite {
  name: String
  claims: List<SuiteClaim>
}
```

Existing suite declarations migrate by wrapping each claim in `Enumerated(...)`. The migration is mechanical (one constructor wrap per claim site); the runner consumes both shapes uniformly via match.

**Alternative considered, rejected**: a single `TestSuite { enumerated_claims, quantified_claims }` record. Rejected per `INVARIANTS.md#p1-modeling-faithfulness` Step 2 — the order claims appear in the suite is a load-bearing facet (test reporting order, dependency traversal order); a record loses ordering across the two lists. The sum-typed list preserves the ordered sequence.

### §2.4 Generator authoring shape

A `ProgramGenerator` references a declaration whose body produces program shapes. The minimal body shape:

```dag
// User-authored generator (example):
data parse_smoke_generator: List<ProgramShape> = [
  LiteralProgram { source: "let x: Int = 1", file_name: "gen_001.v3" },
  LiteralProgram { source: "let y: Bool = true", file_name: "gen_002.v3" },
  LiteralProgram { source: "fn f(): Int = 1", file_name: "gen_003.v3" }
]

// QuantifiedTestClaim consuming it:
data parse_smoke_universal: QuantifiedTestClaim = {
  name: "every parse-smoke fixture compiles",
  generator: ProgramGenerator { generator: parse_smoke_generator },
  quantifier: ForAll,
  predicate: Compiles,
  requires: []
}
```

**Bootstrap ergonomics**: at R3 lane open, generators are explicit `List<ProgramShape>` declarations (no recursion, no induction over a substrate walk). This keeps the bootstrap predicate-runner change small (walk the list, apply predicate per element). Richer generators — programs derived from typed substrate walks (e.g., "every Behavior variant produces a smoke fixture") — attach via the same `ProgramGenerator` carrier without further substrate change: the body returns `List<ProgramShape>` regardless of how the body computes it. Per `INVARIANTS.md#p4-decidability` (decidability) the body must be bounded; that is already the substrate-wide invariant.

### §2.5 Why `ForAll` / `Exists` here, not as `TestPredicate` variants

A tempting alternative shape: `TestPredicate::ForAll { generator, predicate }` and `TestPredicate::Exists { generator, predicate }` — fold the quantifier into the predicate coproduct.

**Rejected** per `INVARIANTS.md#p2-boundary-discipline` (boundary discipline) and `feedback_compositional_not_templating`. Reasons:

1. **Predicate vs claim are different scopes.** A `TestPredicate` is "the property checked once given a single program." A quantifier is "how the family relates to the property." Conflating them confuses the reader: every other `TestPredicate` variant takes "the program" as implicit context (the surrounding `TestClaim.source`); `ForAll`/`Exists` would take "a generator" instead, breaking that invariant.
2. **Dispatch is structurally different.** The runner's `TestPredicate` evaluator (`test_runner::evaluate_predicate`) is a per-program function. A `ForAll` predicate is a per-*generator* function. The dispatch shape differs; encoding both through one coproduct buries the distinction in a runtime branch instead of a type-level partition.
3. **Recursion shape.** A `ForAll/Exists`-wrapped predicate could nest (`ForAll { predicate: Exists { ... } }`), implying second-order quantification. That is meaningful in mathematical logic but not in the substrate's bounded-forward-execution premise (per `INVARIANTS.md#p4-decidability`). Putting the quantifier on the *claim* makes nesting structurally unexpressible, which is the right answer at this stage of the language.

The structural separation (quantifier on `QuantifiedTestClaim`, not in `TestPredicate`) preserves the predicate's "given one program, check property" semantics and isolates the quantification at the claim level where it belongs.

## §3. Migration path: Rust tests → `.dag` TestClaim

Per `project_test_modeling` (from MEMORY): *"tests as verification claims in std/; structural tests from data; string templates are bootstrap seed."* The migration is a structural classification of the existing Rust suite by which `TestPredicate` variant covers each test.

### §3.1 Rust-test classification (seven classes)

The current Rust test population (`src/v3/compiler/tests/` — 87 files, ~357 integration tests + crate-internal `#[cfg(test)] mod tests`) decomposes into seven structural classes (C1–C7 per the enumeration in §10 step 3). Each class names the `TestPredicate` it ports to:

| Class | Description | Target `TestPredicate` | Approx. share | Migration shape |
|---|---|---|---|---|
| **C1: Compile-or-reject** | "this source compiles" / "this source fails with diagnostic D" | `Compiles`, `FailsWithDiagnostic` | ~30% | Direct port. Source string moves to `TestClaim.source`; expected diagnostic moves to `DiagnosticReference`. |
| **C2: Lens output equality** | "lens L applied to source S equals expected E" | `LensOutputEquals` | ~20% | Direct port. Expected `E` is itself a `.dag` declaration (per `feedback_naming_is_aliasing` — declarations not strings). |
| **C3: Behavioral observation** | "running source S on input I produces output O" | `BehavioralObservation` | ~10% | Port via DB-15 carrier. Input + expected output are `.dag` declarations. |
| **C4: Boundary (host process)** | "running emitted artifact via target-language toolchain exits with code N" | `ExecuteCommand`, `ForAllTargets` | ~15% | Port via PB-Runtime `ExecuteCommand` (already landed; per `TESTING.md` capability state). Per-target dispatch via `ForAllTargets`. |
| **C5: Cementing (v2 oracle)** | "v2 oracle and v3 lens produce equal output on source S" | `DifferentialEquals`, `LensOutputEquals` | ~10% | Port via DB-15 carrier. Each cementing module becomes a `.dag` fixture file referenced from the dispatch ratchet. |
| **C6: Census / ratchet** | "census list X has cardinality ≤ N" / "ratchet R reads zero" | `CensusBoundCheck`, `CensusSubsetCount`, `RatchetZero`, `GeneratedFromDag` | ~10% | Port via R1C-A predicates (already landed). |
| **C7: Property-based (NEW under §2)** | "every program in family F has property P" | `QuantifiedTestClaim` with `ForAll` | ~5% (latent — most current tests are enumerated; quantified emerges as fixtures generalize) | Author generator + `QuantifiedTestClaim`. |

The classes are exhaustive over the existing test population. Verification of exhaustiveness: a one-PR audit walks every Rust test file and labels it with one of C1–C7 in a `tests_as_data_migration_audit.dag` declaration (lives alongside `sg0_census_test.rs`'s ratchet); the ratchet asserts `Σ(class_counts) == file_count`. Ports proceed class-by-class; per-PR migration of one class shrinks the SG-0 `EXPECTED_HAND_AUTHORED_TEST` partition.

### §3.2 Mechanical port: an example

A current Rust test (illustrative, simplified):

```rust
// src/v3/compiler/tests/integration/some_lens_test.rs
#[test]
fn cost_of_empty_fold_returns_zero() {
    let dag = compile_to_dag("let x: Int = 0", "fold.v3").expect("compiles");
    let bind = dag.declaration_by_name("x").unwrap();
    let cost = cost_of(&dag, &bind.value_port());
    assert_eq!(cost, Hit(0));
}
```

After migration:

```dag
// src/v3/compiler/tests/dag/cost_lens_smoke.dag
module v3.compiler.tests.cost_lens_smoke

import std.verification { TestClaim, TestSuite, LensOutputEquals }
import v3.std.lookup { Hit }

data expected_zero_cost: Lookup<Int> = Hit(0)

data claim_cost_of_empty_fold: TestClaim = {
  name: "cost_of_empty_fold_returns_zero",
  source: "let x: Int = 0",
  file_name: "fold.v3",
  predicate: LensOutputEquals {
    lens_ref: cost_of_lens,
    input_ref: x_bind_in_source,
    expected_ref: expected_zero_cost
  },
  requires: []
}
```

The shape is identical to existing `t_pb_b_1_execute_command_boundary.dag`. The migration is mechanical: identify the predicate variant for the assertion shape; lift assertion arguments to `.dag` declarations (per `feedback_naming_is_aliasing`); land the `TestClaim`.

### §3.3 Net SG-0 partition flow

Each migration PR retires N Rust test files and lands M `.dag` fixture files (the test text is preserved; only the host language flips). The SG-0 census partitions:

- `EXPECTED_HAND_AUTHORED_TEST` shrinks by N (Rust files retired).
- `EXPECTED_HAND_AUTHORED_NON_TEST` — unchanged (no non-test movement).
- Generated fixtures land outside both partitions (they are `.dag` data, generated test runners are emitted Rust under `target/` — not censused as hand-authored).

R3 close requires `EXPECTED_HAND_AUTHORED_TEST` partition cardinality = 0. The lane closes when that ratchet hits 0 AND all C1–C7 fixtures evaluate green under the runner.

### §3.4 The `m1_5_testgen_test.rs` exception

`m1_5_testgen_test.rs` (per the migration audit in `TESTING.md`) is "exhaustive testgen validation" — the meta-harness that materializes generated TestClaims and validates them. This file is itself the consumer of the migration; it shrinks naturally as generated tests subsume hand-authored ones, but the meta-harness *itself* is the last Rust-test residual to retire. **Resolution per §8.4**: `m1_5_testgen_test.rs` ports to a `.dag` meta-harness that consumes its own emitted test runners — recursive applicability of facet 3 to the meta-test surface itself. The lane verifies this final port.

## §4. Generated target-language test code path

THESIS facet 3 names two parallel rendering paths: `.dag TestClaim` declarations (Path A — runner-evaluated) and **generated target-language test code** (Path B — emitted into target test runners). Path A is implemented today; Path B requires per-target rendering tables for each `TestPredicate` variant.

### §4.1 Per-target rendering schema

Each emitter (`emit_rust_module` / `emit_python_module` / `emit_go_module`) gains a `render_test_claim` entry that consumes a `TestClaim` and produces target-language test code. The rendering is structural — one match arm per `TestPredicate` variant per target.

For Rust target:

```rust
// Target-language rendering for `Compiles` predicate, Rust target.
//
// Input:  TestClaim { name: "...", source: "...", predicate: Compiles, ... }
// Output: a Rust `#[test] fn` whose body invokes the v3 compiler on
//         `source` and asserts no diagnostic.
```

For Python target: emits `def test_*(self):` methods on a generated `unittest.TestCase` subclass. For Go target: emits `func Test*(t *testing.T)` functions.

The per-target rendering lives in the existing per-target template authority (`src/v3/std/rust_method_template_contracts.dag`, `src/v3/std/python_method_template_contracts.dag`, `src/v3/std/go_method_template_contracts.dag`) — extended with a `test_claim_template_contract` row per predicate variant. **Cost-of-change=1 satisfied**: adding a new `TestPredicate` variant requires editing the per-target rendering rows in the three `.dag` template files, not the emitter Rust code.

### §4.2 What generated test code asserts

The generated test runs the same predicate against the same source — but compiles `source` to that target rather than to v3's native runner. For `Compiles`: the generated Rust test invokes `v3_compiler::compile_to_dag(source, file_name)` and asserts no diagnostic. For `BehavioralObservation`: the generated test compiles `source` to Rust, runs the resulting binary on `input_sample`, compares stdout to `expected_output`. For `ExecuteCommand`: the generated test invokes `std::process::Command` and asserts exit code (this is what `t_pb_b_1_execute_command_boundary.dag` already does in `.dag`-runner form).

**Equivalence claim**: Path A and Path B against the same `TestClaim` produce structurally equivalent results — Pass iff Pass — modulo float / nondeterminism policy already declared in `docs/design-cross-target-equivalence.md`. The R3 lane validates this equivalence as one of its closure gates.

### §4.3 When a user wants Path B vs Path A

Path A is the default (faster, one runner, no target toolchain dependency). Path B is opt-in via a `TestSuite.target_emission` field (or equivalent — see §8.5 for the exact shape):

```dag
type TargetEmission
  = NativeRunnerOnly         // Path A only (default)
  | EmitToTargets(List<TargetRef>)   // Path A + Path B for the listed targets

type TestSuite {
  name: String
  claims: List<SuiteClaim>
  target_emission: TargetEmission   // default NativeRunnerOnly
}
```

A user writing test claims for cross-target equivalence proof opts into `EmitToTargets([rust, python, go])`; the emitted target test code goes into `target/<lang>/tests/` (mechanical emission path; the existing emit infrastructure already writes to those paths).

## §5. Cementing test discipline

Cementing is the existing discipline named by `TESTING.md` §"Cementing tests (Band C — lens subsumption)" and `docs/v3-lens-capability-register.md` Discipline §6. The R3 lane closes the discipline to **zero gaps** under the lens-register ratchet.

### §5.1 What cementing means today

A cementing test is a behavioral regression that compares v3 lens output to v2 oracle output on the same source. Per `TESTING.md`:

> When the register row marks `BEHAVIORALLY COMPLETE` while still naming a real v2 counterpart (not `None (v3-native)` / not `N/A`), a cementing test exists that runs the same minimal fixture through both implementations (v2 oracle + v3 lens output) and asserts semantic equality on the published carrier shape.

The dispatch ratchet at `src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs` already enforces:

- `CEMENTING_MODULES_FOR_V2_COMPLETE_CLAIMS` lists exactly the registry `name` keys whose register row is `BEHAVIORALLY COMPLETE` + has a real v2 counterpart.
- Each listed module has an on-disk `cementing/<stem>.rs` AND a `#[path = "integration/cementing/<stem>.rs"]` line in `tests/integration.rs`.

### §5.2 Migration to `.dag` cementing claims

The Rust cementing modules port to `.dag` `TestClaim` declarations using the `DifferentialEquals` predicate (already in the substrate per `src/v3/std/verification.dag:178-182`):

```dag
// src/v3/compiler/tests/dag/cementing_complexity_lens.dag
data cementing_complexity_against_v2_oracle: TestClaim = {
  name: "complexity_lens_matches_v2_oracle_on_minimal_fold",
  source: "let total: Int = fold(...) ...",
  file_name: "complexity_cementing.v3",
  predicate: DifferentialEquals {
    subject_ref: v3_complexity_lens,
    oracle_ref: v2_complexity_oracle,
    input_ref: complexity_cementing_source
  },
  requires: []
}
```

The `DifferentialEquals` predicate already names the comparison shape (subject vs oracle on shared input). The runner evaluates both and compares carriers structurally. **Equivalence with the existing Rust cementing dispatch**: the `CEMENTING_MODULES_FOR_V2_COMPLETE_CLAIMS` list is replaced by a `.dag` declaration listing `DifferentialEquals` claims; the dispatch test still verifies that every register row with `BEHAVIORALLY COMPLETE` + non-N/A v2 counterpart has a corresponding `.dag` `TestClaim` in the cementing fixture set.

### §5.3 The R3 closure gate

Per the lane row `lens_cementing_test_discipline_complete`: at R3 close, every `LensRegistryEntry` whose register row reads `BEHAVIORALLY COMPLETE` with a real v2 counterpart has:

1. A `.dag` `TestClaim` declaration with `predicate: DifferentialEquals { subject: <v3_lens>, oracle: <v2_lens>, input: <fixture> }`.
2. The dispatch ratchet (`cementing_dispatch.dag` — the `.dag` successor to `cementing_lens_registry_dispatch_test.rs`) verifies the cementing claim list matches the register projection exactly.
3. Each cementing claim evaluates green under the test runner.

The four lenses currently at PROXY/STUB/PARTIAL (complexity, cost, parallelism, effect_enumeration) flip to `BEHAVIORALLY COMPLETE` under T-Lens-Behavioral-Parity; their cementing claims land in the same PR per the existing `TESTING.md` cementing-symmetry rule.

### §5.4 Cementing for v3-native lenses (no v2 counterpart)

Per `TESTING.md` §"Cementing tests (Band C — lens subsumption)":

> **No v2 counterpart (`N/A` / v3-native) but the register marks `BEHAVIORALLY COMPLETE`:** pin the lens's published behavioral contract on **minimal constructed `Dag` shapes** (or a single tiny `compile_to_dag` fixture when the contract genuinely spans the pipeline). This is still a cementing test: it cements the `COMPLETE` row against accidental semantic drift.

For v3-native lenses (`provenance`, `unused_parameters`, `variant_payload`, `structural_resolution`, `idempotency`, `named_function_count`), the cementing claim uses `LensOutputEquals` (subject = v3 lens; expected = `.dag` declaration of the expected output carrier) on a minimal source. Same migration path as §5.2 except the predicate variant differs.

## §6. Implementation order (sketch)

Within T-Tests-As-Data-Completeness (per [`docs/r3-structure.md`](r3-structure.md) lane row gates):

1. **Substrate carriers landing** (`forall_exists_quantifier_substrate_landed`, `program_generator_carrier_landed`). Author `ProgramGenerator`, `ProgramShape`, `Quantifier`, `QuantifiedTestClaim`, `SuiteClaim` extensions to `src/v3/std/verification.dag` per §2. Migration of existing `TestSuite.claims` sites to wrap in `Enumerated(...)`. Bootstrap-only: the `LiteralProgram` shape variant; richer shapes deferred (§8.2 — dissolution trigger named).
2. **Runner extension for quantified claims**. Extend `test_runner::evaluate_claim` to dispatch on `SuiteClaim::Quantified`: walk the generator's `List<ProgramShape>` body, apply the predicate per shape, fold via `ForAll`/`Exists`. Per-target test code rendering for the new shapes (§4) lands here.
3. **Migration audit ratchet** (`tests_as_data_migration_audit.dag`). Author the per-class C1–C7 audit declaration listing every Rust test file with its target class; the ratchet asserts file_count = Σ class_counts. Per-PR migration shrinks the C1–C7 column counts and grows the `.dag` fixture count.
4. **Class-by-class Rust → `.dag` ports** (`every_rust_test_ports_to_dag_or_generated`). One PR per class (or per coherent sub-batch); each PR retires Rust files from `EXPECTED_HAND_AUTHORED_TEST` and lands `.dag` `TestClaim` fixtures. Closes when `EXPECTED_HAND_AUTHORED_TEST` cardinality = 0.
5. **Cementing dispatch port** (`lens_cementing_test_discipline_complete`). Replace `cementing_lens_registry_dispatch_test.rs` with a `.dag` dispatch declaration consuming the lens register projection. Each cementing module ports to a `DifferentialEquals` (or `LensOutputEquals` for v3-native) `TestClaim`. Closure gate: every `BEHAVIORALLY COMPLETE` + non-N/A row has a matching cementing `TestClaim` and evaluates green.
6. **`m1_5_testgen_test.rs` final port** (per §3.4). The meta-harness ports to a `.dag` declaration consuming its own emitted test runners.

Steps 1–2 are sequential (carrier substrate must land before runner consumes it). Steps 3–6 are parallel-dispatchable (each is an independent migration PR consuming the substrate from steps 1–2).

**Closure-gate mapping**:

| Lane gate | Closes at step | Receipt |
|---|---|---|
| `forall_exists_quantifier_substrate_landed` | 1 | New types in `src/v3/std/verification.dag` + isomorphism test against substrate snapshot |
| `program_generator_carrier_landed` | 1 | Same authority, same PR; pinned by an exemplar `QuantifiedTestClaim` evaluating green |
| `every_rust_test_ports_to_dag_or_generated` | 4 (final batch) | `EXPECTED_HAND_AUTHORED_TEST` partition cardinality = 0 in SG-0 census |
| `lens_cementing_test_discipline_complete` | 5 | Cementing dispatch `.dag` matches register projection; all rows green |

## §7. Cross-program coordination

This lane is **owned by Verification Manager**, with **Substrate Manager assisting on the carrier authoring**:

- **Substrate Manager owns**: the `ProgramGenerator` / `ProgramShape` / `Quantifier` / `QuantifiedTestClaim` / `SuiteClaim` carrier additions in `src/v3/std/verification.dag`. The substrate-fact-introduction procedure (`INVARIANTS.md#p1-modeling-faithfulness` Steps 1–3) was applied during this design (see §9). Substrate authoring is a small, focused PR.
- **Verification Manager owns** (lane primary): the migration audit, every Rust→`.dag` port, the cementing dispatch port, the `m1_5_testgen_test.rs` final port. Owns all closure gates. Owns the migration audit declaration that drives per-PR progress reporting.
- **Per-target emission tables (§4)** distribute by emitter: Rust template-contract row authoring is Substrate-adjacent (lives in `rust_method_template_contracts.dag`); same for Python and Go. Verification Manager dispatches the emitter table extensions and owns their behavioral-equivalence tests (Path A vs Path B).

The split mirrors T-Lens-Application-Surface (substrate authors carriers; verification asserts demonstrations). Cross-program coordination is via the standard cross-manager queue + closure-ledger receipts.

**No new manager territory required**: the lane fits cleanly into Verification Manager's existing scope (per `docs/r3-structure.md` §"Manager structure" — Verification Manager owns "structural-acceptance-by-construction").

## §8. Resolved design questions

Five design questions surfaced during authoring. Per `feedback_design_before_implement` ("resolve all design questions before implementation"), each is resolved here rather than deferred.

### §8.1 Should `ForAll` evaluate every member or stop at first failure? — RESOLVED: stop at first failure with typed Diagnostic

**Question:** When `QuantifiedTestClaim { quantifier: ForAll, generator: G, predicate: P }` evaluates, does the runner walk every program in G's family even after the first failure, or stop at the first failing program?

**Resolved:** stop at the first failing program; emit a typed `Diagnostic` naming (a) the failing program shape, (b) the predicate's specific failure (not just "predicate failed" but the structured failure carrier the predicate already produces). Per `INVARIANTS.md#p3-fail-closed` (fail-closed): failing fast at the first counterexample matches "every detectable problem is a Diagnostic" — exhaustive walk after first failure produces noise without information gain (the user already knows the predicate doesn't hold universally).

**Why not "walk all and report all":** a `ForAll` claim is logically refuted by one counterexample. Reporting more counterexamples is a performance cost without semantic content. A user who wants per-program diagnostics writes per-program enumerated `TestClaim`s (the existing surface); `ForAll` is the universal-quantifier semantic, not a "report all failing fixtures" surface.

**Implementation note:** `Exists` is symmetric — stop at first passing program; emit Pass with the witnessing program shape recorded in the ClaimResult. If no member passes, emit Fail with the count of programs walked.

### §8.2 Generator richness — what shapes does `ProgramShape` admit at lane open? — RESOLVED: bootstrap with LiteralProgram only; dissolution trigger named

**Question:** `ProgramShape` is a sum type. At lane open, which variants ship? Future extensions exist (parameterized programs, substrate-derived programs); naming all of them at lane open would inflate scope.

**Resolved:** **bootstrap with `LiteralProgram { source, file_name }` only**. This single variant matches the existing `TestClaim.source` / `file_name` pair and lets every today's enumerated test become a 1-element generator if rewritten as quantified. The substrate ships with `ProgramShape` as a 1-variant sum (per `INVARIANTS.md#p1-modeling-faithfulness` — single-variant sums are structurally honest, admitting future variants without restructuring).

**Dissolution trigger for richer shapes** (per `INVARIANTS.md#p5-progress-is-dissolution` scaffold-discipline): when a real lane (e.g., a property-based testing lane in a future release) needs `ParameterizedProgram { template, parameter_bindings: List<...> }` or `SubstrateDerivedProgram { walker_ref: DeclarationRef }`, that lane authors the new variant alongside its first consumer. **The 1-variant bootstrap does not bridge** — the variant is structurally complete for the literal-program use case; future extensions are additive, not migration-required.

### §8.3 How does the cementing dispatch port preserve the register projection? — RESOLVED: cementing dispatch reads the register declaration directly

**Question:** Today's `cementing_lens_registry_dispatch_test.rs` derives `CEMENTING_MODULES_FOR_V2_COMPLETE_CLAIMS` from `docs/v3-lens-capability-register.md` (markdown) plus `regen.dag` (`LensRegistryEntry` rows) and asserts the slice matches exactly. Under `.dag` migration, the markdown table cannot be parsed by `.dag` code; how does the dispatch ratchet stay in lockstep with the register?

**Resolved:** the lens register migrates from markdown to a `.dag` declaration (`LensCapabilityRegister`) at the same time as the cementing dispatch migration. The register table moves from markdown rows to `.dag` `data` declarations of type `LensCapabilityEntry`; the cementing dispatch `.dag` reads the structured register directly:

```dag
type LensCapabilityEntry {
  lens_ref: DeclarationRef
  structural_status: StructuralStatus
  behavioral_status: BehavioralStatus
  v2_counterpart: V2Counterpart   // sum type: NoCounterpart | Counterpart(DeclarationRef)
}

data lens_capability_register: List<LensCapabilityEntry> = [...]

// The cementing dispatch projection:
fn cementing_modules_for_v2_complete_claims(register: List<LensCapabilityEntry>)
   -> List<CementingClaimRef> = ...
```

The markdown table at `docs/v3-lens-capability-register.md` becomes a *rendering* of the structured register (per `feedback_no_metadata_markers` — the source of truth is structural, not textual; the markdown is generated documentation). **Authority shift** is a one-time migration; afterward, every register update is one `.dag` edit.

**Why not "keep markdown, add `.dag` mirror"**: parallel authority (per `INVARIANTS.md#p2-boundary-discipline`) — markdown and `.dag` would drift. Single-authority requires one source of truth.

**Lane scope**: the register migration (markdown → `.dag` declaration) is in scope for this lane (specifically step 5 — cementing dispatch port). The migration is small (one register declaration ~20 rows) and unblocks the cementing dispatch closure gate.

**Cross-lane sequencing — dependent docs**: the following 4 sibling design docs reference the lens-capability register and depend on this migration landing first for their *register-row update* steps (the substrate work in those lanes does NOT depend on this migration; only the closure-gate "register row updates from PROXY/STUB/PARTIAL → COMPLETE" step does):

| Sibling design | Affected closure step | Sequencing |
|---|---|---|
| [`docs/design-complexity-lens-behavioral-completeness.md`](design-complexity-lens-behavioral-completeness.md) | "complexity.dag row → COMPLETE" | After this lane's step 5 lands |
| [`docs/design-cost-lens-sizevar-dimension-wiring.md`](design-cost-lens-sizevar-dimension-wiring.md) | "cost.dag row → COMPLETE" | After this lane's step 5 lands |
| [`docs/design-effect-enumeration-resource-threading.md`](design-effect-enumeration-resource-threading.md) | "effect_enumeration.dag row → COMPLETE" | After this lane's step 5 lands |
| [`docs/design-lens-application-surface.md`](design-lens-application-surface.md) | (no register row affected — this design adds a new substrate carrier, doesn't change a lens row) | None |

This sequencing constraint is one-way: T-Tests-As-Data-Completeness step 5 must land before any lens row in the register flips from PROXY/STUB/PARTIAL to COMPLETE. The substrate carriers + lens consumer rewrites in those sibling lanes do not block on this migration; only the closure-gate row update does.

### §8.4 What about `m1_5_testgen_test.rs` — does the meta-harness migrate? — RESOLVED: yes, recursive applicability of facet 3

**Question:** `m1_5_testgen_test.rs` is the meta-harness that materializes generated TestClaims and validates them. Does facet 3's "tests are data" apply to itself — does the meta-harness port?

**Resolved:** yes, the meta-harness ports. Per THESIS facet 3 (carrier sentence: *"all pipeline/contract tests are .dag TestClaim data; the prior 'two-residual' carve-out is retracted"*), the meta-harness is itself a pipeline test (it tests the pipeline's testgen output). Its migration shape:

```dag
// src/v3/compiler/tests/dag/testgen_meta_harness.dag
data meta_harness_claim: QuantifiedTestClaim = {
  name: "every materialized TestClaim evaluates as expected",
  generator: ProgramGenerator { generator: materialized_test_claims_generator },
  quantifier: ForAll,
  predicate: BehavioralObservation {
    subject: testgen_claim_evaluator,
    input_sample: <the materialized claim>,
    expected_output: <expected evaluation result>
  },
  requires: []
}
```

The meta-harness becomes a `QuantifiedTestClaim` whose generator is "every materialized test claim" and whose predicate is "evaluator produces the expected outcome." Recursive applicability is structurally honest — the meta-harness is just a test, and tests are data.

**Why not "keep meta-harness as Rust as the bootstrap exception":** that re-introduces the two-residual carve-out THESIS facet 3 retracted. The 0-floor target is unconditional; the meta-harness migration is the last domino.

### §8.5 Per-target test emission opt-in shape — RESOLVED: `TargetEmission` field on `TestSuite`, default `NativeRunnerOnly`

**Question:** §4 named "Path B" (emit target-language test code from `TestClaim` declarations). When does Path B fire — always for every claim, or opt-in?

**Resolved:** opt-in via `TestSuite.target_emission: TargetEmission`, default `NativeRunnerOnly`. Per `INVARIANTS.md#p5-progress-is-dissolution` (progress is dissolution): emitting target-language test code for every claim by default would inflate the `target/` tree by ~Nx (one test runner per claim per target) without the user opting into the cross-target equivalence proof. Opt-in matches user intent and keeps default workflow lean.

**Implementation note:** `TargetEmission` is a sum type per §4.3. The default value (`NativeRunnerOnly`) is set at the `TestSuite` declaration level, not per-claim; per-claim override is unnecessary at lane open (a user wanting per-claim variation declares two suites). Per `INVARIANTS.md#p1-modeling-faithfulness` Step 2, `TargetEmission` is a true coproduct (`NativeRunnerOnly` and `EmitToTargets(...)` are alternatives, not coordinates).

**Why on `TestSuite`, not `TestClaim`:** test suites are the natural emission unit (per-suite Cargo target; per-suite Python module; per-suite Go test file). Per-claim emission would generate one target test runner per claim, exploding the file count. Per-suite emission generates one runner per suite per target — same shape as today's hand-authored Rust test files (one file per suite). The granularity matches existing target idioms.

---

All five questions resolved. Implementation can proceed without Director ratification on these specific points. Cascade gates (R2-Evaluator landed; existing TestClaim infrastructure from DB-15 R2; T-Lens-Behavioral-Parity advancing PROXY→COMPLETE for the cementing dispatch step) and per-step closure gates remain as the only outstanding preconditions.

## §9. Relationship to existing authority

This design doc extends:

- [`docs/design-test-infra.md`](design-test-infra.md) — DB-15 R2's locked schema for `TestClaim` / `TestPredicate` / `TestSuite`. **No changes to existing carriers**; this doc adds `QuantifiedTestClaim` / `ProgramGenerator` / `Quantifier` / `SuiteClaim` as sibling additions, and `TargetEmission` as a `TestSuite` extension. The DB-15 single-authority rule (`src/v3/std/verification.dag`) is preserved.
- [`TESTING.md`](../TESTING.md) §"Cementing tests (Band C — lens subsumption)" — the existing cementing discipline. This doc names the migration of the cementing dispatch ratchet from Rust (`cementing_lens_registry_dispatch_test.rs`) to `.dag` (cementing dispatch declaration), preserving the discipline's semantics.
- [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) — the lens capability register. §8.3 resolves the dispatch-projection question by migrating the register from markdown to a `.dag` declaration in the same lane (one-time migration; afterward every register update is one structural edit).
- [`THESIS.md`](../THESIS.md) facet 3 — *"Tests are data too."* This lane closes facet 3 to full coverage by retracting the prior "two-residual" carve-out at the substrate level (the carve-out was already retracted at the THESIS level under 0-floor; this lane lands the structural receipts).
- [`docs/r3-structure.md`](r3-structure.md) lane T-Tests-As-Data-Completeness — the lane row's four closure gates are §6's step receipts.
- [`../INVARIANTS.md`](../INVARIANTS.md) §P1 (modeling faithfulness) — substrate-fact-introduction procedure applied to `ProgramGenerator` / `Quantifier` (DAG-ancestor check: `TestClaim` is the parent for enumerated; `QuantifiedTestClaim` is sibling for quantified — both attach via the existing `verification.dag` authority. Coproduct-vs-coordinate check: `Quantifier` is alternatives, true coproduct. Primitive-vs-lens-extensible check: `ProgramShape` is lens-extensible — the user-extensible variant set is namespacing per `feedback_groundedness_gates_lenses`).
- [`../INVARIANTS.md`](../INVARIANTS.md) §P2 (boundary discipline) — load-bearing for the single-authority `verification.dag` extension and the §8.3 register migration (no parallel markdown / `.dag` authority).
- [`../INVARIANTS.md`](../INVARIANTS.md) §P3 (fail-closed) — load-bearing for §8.1 (`ForAll` stops at first failure with typed Diagnostic; no warnings, no silent passes).
- [`../INVARIANTS.md`](../INVARIANTS.md) §P5 (progress is dissolution) — load-bearing for §8.2 (1-variant `ProgramShape` bootstrap with named dissolution trigger; no scaffold-as-steady-state) and §8.5 (default `NativeRunnerOnly` emission; opt-in target-emission to avoid steady-state file inflation).
- [`feedback_no_metadata_markers`](../) (memory) — load-bearing for §8.3 (lens register moves from markdown to structural `.dag` declaration; markdown becomes generated rendering, not authority).
- [`feedback_naming_is_aliasing`](../) (memory) — load-bearing for §3 (Rust-test ports lift assertion arguments to `.dag` declarations; expected values are not strings).
- [`feedback_compositional_not_templating`](../) (memory) — load-bearing for §2.5 (quantifier on `QuantifiedTestClaim`, not in `TestPredicate`; the structural separation preserves "predicate = property-of-one-program" semantics).
- [`feedback_audit_adjacent_authority_first`](../) (memory) — applied during authoring: searched `docs/design-*.md`, `docs/v3-lens-capability-register.md`, and `TESTING.md` for adjacent authority on `ForAll`/`Exists`/`ProgramGenerator`/cementing. The only existing references are `ForAllTargets` (a different concept — per-target dispatch over the existing target set, not quantification over a generated program family) and the cementing discipline (which this doc extends, not parallel-authors). No prior design doc names `ProgramGenerator` or quantifier carriers; this is the canonical authoring site.

This document does NOT modify:

- The existing `TestClaim` shape (per DB-15 R2 lock — sibling, not replacement).
- The existing `TestPredicate` coproduct (the 22 variants are unchanged; quantification is at the claim level per §2.5).
- The existing per-target template contract authority structure (`rust_method_template_contracts.dag` etc. — extended with new rows, not restructured).
- The existing cementing discipline rules (`TESTING.md` §"Cementing tests" — migration shifts the dispatch carrier, preserves the discipline semantics exactly).

---

**This document is a design spec, not a ship target.** It resolves the structural design questions blocking T-Tests-As-Data-Completeness lane dispatch. The lane runs once cascade gates clear (R2-Evaluator landed; existing TestClaim infrastructure from DB-15 R2). All §8 design questions resolved in-doc; no Director ratification required before substrate authoring begins.
