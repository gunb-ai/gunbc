# Lane A PREFIX — Acceptance artifact (b): cost + complexity (Wave-1 #1 complexity + §5.1 synthesis) — **operator-signed gate for `DISPATCH_HOLD`**

> **Sister artifacts:** `docs/briefs/r4-lane-a-lens-prefix-acceptance.md` (batch (a) — driver/registry + whole-corpus gate). **Brief:** `docs/briefs/r4-lane-a-lens-prefix-t23-t12-ci.md` (`PREFIX-LENS-CI-1`) — **Acceptance-PR batches §** row **3**. **Authority:** `docs/design-lens-framework.md` §2 (`Witness<C>`, `DimensionOk` / `DimensionFail`, `DimensionReport`); `docs/design-lens-application-surface.md` §4.1 (complexity-contract compile error) + §4.2/§4.3 (cost basis worked examples) + **§5.1 default-application synthesis** (operator dev-speed lever — Wave-1 #1 complexity rides this).

## Purpose

Immutable **witness + runnable acceptance table** for the **cost + complexity** PREFIX-lens batch (one shared Acceptance PR for both per **`PREFIX-LENS-CI-1` §Acceptance-PR batches** row 3). **`DISPATCH_HOLD`** on the worker brief lifts for **complexity** + **cost** implementation when this document is **committed on the Acceptance PR branch** and **signed by the operator** (per witty-cat methodology).

**Immutability:** implementation workers **do not** edit witness blocks or expectation rows except **red→green** transitions accompanied by an **operator-signed** amendment to **this** file on the Acceptance PR.

**Why this batch is one PR, not two:** complexity is the **asymptotic projection** of cost's `SymbolicCost` (`src/v4/lens/complexity.dag` U2 header, operator-ratified 2026-05-15); the two lenses share one substrate algebra, one fold-pass, and one `LensEnforcement` family. Splitting their Acceptance signatures would re-litigate `cost ↔ complexity` authority. The shared substrate is the **`SymbolicCost` lattice** declared in `src/v4/lens/cost.dag` (U2: total over the closed kernel — Constant / Linear / Polynomial / PolyLog / Exponential / Factorial / Log / Sum / Product / `UnknownCost`-as-top).

## Issue class (one sentence)

**Cost** (real `Node -> Witness<SymbolicCost>` fold) and **complexity** (asymptotic projection over the **same** fold, plus **§5.1 synthesized `IntrospectApplication<ComplexitySummary>`** for every unannotated function) apply over v4 `.dag` programs as **typed lens applications** — `EnforcedApplication<…>` fires `Diagnostic`s through the existing `Witness<C>` / `DimensionFail` channel **fail-closed**; `IntrospectApplication<ComplexitySummary>` is **synthesized** (not authored) for every function and feeds downstream composition without ever emitting a Diagnostic; CI merge gates stay green while red witnesses are proven by **passing** `cargo test` assertions of structured `Violates` / `DimensionFail` outcomes (never by requiring a failing workflow job for a diagnostic fixture).

## Runnable acceptance table (map every row to §2 of `design-lens-framework.md`)

| ID | Runnable command / surface | Expected **§2** outcome | Notes |
|----|-----------------------------|-------------------------|--------|
| AC-0 | `v2-compiler compile --source-root src/v4` | Compile succeeds; **0** `Diagnostic`s | Unchanged spine bar; `lens/cost.dag` + `lens/complexity.dag` parseable carriers (T-12 fill) do not regress the source-root gate. |
| AC-1 | Whole-corpus driver step — *interim:* `<TBD — slot after Interface-Freeze keystone>` / v3 fold over `src/v4/**/*.dag` per **Fork A** (delete-dated `SUPPLEMENTARY` step) | **`DimensionOk`** aggregate over the enumerated **green-corpus** contract (driver completes; **exit 0** on policy match — zero **unexpected** `Violates` / `DimensionFail` escapes) | Operator pins corpus file-sets in §Green corpus pin below. Per **`design-lens-application-surface.md` §5** ~L355–L362 the cost is **O(applications)**, not O(files²). |
| AC-2-COMPLEXITY-RED | `cargo test -p v3-compiler complexity_violation_compile_error_demonstrated` (or its v4 `TestClaim` runner equivalent per **DB-15**) | **`DimensionFail` / `Violates { reason: "asymptotic class O(n²) exceeds declared budget O(log n)", at: <behavior> }`** on **red** snippet; **test passes** (asserted structured failure) | Cites **`design-lens-application-surface.md` §4.1** closure gate `complexity_violation_compile_error_demonstrated`. ≥1 red row in §Witnesses. |
| AC-2-COMPLEXITY-GREEN | Same harness on **green** snippet | **`DimensionOk` / `Inhabits(ComplexityBound::O_log_n)`**; **test passes** | ≥1 green row in §Witnesses. |
| AC-3-COST-RED | `cargo test -p v3-compiler cost_basis_violation_demonstrated` (CRDT or memory-peak — operator chooses witness) | **`DimensionFail`** with typed `Diagnostic` naming the per-`SymbolicCost` lattice element that escapes the declared budget; **test passes** | Cites **§4.2** / **§4.3** worked examples + closure gates `crdt_cost_basis_demonstrated` / `memory_peak_cost_basis_demonstrated`. ≥1 red row in §Witnesses. |
| AC-3-COST-GREEN | Same harness on **green** snippet | **`DimensionOk` / `Inhabits(SymbolicCost::…)`** matching declared budget; **test passes** | ≥1 green row in §Witnesses. |
| AC-4-SYNTHESIS | `cargo test -p v3-compiler default_introspect_synthesis_total_over_functions` | For an `.dag` file with **N** function declarations and **zero** `apply_lens(complexity, …)` authorings, the fold pass produces **N** `IntrospectApplication<ComplexitySummary>` records; **zero** `Diagnostic`s emitted (Introspect never fires Enforce per **§5.1** + **§8.3**); **test passes** | **§5.1** synthesizer: one record per function; never `Enforce`. |
| AC-5-NEAR-MISS | `cargo test -p v3-compiler complexity_lens_does_not_fire_on_unrelated_lens_application` | An `apply_lens(parallelism, fn, Enforce { … })` declaration in the **same** program produces **zero** complexity diagnostics from the complexity-walk (sibling-lens isolation per **§5** two-separate-walks rule); **test passes** | Sibling-not-caught negative — required by lens-design-TDD discipline (≥1 per Wave-1 lens). |

**Authoring rules consumed (do not re-litigate here):**

- **§3.2** — default policy for unannotated complexity contracts is **user-driven**; the synthesizer in **§5.1** emits **`Introspect` only**, never `Enforce` (per **§8.3** RESOLVED — no implicit baseline).
- **§8.5** — cross-section composition reads **declared budget**, not computed class.
- **U2** (`src/v4/lens/complexity.dag` header) — complexity NEVER independently re-derives cost; it **consumes** `cost.dag`'s `SymbolicCost` and projects asymptotically.

## Witnesses (**≥1 red + ≥1 green** per AC pair — operator fills snippets; candidate-authored substance per `PREFIX-LENS-CI-1` §Witness parallelism)

### Complexity — red witness (expected `DimensionFail` / `Violates`)

- **Fixture id:** `_TODO_OPERATOR_` (candidate: `complexity_violation_O_n_squared_exceeds_O_log_n`)
- **Snippet / path:** `_TODO_OPERATOR_` (candidate: a 2-arg `.dag` function performing a nested `fold` over a `List<List<T>>` with `apply_lens(complexity, fn, Enforce { budget: O_log_n, diagnostic_severity: Error })` — the nested fold composes to `SymbolicCost::Product(Linear, Linear)` ⇒ asymptotic projection `ComplexityBound::O_n_squared` ≻ declared `O_log_n`)
- **Asserted outcome (§2):** `Witness::Violates { reason: "asymptotic class O(n²) exceeds declared budget O(log n)", at: <behavior of nested-fold node> }` ⇒ `DimensionFail.violations` carries a typed `Diagnostic` naming both the application span and the offending node

### Complexity — green witness (expected `DimensionOk` / `Inhabits`)

- **Fixture id:** `_TODO_OPERATOR_` (candidate: `complexity_inhabits_O_log_n_binary_search`)
- **Snippet / path:** `_TODO_OPERATOR_` (candidate: a binary-search function on a sorted `List<T>` whose `SymbolicCost` composes to `Log`; same `Enforce { budget: O_log_n }`)
- **Asserted outcome (§2):** `Witness::Inhabits(ComplexityBound::O_log_n)` ⇒ `DimensionOk { dimension_name: "complexity", composed: O_log_n, witnesses: [...per-step Inhabits...] }`

### Complexity — near-miss (sibling-not-caught) witness

- **Fixture id:** `_TODO_OPERATOR_` (candidate: `complexity_walk_does_not_fire_on_parallelism_application`)
- **Snippet / path:** `_TODO_OPERATOR_` (candidate: same green binary-search but the program *also* contains an `apply_lens(parallelism, other_loop, Enforce { … })`)
- **Asserted outcome (§2):** complexity walk produces exactly the binary-search `Inhabits` witness; **zero** complexity-walk diagnostics attributable to the parallelism application (two-separate-walks per **§5**)

### Cost — red witness (expected `DimensionFail` / `Violates`)

- **Fixture id:** `_TODO_OPERATOR_` (candidate: `cost_crdt_per_write_exceeds_O_log_replicas`)
- **Snippet / path:** `_TODO_OPERATOR_` (candidate: a CRDT field whose per-write cost basis declares `O_n` while `apply_lens(cost, my_crdt_field, Enforce { budget: SymbolicCost { per_op: O_log_replicas }, … })`; **§4.2**)
- **Asserted outcome (§2):** `Witness::Violates { reason: "per-write cost O(n) exceeds declared budget O(log replicas)", at: <behavior of write site> }` ⇒ `DimensionFail` with typed `Diagnostic` per closure gate `crdt_cost_basis_demonstrated`

### Cost — green witness (expected `DimensionOk` / `Inhabits`)

- **Fixture id:** `_TODO_OPERATOR_` (candidate: `cost_crdt_per_write_inhabits_O_log_replicas`)
- **Snippet / path:** `_TODO_OPERATOR_` (candidate: same CRDT shape with declared `PerWrite(O_log_replicas)` matching the budget)
- **Asserted outcome (§2):** `Witness::Inhabits(SymbolicCost { per_op: O_log_replicas })` ⇒ `DimensionOk`

### §5.1 default-application synthesis witness

- **Fixture id:** `_TODO_OPERATOR_` (candidate: `synthesis_introspect_only_unannotated_functions`)
- **Snippet / path:** `_TODO_OPERATOR_` (candidate: a `.dag` file declaring **3** functions, **zero** `apply_lens` authorings; fold-pass output is observed via the registry hook)
- **Asserted outcome (§2):** fold pass produces exactly **3** `IntrospectApplication<ComplexitySummary>` records (one per function declaration); **zero** `EnforcedApplication` records synthesized; **zero** `Diagnostic`s emitted. Per **§5.1**: synthesizer **never** emits `Enforce` (auto-inferred budgets lack persisted authority per **§8.3**).

## Green corpus pin (for AC-1)

Operator enumerates which `.dag` paths the **whole-corpus job** treats as **must-pass** for the cost+complexity walks specifically; until filled, reference **`design-lens-application-surface.md` §1 / §3** whole-tree glob carrier and **§5** O(applications) feasibility. Candidate default (operator-overridable): `src/v4/**/*.dag` minus `src/v4/test/fixture/**` (fixture corpus may carry intentional red snippets), aggregate policy = **zero unexpected `Violates`**, where "expected" = anything enumerated in §Witnesses red rows of this artifact or sister batch (a) acceptance artifact.

## Interface-Freeze interaction (per `PREFIX-LENS-CI-1` §Witness parallelism vs Interface-Freeze)

This artifact's witness substance is **parallel-authorable today** (issue-class prose anchored on existing v3 lens behavior + worked examples in `design-lens-application-surface.md` §4.1/§4.2/§4.3/§5.1). The **runnable invocation column** of AC-1 carries an explicit **`<TBD — slot after Interface-Freeze keystone>`** placeholder for the v4 driver one-liner; until **batch (a) — Interface-Freeze pin** lands, AC-1 falls back to the **delete-dated v3 fold + `v2-compiler compile`** invocations per **Fork A**. **Do not** pretend the v4 CLI is already frozen.

**Structural AST assertions** that depend on `application.dag` body fill (e.g. asserting `EnforcedApplication<ComplexitySummary, AsymptoticClass, AsymptoticClass>` parse-tree shape against checked-in v4 source) serialize on **T-23 keystone**; the harness rows above are written against **§2 `Witness<C>`** carrier names that are stable in the framework doc today.

## P5 / SG-0 receipt note

Any **new** `src/v3/compiler/tests/**` integration file added to support AC-2 / AC-3 / AC-4 / AC-5 MUST land with **INVARIANTS.md §P5 Mechanism (b)** row + **`ROADMAP.md` `T-PB-B` / `pb_rust_tests_outside_residual_zero`** citation + **`sg0_census_test.rs`** line in the **same PR** — see worker brief `PREFIX-LENS-CI-1` §P5 / SG-0. Anti-pattern (forbidden): a "diagnostic fixture" implemented as "the CI workflow step must exit non-zero to prove we emit `Diagnostic`" — red witnesses are **harness-asserted passing tests**, never branch-unmergeable failing jobs.

## DECISIONS.md cross-receipts (operator-amended on red→green transitions)

- **C7 / report / synthesis** rows in `src/v4/DECISIONS.md` citing **T-12** carry the closure-gate receipts for `complexity_violation_compile_error_demonstrated` / `crdt_cost_basis_demonstrated` / `memory_peak_cost_basis_demonstrated` (Verification-Manager-owned per `design-lens-application-surface.md` §6).
- **§5.1 synthesizer** receipt — when AC-4 lands green, `DECISIONS.md` records the default-synthesis-Introspect-only ratification (operator dev-speed lever per **`PREFIX-LENS-CI-1` §WHY THIS MATTERS**).

## DISPATCH_HOLD lift conditions (this artifact only)

1. Operator signs each `_TODO_OPERATOR_` slot (fixture ids + snippet paths + asserted outcomes) — or accepts the candidate substance verbatim.
2. Operator pins **Green corpus** for AC-1.
3. Operator confirms `<TBD>` runnable cell in AC-1 with either (a) live `driver …` argv once batch (a) Interface-Freeze lands, or (b) explicit v3-fold-fallback invocation per Fork A delete-dated supplementary step.

Once signed, `PREFIX-LENS-CI-1` **`DISPATCH_HOLD`** lifts for **complexity** + **cost** implementation; T-12 fill proceeds per worker brief Slice B (cost first as PREFIX reference / algebra root; complexity composes on as **Wave-1 #1**).
