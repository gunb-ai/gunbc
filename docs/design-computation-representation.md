# Sizing: Computation Representation — Function Bodies in the v2 Pipeline (COMPREP)

> **Status: SIZING — measured audit + wave decomposition, 2026-06-09.** Proposed as a
> first-class hard-problems node (sibling of STAGE-ADOPTION, feeder of SELFHOST), not a
> termination-checker footnote. This is the layer-3 gap from the call-graph audit
> (`design-termination-checker.md` §4.3 prerequisite), sized on its own because it gates more
> than the checker. No code from this doc; wave 1 is the designable slice.

## 1. The measured state (receipts, this clone)

The THESIS substrate claim — *"Computation is five L1 behaviors… Transform holds a
FunctionRef to an Arrow declaration; the Arrow's body is a sub-DAG of L1 behaviors"* — is
landed as **consumable shapes with zero producers**:

| Layer | Measured state |
|---|---|
| Carriers | `Behavior = Value\|Transform\|Branch\|Loop\|Bind` landed in `std/node.dag`; **no `FunctionRef` carrier exists**; `canonical_tag_transform` exists for hashing only |
| Producers | **none** — `02_parse` constructs no `ComputationNode`; `03_normalize`/`05_eval` only *match* the kind; resolve handles `Arrow` only as the *type connective* (param-scope resolution of signatures) |
| The keystone, actually | `rust_mvp1_fixture_emitted_add_fn` (`extdeps/languages/rust.dag:1035`) is `TypeNode Arrow` + three positional `i32` edges — **a signature**. The fixture's own header says it: "full source-ingest compile remains blocked on ingest staging" |
| Consequence for the ladder receipts | T0 / RTADD #4544 / T1 #4545 are **signature/type-expr-tier** receipts. Real and discriminating at that tier — but "emit(add)" language oversells until bodies exist. (v2 emits real bodies; v2's ladder so far proves type-expr translation.) |
| Eval | `05_eval` (1,936 lines) has the `InterpretationAlgebra` behavior-dispatch skeleton, but the Transform arm is `transform.call_primitive(node, args, environment)` (`05_eval.dag:1204`) — a **primitive-interpreter slot**; there is no user-function callee path because there are no callees |
| Infer | `InferredFacts { grounding, descent }` — the descent witness channel exists (the termination design extends it); body-shaped facts absent |

So COMPREP = **the missing producer half of the pipeline**: parse bodies → Behavior trees,
resolve them, infer over them, evaluate them, and (later) translate them. It is the largest
unbuilt producer in v2.

## 2. What it gates (why it's a node, not a footnote)

- **SELFHOST facets 1–2** — `compiler.dag` cannot be compiled *by v2* without function
  bodies; COMPREP is a co-equal gate with the emit ladder for the fixed point (the dep
  graph's `SPINE → SELFHOST` edge implicitly assumes it; it should be explicit:
  `COMPREP → SELFHOST`).
