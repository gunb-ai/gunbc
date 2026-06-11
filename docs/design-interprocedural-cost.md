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

## 3. Recursion and cycles: fail closed first, refine by witness later

The call graph of real programs has cycles. Phase ordering:

- **S3-P1 (fail-closed):** compute summaries in topological order over the call DAG's
  strongly-connected components. Any SCC of size > 1, and any self-edge, summarizes to
  `unknown_cost(...)` → `ClassUnknown` → dominance fails against every finite budget.
  Sound, never silently green; recursive code simply cannot pass a finite budget yet.
- **S3-P2 (witness-refined):** a recursive declaration may carry a termination witness
  (`std/cardinality.dag` already models `TerminationProof` / `RequiresTerminationProof` /
  `ProvenTermination`; `base_cost_for_loop` already consumes `loop_bound_witness_for_node`
  the same way). A proven decreasing measure with a size bound lets the SCC summarize as
  the measure's bound times the body's per-iteration cost — exactly the existing Loop
  treatment, lifted from structural loops to recursive calls. No witness → P1 behavior.
- **No fixpoint iteration.** The `SymbolicCost` lattice has unbounded ascending chains
  (polynomial degrees); a Kleene fixpoint needs widening, and widening-to-top is
  indistinguishable from P1's answer at far higher complexity. P1+P2 give the same
  precision frontier with a witness obligation instead of an analysis heuristic — which is
  the substrate's idiom (witnesses over cleverness).

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
| `std/cardinality.dag` termination witnesses | consumed in P2 | additive consumers |

## 6. Activation ladder and perturb obligations

S3 is **substrate-gated**: until COMPREP grammar+eval admit a subject *containing a call*,
there is nothing for the consult to consult. The ladder:

1. **Now (pre-callee-wave):** this design ratified; no implementation. The roster's
   wave-1 subjects are callee-free and the gate is sound without S3.
2. **COMPREP callee-dispatch wave lands:** S3-P1 must land **in the same wave or before
   the first call-containing roster row** — tracked as a blocking edge, not a follow-up
   hope. The first call-containing budget row's witness must include the S3 semantic reds.
3. **Recursion appears in subjects:** S3-P2, witness-by-witness.

Perturb obligations for the first S3-P1 witness, beyond the standard `--perturb-check`:

- **Consult red:** a subject whose body calls a deliberately superlinear helper, with a
  constant declared budget → must fail dominance (proves the consult, not just the
  plumbing — the S3 analogue of S1's tightened-budget red).
- **Cycle red:** a deliberately self-recursive subject with any finite budget → must fail
  (proves SCC fail-closure).
- **Missing-summary red:** a call edge whose callee summary is absent (unresolvable
  binding) → `ClassUnknown` → fail (proves the absent-case is closed, mirroring the
  roster's missing-budget rule).
