---
status: draft (Mgr-tier canvas; ENGAGE-NOW per Director disposition 2026-05-06 at gunbc#828 #issuecomment-4384615320)
authority parent: R3 Substrate Manager (#1739)
ratification: Director disposition 2026-05-06 directs Substrate Mgr to surface gap-test candidate per option (a)
roadmap row: docs/r3-program-plan.md §10.3 Q-Class-2-Chain-Break + §1.8 ledger row #61
authority docs:
  - docs/r3-program-plan.md §4.2 (Class 2 chain-break)
  - docs/r3-program-plan.md §1.8 ledger row #61 (DECLARED, RED until Q-LBP-R3-Closeability resolves)
  - docs/r3-program-plan.md §4.2 (two resolution paths (a)/(b))
  - docs/r3-program-plan.md §"Open questions" Q-Class-2-Chain-Break disposition
  - docs/audit/r3-debt-sweep-2026-05-06.md §3 (YELLOW chain rule)
  - docs/r3-structure.md §"Acceptance — `.dag` gates" (gap-test definition)
  - docs/r3-design-schedule-2026-05-06.md §1 S1
gates:
  - substrate_gap_function_valued_data_closed (#61)
---

# R3 Substrate S1 — Q-Class-2-Chain-Break gap-test surface canvas

## Purpose

Surface a candidate option-(a) gap-test for `substrate_gap_function_valued_data_closed`
(Class 2, ledger row #61) that **traces to GREEN within finite chain
without requiring T-Lens-Behavioral-Parity COMPLETE**. Per Director
disposition 2026-05-06 (gunbc#828 #issuecomment-4384615320): engage now;
recommend option (a) re-pick. If (a) structurally infeasible, escalate
(b) Q-LBP-R3-Closeability scope-calibration as load-bearing.

This is a **Mgr-tier canvas**, not a worker brief. Output is a
recommended gap-test shape + traceability argument to GREEN; worker
dispatch (if any) follows Director ratification of the candidate.

## Class 2 statement (per `r3-structure.md` §"Acceptance — `.dag` gates")

> `substrate_gap_function_valued_data_closed` — `Lens<C>` instance
> with function-typed payload executes through evaluator and produces
> `DimensionReport<C>` without Rust mediation. Closes via
> T-Lens-Application-Surface + T-E-P-Producer-Broadening.

The original chain — T-Lens-Application-Surface ← T-Lens-Behavioral-Parity
← Q-Lens-Behavioral-Parity-R3-Closeability (RED) — does not terminate
at GREEN within finite steps; per Refinement 3 YELLOW chain rule
(`r3-debt-sweep-2026-05-06.md` §3) Class 2 inherits RED.

## Candidate surface (option-(a) re-pick)

**Recommendation**: narrow the gap-test from "function-typed payload
in arbitrary `Lens<C>` instance" to "function-typed `data` declaration
consumed by an evaluator-executable representative". The narrowed test
exercises the substrate fact (function-valued data is first-class)
without requiring any specific lens to reach BEHAVIORALLY COMPLETE.

### Proposed gap-test shape

Authored as a `.dag` representative under `dsl/tests/` or similar:

1. **Function-valued data declaration** — top-level `data f: Int -> Int = ...`
   form using existing `func`/`Arrow` substrate (already landed; see
   `src/v3/std/types.dag` Arrow + `v3_compiler::dag::Callable`).
2. **Evaluator consumption** — `eval_*` path that takes the function-valued
   data as input, applies it to a representative argument, and produces
   a `Value` result. Target: existing E6-G0d constructor runtime
   execution path (Evaluator E1, dispatched 2026-05-06 to valiant-carp-10
   per #1767 #issuecomment-4385079490) extended to `Callable` payloads
   referenced through `data` rather than constructed inline.
3. **No Rust mediation predicate** — assertion that the evaluator's
   handling of the function-valued payload routes through `eval_call`
   / `eval_lens_apply` (or equivalent) using only substrate-facts
   produced by T-E-P-Producer-Broadening (descent-evidence side-table).

### Traceability to GREEN

- **Prerequisite 1**: T-E-P-Producer-Broadening Phase 1 (descent-evidence
  full coverage) — independently dispatchable, foundational, no LBP
  prereq. Already scheduled (S10).
- **Prerequisite 2**: Evaluator E6-G0d (constructor + `Callable`
  runtime execution) — DISPATCHED 2026-05-06; valiant-carp-10 owner.
- **No T-LBP dependency**: the gap-test does NOT assert "complexity
  lens reaches BEHAVIORALLY COMPLETE on this input". It asserts only
  "evaluator executes function-valued data path". Lens behavior is
  out of scope.

Both prerequisites are EVAL-3-side or substrate-side — no chain through
Q-LBP-R3-Closeability. **Trace to GREEN is finite**: T-E-P Phase 1
+ E6-G0d → gap-test executes → green.

## Ratification surface

This canvas is the Substrate-Mgr-side surface for Director ratification of:

- **Q1**: Is the narrowed gap-test (function-valued `data` + evaluator
  consumption, lens-behavior out of scope) acceptable as a Class 2
  closure under §1.8 row #61?
- **Q2**: If yes, ledger row #61 reframes from CHAIN-BREAK / RED →
  DECLARED / YELLOW (gated only on T-E-P + E6-G0d, both in flight).
  Substrate Mgr authors a worker brief on the gap-test representative
  for dispatch post-prerequisite.
- **Q3**: If no (i.e., Class 2 must demonstrate via `Lens<C>` shape
  specifically), then option (b) — Q-LBP-R3-Closeability scope-
  calibration becomes load-bearing. This canvas hands off to S2
  (T-LBP scope-calibration canvas).

## STOP-AND-ESCALATE

- **Director rules option-(a) infeasible**: hand off to S2 immediately;
  Class 2 closure becomes LBP-cascade-gated. Not a Substrate-side
  problem to solve.
- **T-E-P-Producer-Broadening Phase 1 produces no surface usable by
  function-valued data path**: the prerequisite chain itself has a
  gap; surface to Director as a substrate-extension question (does
  function-valued data require additional descent-evidence variants
  beyond current 8 `CallPattern`?). New variant addition follows P1
  procedure per `INVARIANTS.md#p1-modeling-faithfulness`.
- **E6-G0d worker-brief surface drifts** (e.g., Evaluator scope narrows
  to non-`Callable` constructors only): Class 2 closure must wait for
  separate `Callable`-runtime work; gap-test deferred until that
  surface lands. Coordinate with Evaluator Mgr (#1743).

## Authority audit receipt

1. **Substrate exists?** Function-valued `data` substrate exists:
   `Arrow` (in `src/v3/std/types.dag` + `dag.rs`), `Callable`
   (`dag.rs` evaluator path), `data` declarations (lowerer evaluates
   to `Value` at lower time per memory note "Data declaration wiring
   2026-02-25"). No new top-level carrier required for the gap-test;
   re-uses landed surfaces.
2. **Existing brief?** None. `r3-pr-e6-g0d-constructor-runtime-execution-worker.md`
   is the upstream Evaluator brief; this canvas is the Substrate-side
   gap-test specification that consumes E6-G0d's runtime extension.
3. **Design-doc match?** `r3-program-plan.md` §10.3 Q-Class-2-Chain-Break
   names option (a) re-pick + option (b) LBP escalation. Director
   disposition 2026-05-06 selects (a). Canvas surfaces (a) candidate.
4. **Citations live?** `r3-program-plan.md` §4.2 + §1.8 ledger
   row #61 + §"Open questions" Q-Class-2-Chain-Break, plus
   `r3-structure.md` §"Acceptance — `.dag` gates" verified at
   HEAD 2026-05-06. Section-anchor / quote form per citation
   discipline; no bare line numbers.
5. **Carrier dissolves the bridge?** Gap-test is structural
   demonstration, not a carrier. The "bridge" being dissolved is the
   YELLOW chain-rule violation in §1.8 row #61: re-picking the
   gap-test removes the invalid YELLOW status by replacing the
   non-finite chain with a finite one.

## Provenance

Drafted 2026-05-06 per Director disposition (gunbc#828
#issuecomment-4384615320) directing Substrate Mgr to surface option-(a)
candidate. Dispatched as part of R3 design schedule (PR #1810 §1 S1).
Awaits Director Q1/Q2/Q3 ratification before worker brief authoring
proceeds.
