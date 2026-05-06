# Thesis Validation Plan

> **Parent docs:** `THESIS.md` (the claims), `src/v3/SELF_HOSTING.md`
> (the implementation roadmap), `docs/v3-validation-experiments.md`
> (the M0-era experiment log this extends).
>
> **Purpose:** map every testable thesis claim to a concrete
> validation, a milestone where it becomes provable, and a test
> fixture that serves as the regression gate. The claims are
> organized by the thesis's own three-tier structure (Tier 1:
> structural bugs impossible by construction, Tier 2: runtime
> safety, Tier 3: verification from structure) plus the killer
> features (KF-1 through KF-8).
>
> **Principle:** each claim is either (a) tested today with a
> regression fixture, (b) testable at a named milestone with a
> described fixture, or (c) explicitly deferred with a named
> blocker. No claim lives in an ambiguous "we'll get to it"
> state.

---

## How to read this document

Each claim has:
- **Thesis reference:** where in THESIS.md the claim lives
- **Testable form:** what a test would check concretely
- **Status:** TESTED (fixture exists), TESTABLE AT (named
  milestone), or DEFERRED (named blocker)
- **Fixture sketch:** what the test program and assertion look
  like
- **Regression value:** what breaks if this regresses

---

## Tier 1: Structural bugs — impossible by construction

### T1.1 Field typos in generated code

**Thesis ref:** §"Zero bugs" Tier 1 table, row 1.
**Testable form:** the emitter derives field names from
declarations, never from string literals. A field rename in the
`.dag` source propagates to all emitted code.
**Status:** TESTED (v3 emit_rust tests).
**Fixture:** rename a field in a type declaration, recompile,
verify emitted Rust uses the new name.
**Regression value:** if this breaks, the emitter has a hardcoded
field name — a layer-opacity violation.

### T1.2 Field typos in `.dag` source

**Thesis ref:** §"Zero bugs" Tier 1 table, row 2.
**Testable form:** accessing a nonexistent field on a record
produces a FieldNotFound diagnostic naming the field.
**Status:** TESTED (Prereq 1 tests — `prereq1_nonexistent_field_is_rejected_with_field_name`).
**Fixture:** `fn f(p: { a: Int, b: Int }) -> Int = p.c` → diagnostic.
**Regression value:** if this breaks, the compiler silently
invents a field.

### T1.3 Non-exhaustive match

**Thesis ref:** §"Zero bugs" Tier 1 table, row 3.
**Testable form:** a match expression that doesn't cover all
variants of a Disj produces a NonExhaustiveMatch diagnostic.
**Status:** TESTED (M1(2.8) tests).
**Fixture:** `type AB = A | B; fn f(x: AB) -> Int = match x { A => 1 }` → diagnostic.
**Regression value:** if this breaks, runtime dispatch on an
unhandled variant crashes instead of failing at compile time.

### T1.4 Type mismatches

**Thesis ref:** §"Zero bugs" Tier 1 table, row 4.
**Testable form:** passing a value of type A where type B is
expected produces a TypeMismatch diagnostic.
**Status:** TESTED (inference tests throughout).
**Fixture:** `fn f(x: Int) -> Bool = x` → diagnostic.
**Regression value:** if this breaks, the compiler silently
coerces incompatible types.

### T1.5 Non-termination — structural descent proof

**Thesis ref:** §"Zero bugs" Tier 1 table, row 11. Also
§"Decidability Invariant", §"Recursive syntax is sugar."
**Testable form:** every recursive call pattern lowers to a
bounded form (fold/descend/repeat). A call pattern with no
provable bound fails with a diagnostic.
**Status:** PARTIALLY TESTED.
- Self-call with `n - 1`: TESTED (v3 termination checker
  accepts numeric descent)
- Self-call on `tail` of `List<T>`: TESTABLE AT current
  reflection work (structural list carrier landed)
- Mutual recursion (SCC): TESTABLE AT §2.4 mutual recursion
  lowering prereq
