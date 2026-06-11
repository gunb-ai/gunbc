# Design: Interprocedural Cost (S3) — Callee-Cost Consult for the Complexity Gate

> **Status: DESIGN — 2026-06-11 (swift-bat-315).** Third stage of the complexity-gate
> ladder: S1 landed the real-body exemplar + budget roster (#4668/#4669, design
> `design-lens-subject-supply.md`), S2 landed the fail-closed roster completeness gate
> (#4673). This designs the remaining *soundness* gap: a call site currently costs O(1)
> regardless of what the callee does. No code in this doc. Substrate-gated: activation
> rides the COMPREP callee-dispatch wave (see §6).

## 1. The problem

`base_cost_for_behavior(Transform) = unit_cost()` (`src/v4/lens/cost.dag:206`). The cost
fold charges a call site one unit plus the fold of its *argument* children; the callee's
body is never consulted. Consequence: any superlinear body hidden behind a helper call
reads as constant at every caller. The S1/S2 gate is honest only because today's subjects
(COMPREP wave-1 `add`) are callee-free — `05_eval.dag` itself documents "there is no
user-function callee path because there are no callees." The moment COMPREP's
callee-dispatch wave lands, a budget gate without callee consult would pass O(n²)-via-helper
as O(1): the gate would go from incomplete to *unsound*. S3 must therefore land (at least
its fail-closed phase) no later than the first roster subject that contains a call.

## 2. Shape: a summary map over the call graph, composed OUTSIDE cost.dag

`cost.dag` carries an operator-STOP header (additive consumers only). S3 does not edit the
node-local fold. Instead it is a new additive lens module (working name
`src/v4/lens/cost_call_graph.dag`) that composes the existing pieces:

1. **Summaries.** For each declaration reachable from the subject, compute
   `cost_lens(body)` — the existing node-local fold — yielding a *summary*:
   `binding_id → SymbolicCost`. Keying is by **binding_id** (#4581, COMPREP Q-C1's
   ratified callee channel): the same identity the resolver writes and eval dispatches on.
   Never by name string (reflection ban; same rule as the budget roster's
   no-name-strings interface).
2. **Call-graph extraction.** Edges come from resolved Transform nodes' callee references
   — the canonical accessor for "callee edge of a Transform" that the COMPREP dispatch
   wave lands (today's `eval_transform_callee_edge` in `05_eval.dag` is the 🟡-marked
   interim; S3 consumes the canonical accessor when it exists, never a local edge-walker —
   INVARIANTS "second path").
3. **Substitution.** Interprocedural cost of a node = the node-local fold with each
   Transform's `unit_cost()` base **sequentially composed with the callee's summary**
   (argument folding unchanged). Composition uses the existing `symbolic_sequential` /
   `symbolic_product` algebra — no new cost operators.
4. **Projection + gate unchanged.** The result is still a `SymbolicCost`; the existing
   `asymptotic_projection` → `complexity_bound_dominates` path consumes it. Budget rows,
   roster, family gate (S2) are untouched as interfaces — only the *computed* side gets
   sharper. `ClassUnknown` remains the fail-closed top.

The substitution in (3) must not duplicate the fold. Two legal placements, decided at
implementation time by whichever avoids a second fold path: (a) a parameterized fold —
the node-local `symbolic_cost_fold` gains a *consult* hook defaulting to "unit" (an edit
to cost.dag ⇒ operator-STOP escalation, but a one-point hook, not a second walker); or
(b) a wrapper lens that runs the canonical fold over a *call-expanded view*. Preference:
(a), precisely because it keeps ONE fold; it is the smaller honest change and the
escalation is the point — the operator sees the single hook rather than a shadow fold
growing next to the canonical one.

## 3. Recursion: lowered away before the fold — cycles are a defect backstop, not a semantics

The substrate has no general recursion. "Recursive Syntax Is Sugar" (INVARIANTS;
DB-9 for the mutual case): recursive surface forms lower through `CallPattern`
classification (`dsl/std/computation.dag:192` — every pattern maps to a primitive,
no unknown category) onto the closed iteration primitives (`dsl/std/iteration.dag:1`
— descend / fold / repeat are the ONLY loop mechanism), with termination carried
structurally by descent facts (`DescentEvidence` lattice, `dsl/std/termination.dag`).
A recursive form that cannot classify does not lower and is rejected at the boundary
(INVARIANTS "lower with an explicit bound or reject") — the cost lens never sees it.

Consequence for S3: a **well-formed lowered call graph is acyclic by construction**,
and the cost of recursive code is not an S3 problem at all. Recursion reaches the fold
as `Loop` nodes, and the node-local fold already prices those (`base_cost_for_loop`
consuming `loop_bound_witness_for_node`: bound × per-iteration body cost). The consult
only ever follows acyclic call edges; S3 adds no recursion machinery.

The SCC/self-edge clause survives only as an **integrity backstop**: if summary
computation encounters a cycle in the lowered graph, the lowering invariant was
violated (compiler defect, or an unlowered recursive form leaked past the boundary).
Summarize to `unknown_cost(...)` → `ClassUnknown` → red. Never silently green — but a
red here is a defect report against the lowering, not a budget verdict the function's
author can act on. No fixpoint/widening machinery is warranted for a case that is
by-construction a defect.

**Rejected (was "P2" in an earlier draft): witness-refined SCC summaries at the lens
level.** Consuming termination witnesses inside the cost lens to bound call cycles
would re-derive, at analysis time, exactly the fact the CallPattern lowering already
establishes structurally — a second path to the same authority (INVARIANTS second-path
ban). The witness obligation lives in the lowering (descent facts), and the cost lens
inherits it for free through the Loop treatment.

## 4. What this is NOT

- **Not a second cost semantics.** One fold, one algebra; S3 adds consult, not carriers.
  Any proposal introducing a parallel "call cost" type next to `SymbolicCost` is the
  anti-pattern.
- **Not name-keyed.** Summary keys and call edges are binding_id-channel references. A
  `summaries["function_name"]` shape anywhere is the banned reflection pattern re-entering
  through the side door.
- **Not eager whole-program analysis.** Summaries are computed for declarations reachable
  from roster subjects, on demand, inside the witness run — same execution boundary as
  every other `--claim-run` gate row. No persisted summary cache in phase 1 (a cache is a
  parallel ledger until proven necessary by wall-time).

## 5. Interfaces touched, and the escalation surface

| Piece | Change | Authority bar |
|---|---|---|
| `lens/cost_call_graph.dag` (new) | summaries, SCC handling, substitution | additive — normal PR |
| `lens/cost.dag` consult hook (§2a) | one parameterized point in the fold | **operator-STOP — escalate with this doc before touching** |
| canonical Transform-callee accessor | consumed, not defined here | lands with COMPREP callee-dispatch wave; S3 blocks on it |
| budget roster / family gate (S1/S2) | none — computed side sharpens | — |
| `std/cardinality.dag` loop-bound witnesses | already consumed by the node-local Loop fold — recursion arrives lowered as `Loop`, no new consumption | — |

## 6. Activation ladder and perturb obligations

S3 is **substrate-gated**: until COMPREP grammar+eval admit a subject *containing a call*,
there is nothing for the consult to consult. The ladder:

1. **Now (pre-callee-wave):** this design ratified; no implementation. The roster's
   wave-1 subjects are callee-free and the gate is sound without S3.
2. **COMPREP callee-dispatch wave lands:** S3 must land **in the same wave or before
   the first call-containing roster row** — tracked as a blocking edge, not a follow-up
   hope. The first call-containing budget row's witness must include the S3 semantic reds.
3. **Recursion appears in subjects:** nothing attaches to S3 — recursive surface forms
   arrive at the fold already lowered to `Loop` (the recursion-sugar wave, §3), priced
   by the existing Loop treatment. The gate's reach over recursion is gated on the
   lowering wave, not on more lens work.

Perturb obligations for the first S3 witness, beyond the standard `--perturb-check`:

- **Consult red:** a subject whose body calls a deliberately superlinear helper, with a
  constant declared budget → must fail dominance (proves the consult, not just the
  plumbing — the S3 analogue of S1's tightened-budget red).
- **Cycle red:** a hand-built cyclic call-graph fixture (not legal surface code — legal
  recursion lowers to `Loop`, §3) with any finite budget → must fail (proves the
  integrity backstop closes; a lowering defect cannot read as green).
- **Missing-summary red:** a call edge whose callee summary is absent (unresolvable
  binding) → `ClassUnknown` → fail (proves the absent-case is closed, mirroring the
  roster's missing-budget rule).
