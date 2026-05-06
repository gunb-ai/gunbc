---
status: draft (worker brief; dispatch-blocked on T-E-P Phase 1 landing + E6-G0d landing per S1 canvas trace-to-GREEN)
authority parent: R3 Substrate Manager (#1739)
ratification: Director ratified S1 Q1+Q2 at gunbc#828 inbox response 2026-05-06 (zesty-bear-812; "Narrowed gap-test (function-valued data + evaluator consumption; lens-behavior out of scope) is correct as Class 2 closure"); ledger row #61 reframes RED → DECLARED YELLOW
roadmap row: docs/r3-program-plan.md §1.8 ledger row #61 (substrate_gap_function_valued_data_closed)
authority docs:
  - docs/briefs/r3-substrate-s1-q-class-2-chain-break-gap-test-canvas.md (canvas this brief consumes)
  - docs/r3-program-plan.md §10.3 Q-Class-2-Chain-Break (Director disposition + ratification record)
  - docs/r3-structure.md §"Acceptance — `.dag` gates" (Class 2 gap-test definition)
  - docs/briefs/r3-pr-e6-g0d-constructor-runtime-execution-worker.md (Evaluator E1 prerequisite)
  - docs/briefs/r3-t-e-p-producer-broadening-worker.md (T-E-P Phase 1 prerequisite)
gates:
  - substrate_gap_function_valued_data_closed (#61)
---

# R3 Substrate S1 — Class 2 gap-test representative worker brief

## Context

Director ratified the option-(a) narrowed gap-test surface from S1
canvas: function-valued `data` declaration consumed by evaluator
(via E6-G0d / `Callable` runtime path) producing a `Value` result,
asserting "evaluator executes function-valued data path" without
requiring any specific lens to reach BEHAVIORALLY COMPLETE.

This brief lands the **representative `.dag` test program + assertion
predicate** that closes ledger row #61. The substrate fact under
test is "function-valued data is first-class"; the test is
deliberately decoupled from lens-behavior axes.

**Dispatch is gated** on two prerequisite landings:
1. **T-E-P-Producer-Broadening Phase 1** (`per_call_descent_evidence`
   full coverage) — descent-evidence side-table is the substrate the
   evaluator path may consume for the representative.
2. **Evaluator E6-G0d** (`Callable` runtime execution in
   `eval_transform_node` / `eval_call`) — the runtime path the test
   exercises. Currently DISPATCHED to valiant-carp-10 via #1767.

Worker confirms both prerequisites at HEAD before proceeding.

## Slice

### Phase 1 — Representative `.dag` source

1. Author `dsl/tests/r3/class_2_function_valued_data.dag` (or under
   the canonical `.dag` test fixture root; worker greps for the
   project's representative-test convention at dispatch).

2. Test program shape:

   ```
   // Function-valued data — the substrate fact under test.
   data add_one: Int -> Int = fn(x: Int) -> Int { x + 1 }

   // Consume function-valued data via apply.
   func test_function_valued_data() -> Int {
     add_one(41)
   }
   ```

   Exact syntax aligns to current DSL grammar (worker greps existing
   `func` + `data` + `Arrow` test fixtures for canonical authoring).
   The decisive structural property: `add_one` is declared as
   **top-level `data` with `Arrow` type and function body**, NOT
   as a `func` declaration. This is what makes it function-valued
   data rather than a callable-by-name function.

3. Expected runtime behavior: `test_function_valued_data()` evaluates
   to `42` (`Int`-valued).

### Phase 2 — Evaluator-path assertion

The closure predicate for ledger row #61 is "evaluator executes
function-valued data path producing `Value` without Rust mediation."
The assertion has two arms:

1. **Positive arm — runtime evaluation produces correct result.**
   Run the representative through E6-G0d evaluator path; assert
   output `Value` equals `Value::Int(42)`.

2. **Routing arm — evaluator dispatches through `eval_call` /
   `eval_lens_apply` (or equivalent) using only descent-evidence
   substrate facts.** Assert no Rust-side fallback is invoked
   (e.g., no `unimplemented!` / `todo!` / Rust-mediated identity
   passthrough). The "no Rust mediation" predicate is structural:
   the runtime path traces through DSL-substrate-defined operations
   only.

Worker authors the assertion in the project's canonical test harness
(likely `cargo test -p gunbc-codegen` or `cargo test -p v2-compiler-tests`
depending on where Class-2 representative tests slot; greps at
dispatch).

### Phase 3 — Cementing test discipline

Per `r3-structure.md` Class 2 closure definition: gap-test must
demonstrate the substrate fact under named conditions. Cementing
discipline requires:

1. Test runs deterministically — no timing/network/random
   dependencies. The representative is closed-form integer arithmetic.
2. Test is idempotent — reruns produce same result.
3. Test is in `cargo test --workspace` default invocation (not
   `--ignored`); CI runs it on every push.

If test infrastructure currently routes Class 2 representatives
through a separate harness (e.g., `cargo test -p v2-compiler-tests`),
worker preserves that routing — do NOT reframe test infrastructure
as part of this slice.

## Acceptance

- `dsl/tests/r3/class_2_function_valued_data.dag` (or canonical
  equivalent path) lands with the representative program.
- Evaluator-path assertion lands in canonical test harness; positive
  + routing arms both pass.
- `cargo test --workspace --exclude v2-compiler-tests` green;
  representative test included in default invocation.
- §1.8 ledger row #61 status moves DECLARED YELLOW → CONSUMER_LANDED
  (representative + assertion landed) → PASSING (both arms green
  + cementing discipline verified).
- ROADMAP / structure receipt: Class 2 substrate-gap closes per the
  narrowed gap-test surface; no T-LBP load-bearing for Class 2.
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo fmt --all --check` clean.

## STOP-AND-ESCALATE

- **T-E-P Phase 1 lands but `per_call_descent_evidence` does not
  produce the substrate-fact the evaluator path consumes for
  function-valued data**: the prerequisite chain has a gap.
  STOP — surface to Substrate Mgr (#1739). Phase 1 may need
  additional `CallPattern` / `SubValueRelation` variants (P1
  procedure for substrate-fact introduction).
- **E6-G0d landing produces `Callable` runtime path but routes
  through a Rust-mediated fallback** (e.g., Rust-side identity
  passthrough for function-valued data because DSL-side
  dispatch is incomplete): routing arm fails. STOP — coordinate
  with Evaluator Mgr (#1743) on E6-G0d completion scope. The
  routing arm is load-bearing; do not weaken it to "as long as
  result is correct".
- **Representative test passes positive arm but routing arm
  fails because evaluator dispatches through a `lower_*` /
  bridge function**: the bridge is the substrate gap the test
  is supposed to surface. STOP — do not bridge the bridge.
  Coordinate with Substrate Mgr (#1739) + Evaluator Mgr (#1743)
  on the dispatch-path bridge as a substrate finding.
- **Test must be marked `#[ignore]` or routed to a separate
  harness to pass**: STOP — Class 2 closure requires green-by-
  default. If test is fundamentally not-green-by-default, the
  closure predicate is not satisfied. Surface as a finding,
  do not declare row #61 PASSING.
- **DSL grammar rejects the `data add_one: Int -> Int = fn(...)`
  shape**: the substrate fact under test is "function-valued data
  is first-class". If grammar does not support the shape, the
  substrate gap is upstream of evaluator-side. STOP — coordinate
  with Substrate Mgr; this becomes a parser-grammar finding
  (Class 1 cross-pollination).

## Authority audit receipt

1. **Substrate exists?** `Arrow` type + `Callable` runtime substrate
   exist (per memory notes + per existing `func` / lambda fixtures);
   worker re-greps `dsl/std/types.dag` + `src/v3/compiler/src/dag.rs`
   at dispatch to confirm. Top-level `data X: Arrow = fn(...)` shape
   may or may not be exercised by existing fixtures — worker
   confirms representative-shape-novelty vs precedent at dispatch.
2. **Existing brief?** S1 canvas (`r3-substrate-s1-q-class-2-chain-break-gap-test-canvas.md`)
   is the upstream design canvas; this brief is the worker dispatch
   per Director Q1+Q2 ratification. No competing brief.
3. **Design-doc match?** Director ratification at gunbc#828 inbox
   response 2026-05-06 anchors the Q1 narrowing decision; this brief
   structurally matches the ratified scope (lens-behavior out of scope).
4. **Citations live?** S1 canvas references verified at HEAD 2026-05-06.
   E6-G0d brief #1784 / T-E-P brief
   `r3-t-e-p-producer-broadening-worker.md` referenced for
   prerequisite definitions; worker re-verifies at dispatch.
5. **Carrier dissolves the bridge?** N/A — this brief lands a
   representative + assertion, not a substrate carrier. The "bridge"
   under test is "function-valued data path executes through evaluator
   without Rust mediation"; the test surfaces the bridge if any
   currently exists, allowing closure or escalation per
   STOP-AND-ESCALATE.

## Provenance

Drafted 2026-05-06 per Director Q1+Q2 ratification of S1 canvas
(gunbc#828 inbox response 2026-05-06; zesty-bear-812). Dispatch
gated on T-E-P Phase 1 + E6-G0d both landing. Worker pin TBD;
representative test authoring is small (~M sized, likely 1 PR).
