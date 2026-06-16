# Design: v2 Runtime Architecture — eval for real programs, effects/IO, dispatch (TODO A.1)

> **Status: DESIGN — map, not territory.** The runtime half pairing COMPREP's producer half
> (`design-computation-representation.md`): COMPREP makes function bodies exist; this design
> says how they execute. Gates compute-fabric Brief D (`design-compute-dispatch.md` §3 is
> this design's consumer-side input spec — answered in §5 below).

## 1. Problem

`05_eval` evaluates fixture subgraphs through an `InterpretationAlgebra` whose Transform arm
is a `call_primitive` slot. Real programs need: callee dispatch over growing environments,
defined error semantics, a run-loop, and — the load-bearing piece — an **effects/IO
boundary**: how eval performs effects, who handles them, and how purity is *enforced* rather
than assumed.

## 2. What already exists (M9 DFS)

| Piece | Where | Role |
|---|---|---|
| Option-C split: abstract runtime carriers in `std/`, concrete bundles in `extdeps/runtimes/` | `src/v2/std/runtime.dag` (`RuntimeValue` = Primitive/Aggregate/Reference/Closure/Unit; `EvaluationEnvironment`; `RuntimeOutcome`; `ValueRepresentationModel`), `extdeps/runtimes/v2_evaluator.dag` (`V4EvaluatorRuntime`, wave-1 effect-signature/boundary nodes) | the value/environment substrate and the layer pattern this design extends — same split the compute-provider design reuses |
| `InterpretationAlgebra` + `EvalFoldState`/`EvalProgress` + cache authority | `src/v2/compiler/05_eval.dag` | eval is already a fold with typed progress; this design fills its arms, it does not replace the fold |
| Effect vocabulary | `std/effects.dag` (`IsIdempotent \| IsBreaking`), `model_core.dag` (`EffectSignature`, `PrimitiveOperationRef`), `std/determinism.dag` (orthogonal determinism axis) | declared effect facts the boundary enforces |
| Handler-shape precedent | INVARIANTS P1 kernel-calculus note: lenses are "algebraic-effect-handler-shaped (Plotkin & Pretnar 2009)" | the external grounding for §4 |
| Host-process boundary discipline (a)–(e); `host_run.dag` receipts | INVARIANTS P2; `std/host_run.dag` | the only door to the host, reused not duplicated |
| Scheduler frontier (#4566), executor (#4603) | `workflow/scheduler.dag` | the run-loop's work source (§4.4) |
| THESIS Tier-2 ("no partial functions in the runtime"), P4 bounded forward execution | THESIS/INVARIANTS | the error-semantics answer (§4.2) |

**Substrate target (P1):** effect-request/handler carriers extend `std/runtime.dag`;
concrete handler bundles are `extdeps/runtimes/*` instances; `05_eval` consumes through the
existing algebra. No connective/behavior change.

## 3. The architecture in one paragraph

A **pure total core** evaluated by the existing fold, with **effects reified as typed
requests** that eval *yields* to declared **handlers** at the boundary — never performs
inline. Environments are frame-chained binding maps; values are immutable and acyclic;
"mutation" is the returned-modified-resource pattern THESIS already commits to. Everything
host-shaped (IO, process, dispatch-to-provider) is a handler bundle in `extdeps/runtimes/`;
swapping handler bundles is how the same program runs under test (pure/mock handlers —
mocking is dependency-injection by construction), locally (host handlers), or on the fabric
(the dispatcher handler = Brief D's bridge).

## 4. Commitments

### 4.1 Environment and store growth
Frames chain (`EvaluationEnvironment` + closure capture, both landed); binding maps follow
the Map-unification representation. The store is an **arena per run**: allocation is
append-only along bounded forward execution (P4 — values are acyclic, so no cycles to
collect), reclamation is whole-run in wave 1. No GC design; if region-level reclamation is
ever wanted, it derives from ownership-lens facts (a later wave with its own consumer), not
from a collector.

### 4.2 Error/panic semantics: there are no panics
The modeled fragment is total (Tier 2): every partial operation is proven safe or returns a
typed `Outcome`/`RuntimeOutcome`. So the runtime has exactly two failure categories, never
collapsed: **modeled refusal** (typed, located, part of the program's semantics) and **host
fault** (interpreter defect, resource exhaustion) — the latter is a host receipt under the
P2 host-boundary rules (setup-failure ≠ logical exit), attributed to the runtime bundle,
never reinterpreted as a program value. "Panic semantics" is therefore a non-question by
construction; the design work is keeping the two categories separated at every boundary.

### 4.3 The effects/IO boundary (the handler model)
- An **effect request** is a typed carrier: the operation ref, its declared
  `EffectSignature`, the dimensioned arguments, and a continuation slot — minted *only* by
  a constructor that requires the signature fact. A function whose Arrow declares no
  effects has no path to mint one: **purity is enforced structurally at construction, and
  fail-closed at eval** (an undeclared request surfacing mid-run is a defect diagnostic
  naming the operation, never executed).
- A **handler** is a declared inhabitant per effect kind in a runtime bundle: the pure/test
  bundle answers from declared fixtures (and records the request sequence as a receipt —
  the test surface for effectful claims); the host bundle routes through the *existing*
  `host_run`/ExecuteCommand discipline (one host door, P2 (a)–(e) inherited verbatim).
- Wave 1 is deliberately flat: **one handler per effect kind per run**, bound at run start
  from the selected runtime bundle. No dynamic handler stacks, no handler shadowing —
  that's expressive power with no consumer; revisit only with one (E-10).
- Determinism: the core is deterministic; effects are the only nondeterminism source,
  classified on the `determinism.dag` axis; a run's receipt sequence is its replay log.

### 4.4 Run-loop, scheduler, executor → dispatcher
The run-loop is a bounded fold over the scheduler's `RunnableFrontier` (#4566): take ready
work, evaluate (pure core + yielded requests), append receipts, advance. Sequential in
wave 1. **Dispatch is an effect**: "run this work on that provider" is one more request
kind, and Brief D's `OaaS_v2` bridge is *one handler implementation* of it — which is the
unification that makes D small: scheduler picks what, executor batches, the dispatch
handler fires it and returns `host_run`-family receipts. Parallel frontier evaluation (the
PRT lane) becomes safe *because* of this section — pure core plus explicit effect requests
is exactly the precondition the THESIS parallelism-by-default claim needs — but it is a
separate lane, not designed here.

## 5. Answers to D's input spec (`design-compute-dispatch.md` §3)
1. Effects/IO enforcement = §4.3 (structural minting + fail-closed surfacing + handler
   receipts). 2. Preemption = the dispatch handler's contract: a `Preemptible` placement may
   return a typed preempted-receipt; re-execution admissibility was discharged at selection
   (C) and is re-checked by the handler before re-fire. 3. Receipts = `host_run` family,
   extended (if needed) in `std/host_run.dag` only. 4. Bounded waiting = the run-loop waits
   only inside a handler with a declared bound (timeout fact on the request); expiry is a
   typed outcome, not a hang. 5. Re-selection = a declared policy step that re-enters C
   excluding the offender, count-bounded, receipt-leaving — the run-loop has no retry
   primitive of its own.

## 6. Consumers and minimal slice (E-10)
Co-keystone with COMPREP wave 1: its claim (source-ingested `add` through eval with real
callee dispatch) exercises §4.1/§4.2 with **zero effects** — pure-core first. The runtime
slice proper adds the boundary: one effectful workload under the **pure/test handler
bundle** — green: declared write-effect request handled, receipt sequence asserted; red
(discriminating): the same operation invoked from an effect-free signature is
unconstructable/fails closed naming the operation; red: an unhandled effect kind refuses at
run start (no handler bound), not mid-run. All `--claim-run`, no host IO needed for the
slice — the host bundle and the dispatch handler are wave 2/3, each with its own claim.

## 7. Open questions — escalate, don't improvise
- **Q-R1 — effect-kind vocabulary wave 1.** Propose: `ReadResource`/`WriteResource` (the
  effects.dag shapes), `ExecuteCommand` (existing discipline), `Dispatch` (D). Operator
  trims; each kind ships with its handler contract.
- **Q-R2 — continuation representation** in the request carrier (the run-loop resumes the
  fold after a handled request): wave 1 can be handler-returns-value-then-fold-continues
  (no first-class continuations); confirm that restriction — it keeps the calculus inside
  the kernel grounding.
- **Q-R3 — runtime-bundle selection surface**: per-run argument vs per-claim declaration.
  Recommend per-claim (tests declare their bundle, the discriminating-red stays cheap).

## 8. Non-goals
GC/regions, parallel runtime (PRT lane), JIT/performance, handler stacks/shadowing,
first-class continuations, and any second host-process door.