- **Termination checker** — call-graph edges (layer 3 of the audit) exist only over bodies.
- **Emit ladder T4+ (value-expr tiers)** and **BIDIR** — bodies must emit/ingest through the
  same grammar-relation rows (design #3); building T4+ against fixture signatures defers
  the risk instead of exercising it.
- **05_eval growth past `call_primitive`**, **STAGE-ADOPTION axis A** (a fold needs a real
  body corpus), and the **impossible-bug demos** (they demo on programs, not signatures).

## 3. Wave decomposition (keystone-first, each wave consumer-gated per E-10)

**Wave 0 — carriers + identity (days).** The callee reference on `Transform` and the body
attachment on the declaration. Recommended shapes, applying the operator's fewer-variants
preference: **no new `FunctionRef` carrier** — the callee edge targets the declaration
through the **`binding_id` channel** (#4581), i.e. the same T-9 rider the call-graph audit
already routes (one write, one seam; a `FunctionRef` record would be a second identity
vocabulary). Body attachment: an **`Arrow.body` edge** — INVARIANTS already names
`Arrow.body` as the single authority for external realization (E-9/DB-14); internal
realization (a Behavior sub-DAG) is the same edge with a body target, which keeps one
authority for "what realizes this declaration." The Node-field moment, equality
participation, and landing order are owned by
[`design-node-identity-channels.md`](design-node-identity-channels.md) — this wave lands
through that table (#4581 → T-9 rider → COMPREP refs ride the channel), not independently.

**Wave 1 — the keystone body, source-ingested (1–2 weeks; the designable slice).** Parse
`add`'s *body* from real source: grammar productions for the minimal expression subset
(param reference, one call/operator application), Behavior-tree construction
(`Transform`/`Value`), body-scope resolution (params → `binding_id`), infer admission, and
eval executing it through a real callee path (retiring `call_primitive` as the only arm).
**Green criterion:** source-ingested `add` (not the fixture) flows parse → resolve → infer
→ eval and `run_test_claim` compares the executed result to expected, by execution; the
discriminating red is a body perturbation (swap the operands' bindings) flipping the claim.
Risk concentration — and therefore what the slice must exercise: body-scope resolve and the
eval callee dispatch. This wave alone is roughly PROV-sized.

**Wave 2 — the remaining behaviors (weeks).** `Bind` (let), `Branch` (match/if), `Loop`
(bounded — wiring the already-landed `LoopBound`/`termination` slot in
`std/cardinality.dag`), each with parse + resolve + eval + claims. Bounded per behavior;
`Branch`/`Loop` semantics and their discriminating claims are the real work.

**Wave 3 — bodies through translate/emit.** Joins the emit ladder's value-expr tiers;
bodies emit via grammar-relation rows per `design-bidirectional-coercion.md` §6 (rows, not
render closures), so T7 ingest of bodies is the forward interpreter, not new work. Not
independently sizable — ladder-coupled by design.

**Wave 4 — compiler-self breadth (the long tail).** Enough expression-surface coverage that
v2's own modules are representable — the SELFHOST/STAGE-ADOPTION feeder. Dominated by
surface breadth, not new mechanism; the FRONTEND-SUSTAINABILITY census (adhoc-9ad4147d) is
the natural scoping input. Size emerges from waves 1–2 velocity; do not estimate it now.

## 4. Dependency edges (for the dep graph)

In: **#4581 binding_id** (wave 0 identity), **T-9 rider** (same write), design #3
obligations (wave 3). PROV/T-8 is orthogonal but shares the Node-field moment (wave 0
coordination). Out: **SELFHOST** (co-equal gate with SPINE), termination-checker layer 3,
emit-ladder T4+, 05_eval, STAGE-ADOPTION, impossible-bug demos.

## 5. Recommendations beyond the build

1. **Honest relabeling (cheap, do now):** the dep graph's T0/RTADD/T1 rows should say
   *type-expr/signature-tier* explicitly — same honest-labeling move the round-trip claims
   already make ("not bit-identical unless claimed"). Nothing regresses; the language stops
   overselling.
2. **Sequencing:** wave 0 rides #4581 (with the T-9 rider); wave 1 starts immediately after
   and is the next keystone-shaped GO candidate once the coercion arc's follow-ups settle.
   The termination checker's slice step 2 and SELFHOST stage B both queue behind wave 1's
   producer, which is precisely why COMPREP deserves its own node: two goal lanes currently
   read as gated on other things are actually gated on this.
3. **THESIS hygiene:** the "Transform holds a FunctionRef" sentence should either be updated
   to the binding_id-channel shape when wave 0 lands, or wave 0 should consciously implement
   FunctionRef-as-described — either is fine; carrying the drift silently is not (P1
   documentation-describes-live-state).

## 6. Open questions — escalate, don't improvise

- **Q-C1 — callee-reference shape.** Recommended above: binding_id-channel edge, no new
  `FunctionRef` carrier (fewer variants). Confirm, since it amends a THESIS substrate
  sentence (see recommendation 3).
- **Q-C2 — body attachment.** Recommended: `Arrow.body` edge, unifying internal and external
  realization under the existing E-9 authority. Confirm.
- **Q-C3 — lane ownership.** Wave 3 couples to Mgr-SPINE's ladder; waves 0–2 are
  substrate/pipeline work. One owner for 0–2 with an explicit handoff at 3, or SPINE owns it
  all? Operator call.