- Self-call with unchanged argument (`repeat(max_int)`):
  DEFERRED (v3 doesn't support this pattern yet)

**Fixture sketches:**

```dag
// Numeric descent — should compile (TESTED)
fn countdown(n: Int) -> Int =
  if n == 0 then 0 else countdown(n - 1)

// Structural descent — should compile (TESTABLE NOW)
fn count(list: List<Int>) -> Int =
  match list { Empty => 0, Cons(p) => 1 + count(p.tail) }

// No descent evidence — should FAIL (TESTED)
fn diverge(x: Int) -> Int = diverge(x)

// Mutual recursion on children — should compile (TESTABLE AT §2.4)
fn even(list: List<Int>) -> Bool =
  match list { Empty => true, Cons(p) => odd(p.tail) }
fn odd(list: List<Int>) -> Bool =
  match list { Empty => false, Cons(p) => even(p.tail) }

// Mutual recursion without descent — should FAIL (TESTABLE AT §2.4)
fn ping(x: Int) -> Int = pong(x)
fn pong(x: Int) -> Int = ping(x)
```

**Regression value:** if descent checking regresses, programs
that don't terminate compile silently — the decidability
invariant is violated.

### T1.6 Non-idempotent workflow detection

**Thesis ref:** §"Zero bugs" Tier 1 table, row 12. Also
§"Algebraic simplification."
**Testable form:** a workflow composed of operations where one
operation's effect is NOT idempotent is flagged when the workflow
is declared idempotent.
**Status:** DEFERRED.
**Blocker:** Effect shapes (std/effects.dag) exist in v2 but v3
doesn't consume them. Requires L2 effects lens migration.
**Milestone:** L2 M3 (effects lens, ~66 lines).

**Fixture sketch:**

```dag
// Idempotent workflow (all operations are lattice effects)
fn upsert_config(key: String, value: String) -> Result =
  put_secret(key, value)    // effect: MapUpsert → idempotent

// Non-idempotent workflow (append is NOT idempotent)
fn log_event(event: Event) -> Result =
  append_log(event)          // effect: ListAppend → NOT idempotent

// Composition should flag:
fn deploy_and_log(config: Config) -> Result {
  let _ = upsert_config(config.key, config.value)  // idempotent
  let _ = log_event(config.change_event)            // NOT idempotent
  // If this workflow is declared idempotent, the compiler should
  // flag log_event as the breaking operation
}
```

**Regression value:** if this works and then regresses, workflows
that are supposed to be safely retryable silently become
non-retryable.

### T1.7 Cross-target drift — impossible by construction

**Thesis ref:** §"Zero bugs" Tier 1 table, row 9. Also
§"Omni-emission."
**Testable form:** the same `.dag` declaration emitted to two
Shape A targets produces structurally equivalent code (same
fields, same types, same function signatures).
**Status:** DEFERRED.
**Blocker:** Only one Shape A target (Rust) exists in v3. Needs
M1(4) multi-target emission (go.dag + python.dag).
**Milestone:** M1(4).

**Fixture sketch:**

```dag
type Order { customer_id: String, total: Int }
fn create_order(draft: Order) -> Order = draft
```

Emit to Rust → `struct Order { pub customer_id: String, pub total: i64 }`
Emit to Go → `type Order struct { CustomerID string; Total int64 }`
Both declare the same fields with the same types. Adding a field
to `Order` in `.dag` adds it to both targets automatically.

**Regression value:** if this breaks, frontend/backend drift
becomes possible — the thesis's central coherence claim fails.

---

## Tier 1.5: Compositional bugs — caught by causal chain analysis

These are the bugs from our conversation — not LOCAL type errors
but CONTRADICTIONS across a composition chain. No normal compiler
catches them because they require tracing properties through the
full dependency graph.

### T1.5.1 Effect contradiction across chain

**Thesis ref:** §"Correctness dimensions" + §"Algebraic
simplification."
**Testable form:** a downstream operation depends on state that
an intermediate operation mutates, breaking the causal chain.
**Status:** DEFERRED.
**Blocker:** Effect shapes not carried on transforms. Requires L2
M3 effects lens + SubValueRelation.
**Milestone:** L2 M3.

### T1.5.2 Algebraic law violation — dead work detection

**Thesis ref:** §"Concept unification" → "idempotency +
cancellation + redundancy = algebraic simplification."
**Testable form:** an operation is upstream of a consumer that
doesn't need the operation's contribution (e.g., `sort_by` before
a commutative `fold`).
**Status:** DEFERRED.
**Blocker:** CX needs symbolic cost + algebra awareness. Requires
L2 M1 complexity lens with CommutativeMonoid checks.
**Milestone:** L2 M1.

**Fixture sketch:**

```dag
fn wasteful_sort(items: List<Int>) -> Int =
  items |> sort_by(|a, b| a - b) |> fold(0, |acc, x| acc + x)
// sort_by is O(n log n) but fold(0, add) is commutative
// the sort contributes nothing — dead work
```

### T1.5.3 Provenance loss across transform chain

**Thesis ref:** §"Epistemic stacking" (every fact is traceable)
+ §"Facts Flow Forward" invariant.
**Testable form:** a field produced at step N, removed at step M,
and required at step K (where N < M < K) produces a diagnostic
at step K naming the removal at step M.
**Status:** DEFERRED.
**Blocker:** Per-field provenance not tracked. Requires
SubValueRelation + provenance lens.
**Milestone:** L2 (after SubValueRelation substrate gap closes).

### T1.5.4 Complexity explosion in composition

**Thesis ref:** KF-1. §"Complexity proof on every compile."
**Testable form:** a nested operation whose cost is multiplicative
with an outer operation produces a complexity diagnostic showing
the cost breakdown.
**Status:** PARTIALLY TESTABLE.
- Structural operation counting: TESTED (`lens_cost` exists)
- Symbolic O(n) vs O(n²) distinction: DEFERRED (needs L2 M1
  complexity lens with SizeExpr/CostExpr)
- Nested-capture detection (lambda captures outer list and
  filters it per element): DEFERRED (same)

**Fixture sketch (testable NOW with lens_cost):**

```dag
// Compare structural cost: flat vs nested
fn flat_sum(items: List<Int>) -> Int =
  fold(items, 0, |acc, x| acc + x)
// lens_cost: O(n) operations

fn nested_filter(items: List<Int>) -> List<Int> =
  map(items, |x| fold(items, 0, |acc, y| if y == x then acc + 1 else acc))
// lens_cost: the inner fold runs per element of the outer map
// structural cost is higher — the lens should report this
```

**Fixture sketch (testable at L2 M1 with symbolic CX):**

```dag
// Same programs, but complexity lens reports:
// flat_sum: O(n)
// nested_filter: O(n²)
// and flags: "inner fold captures outer `items` — cost is
// multiplicative, not additive"
```

---

## Tier 2: Runtime safety — proven safe or total

### T2.1 Division by zero

**Thesis ref:** §"Zero bugs" Tier 2 table, row 1.
**Testable form:** division by a value that could be zero fails
at compile time (refinement type `NonZero<Int>` required) or the
operation returns `Option<Int>`.
**Status:** DEFERRED.
**Blocker:** Refinement types (Track 11, design phase).
**Milestone:** L3+ (after self-hosting, refinement types land).

### T2.2 Integer overflow

**Thesis ref:** §"Zero bugs" Tier 2 table, row 2.
**Testable form:** arithmetic that could overflow either fails at
compile time (bounded arithmetic proof) or uses checked ops.
**Status:** DEFERRED.
**Blocker:** Same as T2.1.
**Milestone:** L3+.

### T2.3 Out-of-bounds access

**Thesis ref:** §"Zero bugs" Tier 2 table, row 3.
**Testable form:** indexing a collection without a bounds proof
either fails at compile time or returns `Option<T>`.
**Status:** DEFERRED.
**Blocker:** Same as T2.1.
**Milestone:** L3+.

### T2.4 Optional force-unwrap

**Thesis ref:** §"Zero bugs" Tier 2 table, row 4.
**Testable form:** extracting a value from `Option<T>` requires
a match — no `.force()` or `.unwrap()` equivalent exists.
**Status:** PARTIALLY TESTABLE.
- Match on Option variants: TESTED (Prereq 2 + Bool/Option as
  Disj in types.dag)
- No force-unwrap primitive exists: TESTED by omission (the
  language simply doesn't have one)
- Completeness: DEFERRED (all runtime helpers must be total)

---

## Tier 3: Verification from structure

### T3.1 L4 — Semantic correctness

**Thesis ref:** §"Zero bugs" Tier 3, L4 row.
**Testable form:** for any `.dag` function, evaluate it in the
interpreter and in emitted Rust — results must agree.
**Status:** PARTIALLY TESTABLE.
- Emitted Rust roundtrips: TESTED (emit_rust tests execute
  emitted code and check output values)
- Interpreter evaluation: DEFERRED (v3 interpreter doesn't exist
  yet; §4.4 Path C)
- Interpreter vs emitted comparison: DEFERRED (needs both paths)
**Milestone:** After §4.4 Path C (interpreter) lands.

### T3.2 L5 — Cross-language equivalence

**Thesis ref:** §"Zero bugs" Tier 3, L5 row. KF-4.
**Testable form:** same `.dag` → same behavior in Rust, Python,
Go.
**Status:** DEFERRED.
**Blocker:** Only Rust target exists. Needs M1(4).
**Milestone:** M1(4).

### T3.3 L7 — Algebraic law verification

**Thesis ref:** §"Zero bugs" Tier 3, L7 row.
**Testable form:** for types that inhabit algebras, the declared
laws hold — `fold(identity, x) == x`, `concat(a, concat(b, c))
== concat(concat(a, b), c)`.
**Status:** DEFERRED.
**Blocker:** Witness generation + algebra-law test generation.
**Milestone:** L2 M1+ (after complexity lens can walk algebra
inhabitants).

---

## Killer features

### KF-1: Complexity proof on every compile

**Thesis ref:** ROADMAP §"Killer features" KF-1.
**Testable form:** every function gets a proven asymptotic bound.
No `Conservative`. No `Unknown`. Every function is `Proven`.
**Status:** PARTIALLY TESTABLE.
- Integer cost counting: TESTED (`lens_cost`)
- Symbolic bounds (O(n), O(n²)): DEFERRED (needs L2 M1)
- Tight bounds (not just upper): DEFERRED (needs L2 M1)
**Milestone:** L2 M1.

### KF-2: Reject suboptimal algorithms

**Thesis ref:** ROADMAP §"Killer features" KF-2.
**Testable form:** the compiler refuses to compile code when a
provably cheaper equivalent exists.
**Status:** DEFERRED.
**Blocker:** Needs KF-1 (working cost algebra) + equivalence
catalog in `std/optimization.dag`.
**Milestone:** After KF-1.

### KF-3: Verification from structure (free tests)

**Thesis ref:** ROADMAP §"Killer features" KF-3. Design:
SELF_HOSTING.md §14.
**Testable form:** add a type → verification appears. Add a
service → integration test appears. No hand-written tests.
**Status:** PARTIALLY TESTED.
- L0 coercion tests from data: TESTED (v2).
- Structural testgen (from types): TESTABLE AT L1.5 (§14.1)
- Behavioral testgen (from transforms): TESTABLE AT L2.6 (§14.2)
- Mock generation + dry-run: TESTABLE AT L2.6 (§14.3)
- Composition testgen (from pipelines): TESTABLE AT L3 (§14.4)
**Milestone:** L1.5 (first layer), L2.6 (behavioral + mock),
L3 (compositional). Progressively stronger through each layer.

### KF-5: Decidable high-level language

**Thesis ref:** ROADMAP §"Killer features" KF-5.
**Testable form:** all `.dag` programs terminate. Undecidable
programs are structurally unrepresentable.
**Status:** TESTED for the supported call patterns.
- Three bounded primitives (fold/descend/repeat): TESTED
- Fail-closed on unknown descent: TESTED
- Every call pattern in the lowering table: PARTIALLY TESTED
  (mutual recursion pending §2.4)
**Milestone:** Current (base), §2.4 (mutual recursion).

### KF-7: Space complexity

**Thesis ref:** ROADMAP §"Killer features" KF-7.
**Testable form:** every function gets a proven space bound.
**Status:** PARTIALLY TESTABLE.
- Fan-out counting (clone elision): TESTABLE AT L1.5 (§14.7
  Phase 1 — lens_fanout)
- Clone-count ratchet on generated code: TESTABLE AT L1.5
- Full ownership proof (sole-owner, fold linearity): TESTABLE
  AT L2 (§14.7 Phase 3 — ownership.dag migration)
- Proven space bound per function: DEFERRED (needs ownership +
  provenance composition)
**Milestone:** L1.5 (fan-out), L2 (full ownership), L2+
(space bounds).

### KF-8: Optimality gate

**Thesis ref:** ROADMAP §"Killer features" KF-8.
**Testable form:** compile error if a function's complexity
exceeds a declared bound.
**Status:** DEFERRED.
**Blocker:** Needs KF-1 + structural CostBound comparison.
**Milestone:** After KF-1.

---

## Invariant lenses — self-enforcement validation

Each invariant from `INVARIANTS.md` with a structural test should
become a lens (see SELF_HOSTING.md §14.5). The lens running
against the compiler's own `.dag` code validates both the
invariant and the lens mechanism.

### IL-1: Layer opacity lens

**Invariant ref:** [INVARIANTS.md#p2-boundary-discipline](../INVARIANTS.md#p2-boundary-discipline) (Layer Opacity).
**Testable form:** no below-boundary string literal in consumer
code. Rename a std/ identifier → no compile behavior change.
**Status:** PARTIALLY TESTABLE.
- Rename test on weather example: TESTED (v2, 2026-04-15)
- Lens scanning .dag consumer files: TESTABLE AT L1.5
- Lens running on compiler's own .dag: TESTABLE AT L3
**Milestone:** L1.5 (lens on std/ consumers), L3 (self-analysis).

### IL-2: Single authority lens

**Invariant ref:** [INVARIANTS.md#p2-boundary-discipline](../INVARIANTS.md#p2-boundary-discipline) (No duplicate representations).
**Testable form:** for each fact, exactly one declaration is
authority. No `fn(A) -> B` where A and B are structurally
equivalent (parallel representation).
**Status:** DEFERRED.
**Blocker:** Needs structural equivalence check on declarations.
**Milestone:** L2.

### IL-3: Structural dispatch lens

**Invariant ref:** [INVARIANTS.md#p3-fail-closed](../INVARIANTS.md#p3-fail-closed) (No case enumeration for open sets).
**Testable form:** no string-keyed match arm where a structural
walk through declarations would suffice.
**Status:** DEFERRED.
**Blocker:** Needs pattern analysis on match expressions.
**Milestone:** L2.

### IL-4: Cost coverage lens

**Invariant ref:** [INVARIANTS.md#p4-decidability](../INVARIANTS.md#p4-decidability) (Cost algebra upstream of primitives).
**Testable form:** every primitive declaration has a corresponding
`CostExpr` in the algebra. `∀ primitive p, ∃ cost_class(p)`.
**Status:** DEFERRED.
**Blocker:** Needs complexity lens (L2 M1).
**Milestone:** L2 M1.

### IL-5: Facts flow forward lens

**Invariant ref:** [INVARIANTS.md#p2-boundary-discipline](../INVARIANTS.md#p2-boundary-discipline) (Facts Flow Forward).
**Testable form:** no per-field re-derivation across stages. Every
field consumed at stage K traces to a producer at stage N ≤ K.
**Status:** DEFERRED.
**Blocker:** SubValueRelation + per-field provenance.
**Milestone:** L2+.

### IL-6: No fabrication lens

**Invariant ref:** [INVARIANTS.md#p3-fail-closed](../INVARIANTS.md#p3-fail-closed) (No fallbacks that fabricate).
**Testable form:** no catch-and-default pattern without an
explicit error variant in the return type.
**Status:** DEFERRED.
**Blocker:** Effect typing (L2 M3).
**Milestone:** L2 M3.

---

## Diagnostics as corrections — roundtrip testing

**Thesis ref:** THESIS.md §"Error handling: show the correct
code." Design: SELF_HOSTING.md §14.6.

Every diagnostic should carry a correction — the literal fixed
code. The test pattern is a three-assertion roundtrip:
1. Broken source → compile → diagnostic fires (detection)
2. Apply correction → compile corrected source → succeeds (fix correctness)
3. Corrected program has zero diagnostics (fix completeness)

### DC-1: TypeMismatch correction

**Testable form:** `let x: Bool = 1` → TypeMismatch → correction
offers "change annotation to `Int`" or "change value to `true`"
→ apply either → compiles.
**Status:** TESTABLE AT L1.5 (requires Correction field on
Diagnostic).
**Milestone:** L1.5.

### DC-2: NonExhaustiveMatch correction

**Testable form:** `match x { A => 1 }` on `A | B` → diagnostic
→ correction shows `match x { A => 1, B => todo() }` → apply →
compiles.
**Status:** TESTABLE AT L1.5.
**Milestone:** L1.5.

### DC-3: FieldNotFound correction

**Testable form:** `point.c` on `{ a: Int, b: Int }` → diagnostic
→ correction shows available fields `a`, `b` → apply `point.a` →
compiles.
**Status:** TESTABLE AT L1.5.
**Milestone:** L1.5.

### DC-4: ArityMismatch correction

**Testable form:** `f(1, 2)` on `fn f(x: Int) -> Int` → diagnostic
→ correction shows `f(1)` → apply → compiles.
**Status:** TESTABLE AT L1.5.
**Milestone:** L1.5.

### DC-5: UnboundedRecursion correction

**Testable form:** `fn f(x: Int) -> Int = f(x)` → diagnostic →
correction shows `f(x - 1)` or the descent argument needed →
apply → compiles.
**Status:** TESTABLE AT L1.5.
**Milestone:** L1.5.

### DC-6: Lens violation correction (per lens)

**Testable form:** for each lens that reports a violation, the
correction field shows the fixed code. Applying the correction
eliminates the violation when the lens re-runs.
**Status:** TESTABLE AT each lens's ship date.
**Acceptance criterion for every new lens:** roundtrip test
where the fix actually eliminates the lens's reported violation.
**Milestone:** L1.5 (for lens_unused_parameters), then each
subsequent lens.

---

## Summary: what's testable when

| Milestone | Claims testable | Count |
|---|---|---|
| **NOW (current v3)** | T1.1-T1.4, T1.5 (numeric descent), T2.4 (partial), KF-5 (partial), lens_cost structural | ~8 |
| **L1 (reflection PR)** | T1.5 (structural descent on lists), T3.1 (partial — Rust roundtrip) | +2 |
| **L1.5 (clean bootstrap)** | Per-stage fixed-point, KF-3 structural testgen (§14.1), IL-1 layer opacity, DC-1 through DC-5 correction roundtrips, KF-7 fan-out lens + clone ratchet (§14.7 P1) | +9 |
| **§2.4 (mutual recursion)** | T1.5 (SCC descent + SCC rejection) | +2 |
| **L2 (lens migrations)** | IL-2 single authority, IL-3 structural dispatch | +2 |
| **L2 M1 (complexity lens)** | KF-1, T1.5.2 (dead work), T1.5.4 (symbolic O(n²)), IL-4 cost coverage | +4 |
| **L2.7 (ownership lens)** | KF-7 full ownership proof, clone-count ratchet near zero | +2 |
| **L2 M3 (effects lens)** | T1.6 (idempotency), T1.5.1 (effect contradiction), IL-6 no fabrication | +3 |
| **L2.6 (testgen + dry-run)** | KF-3 behavioral testgen (§14.2), mock generation + dry-run (§14.3) | +2 |
| **L2 + SubValueRelation** | T1.5.3 (provenance loss), IL-5 facts flow forward | +2 |
| **M1(4) (multi-target)** | T1.7 (cross-target drift), T3.2 (cross-language equiv) | +2 |
| **L3 (pipeline stages)** | KF-3 composition testgen (§14.4), IL-1/IL-2/IL-3 on compiler .dag (self-analysis) | +4 |
| **L3+ (refinement types)** | T2.1-T2.3 (runtime safety) | +3 |
| **After KF-1** | KF-2 (reject suboptimal), KF-8 (optimality gate) | +2 |

---

## When this doc updates

Each milestone landing should:
1. Move the relevant claims from TESTABLE AT to TESTED
2. Add the actual test fixture paths
3. Update the summary table
4. Flag any claim whose fixture FAILED — that's a thesis gap,
   not a test bug
