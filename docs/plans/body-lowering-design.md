# Body-lowering — make v2's inferred tree faithfully represent runtime iteration

**Status:** design-first (model-before-implement). This document is the deliverable for the
BODY-LOWERING MILESTONE. It fixes the gap, the thesis, the fail-closed guarantees, the staged plan,
and the discriminating witnesses. No load-bearing pipeline/substrate edits land in the design PR;
each stage below is a separate, separately-signed implementation PR.

Reasoned serially, per DESIGN.md's preamble: §1 fixes the gap from receipts; each later section is a
consequence, not a restatement.

---

## 1. The gap (from receipts, not assertion)

DESIGN.md §4 fixes the substrate's iteration model: *"cyclic relations via acyclic encodings, never
cyclic values; recursion is sugar over `Loop`."* A `Loop` node
(`src/v2/std/node.dag:26` — `Behavior = Value | Transform | Branch | Loop | Bind | Match`) carries its
cycle as an **acyclic encoding**: exactly one `Named { name: ^loop_bound_edge }` edge (the descent
**measure**) plus ≥1 positional child (the iteration **body**), enforced by `LoopBoundEdges`
(`node.dag:210,279-284,402`). The measure edge *is* the acyclic stand-in for the recursion — no
cyclic value.

The substrate **models** this faithfully. The **inferred tree does not interpret it.** Three receipts:

1. **The Loop multiplicity decision is tag-only — it never reads the measure.**
   `node_multiplicity` (`cardinality.dag:230-235`) dispatches a `Loop` to
   `behavior_multiplicity(b: Behavior)` (`cardinality.dag:219-228`), whose **entire input is the
   behavior tag** — the `Node`, and therefore the `^loop_bound_edge` measure, is structurally not in
   scope. It returns `RequiresTerminationProof` unconditionally, which
   `multiplicity_termination_witness` (`cardinality.dag:199-207`) turns into a hard
   `Violates { cardinality_descent_not_proven }`. **Every raw `Loop` is rejected regardless of the
   descent evidence its measure edge carries.** The acyclic encoding of the cycle is present in the
   tree and *ignored* by the type — the precise "does not faithfully represent runtime iteration"
   symptom (this is the §5 "bounded-forever ≠ unknown" fail shape inverted: here a *provably-bounded*
   loop is indistinguishable from an unbounded one because the proof input is never consulted).

2. **The faithful carrier exists but is a dead scaffold.**
   `loop_bound_witness_for_node` (`cardinality.dag:278-289`) already builds the right shape —
   `LoopBound { measure: Node, termination: Witness<TerminationProof> }` (`cardinality.dag:119-122`)
   — pairing the extracted measure (`loop_measure_node`, `cardinality.dag:161-181`) with a termination
   witness. But (a) it has **zero callers** — `04_infer.dag` never consumes it; and (b) its
   `termination` field is computed by the same `termination_proof_witness_for_node` → tag-only path,
   so even the carrier that *names* the measure→proof binding **derives its proof without reading the
   measure**. A scaffold that duplicates a fact it does not actually compute (§5 parallel-representation
   debt; §6 named-dissolution-trigger missing).

3. **The loop body is typed opaquely — no per-iteration relation.**
   `04_infer.dag` has **no `Loop` arm.** `infer_gather_fold_init` (`04_infer.dag:933-985`)
   special-cases `Branch` (`:935`) and `Match` (`:944`) — each gets real per-arm body-type rows
   (`InferMatchArmRow { body_type }`, `:593-596`) unified across arms
   (`infer_unify_branch_arm_types`, `:484-497`). `Loop` falls to the generic `_ =>`
   (`:953`) → `infer_node_facts`, which types each positional child as an independent constraint-graph
   node with **no relation asserted between the pre-iteration and post-iteration type.** The tree says
   "these nodes exist," not "this body is applied repeatedly, threading a value." A loop is typed as
   if it were a one-shot conjunction.

**Net:** the substrate carries loop *shape* (measure + body edges) but the inferred tree carries
neither the loop's *termination fact* (gap 1/2) nor its *iteration fact* (gap 3). Body-lowering closes
both so the inferred tree is a faithful image of what runs.

## 2. Adjacent finding — `DescentEvidence` is forked (§3), and it blocks the fix

The descent derivation this milestone needs is **already modeled twice**:

- `src/v2/std/cardinality.dag:108-111` — bare `DescentEvidence = Strict | NonIncreasing |
  DescentUnknown`. No lattice, no combinator.
- `dag/std/termination.dag:5-` — the **same** `DescentEvidence` with the full authority: an
  `evidence_rank`, a `descent_evidence_bounded_lattice: BoundedLattice<DescentEvidence>` with
  `meet`/`join` (`termination.dag:18-52`), `promote_to_strict`, and per-key merge. `dag/std/computation.dag`
  consumes it.

This is a §3 nickname — one concept, two homes — and it is **on the critical path**: a loop whose
iteration fold has several contributing measures terminates only if **all** descend, which is exactly
a **lattice meet** over their per-edge `DescentEvidence`. The bare v2 copy cannot express that; the
`dag/std/termination.dag` authority can. So the fix must **ground the v2 descent derivation on the
single `DescentEvidence` authority (the lattice one)**, not extend the bare copy — otherwise the fix
itself widens the fork (§2 net-concepts-must-not-grow test). Resolving which layer that authority
lives in (the v2 `src/v2/std` tree vs the `dag/std` tree) is FLAG A below.

## 3. Thesis — derive the loop's type from its measure and its body, not from its tag

Two faithful bindings, mirroring the two facts a loop carries:

**T1 — termination is *derived from the measure*, fail-closed.** Replace the tag-only
`behavior_multiplicity(Loop) => RequiresTerminationProof` with a **node-aware** Loop multiplicity that
reads the `^loop_bound_edge` measure target and its contributing iteration-fold edges, extracts each
edge's `DescentEvidence`, combines them by **lattice meet** on the single authority, and maps:

- `Strict` → `ProvenTermination { proof }` (a real `TerminationProof` naming the ranking dimension the
  measure descends on);
- `NonIncreasing` → `ProvenTermination` **only** when paired with a strict dimension elsewhere in the
  fold (that is precisely what `TerminationProof { non_increasing: List, strict: RankingDimension }`
  at `cardinality.dag:115-118` already encodes — a lexicographic descent); a lone `NonIncreasing`
  stays `RequiresTerminationProof`;
- `DescentUnknown` or absent/malformed measure → `RequiresTerminationProof` → `Violates`
  (**construction-preserving fail-closed** — the honest bottom of the lattice; DESIGN.md §5: a
  provably-bounded loop must be *distinguishable* from an unknown one, and an unknown one must still
  refuse loudly).

This makes the measure edge *load-bearing in the type*: perturb the measure and the loop's
termination verdict changes — the acyclic encoding is now interpreted, not decorative. `LoopBound`
(gap 2) becomes the live carrier of this binding, and `loop_bound_witness_for_node` gains its first
real consumer (dead scaffold dissolved).

**T2 — the body is typed as a per-iteration transform.** Give `04_infer.dag` a `Loop` arm parallel to
`Branch`/`Match`: type the iteration-fold body (positional children, per
`loop_edge_contributes_to_iteration_fold`, `node.dag:374-387`) as a transform from the pre-iteration
carrier type to the post-iteration carrier type, and assert the **fixpoint relation** that the body's
output type unifies with its input type (a loop that changes the carrier's type each turn does not
have a runtime iteration semantics — it fails closed). The inferred tree then states "body : τ → τ,
repeated under measure μ," which *is* the runtime iteration. This reuses the existing arm-unification
machinery (`infer_unify_branch_arm_types`) rather than minting a parallel fold.

Both bindings are **`Loop` sugar readers**: per DESIGN.md §4/§7, `For`/`While`/`Retry` (and recursion)
are sugar over `Loop`, so typing `Loop` faithfully types all of them at once — no per-form N×M work.
This is the same "one grammar, both directions" move: the descent measure is read *forward* to a
proof; body-lowering is the inference direction of the same edge discipline `content_hash`
canonicalizes.

## 4. Non-goals / scope fence

- **Not** touching `program.dag` or surface syntax — this is inference over the already-lowered
  `Node` tree, not a parser change.
- **Not** adding orchestration `While.bound`/`Retry` emission — `orchestration.dag` already declares
  `While { bound: DescentEvidence }` (`orchestration.dag:56`); this milestone makes the *inference*
  that would justify such a bound faithful. The two meet at the shared `DescentEvidence` authority
  (FLAG A), not by touching orchestration here.
- **Not** enriching `03_body_producer.dag`'s MVP-1 fixture bodies (gap 3's degenerate single-literal
  body). That is a fixture-realism follow-on; T2 types whatever body the producer emits and does not
  require a richer producer to be faithful about the *relation*. Flagged as Stage 4 (optional).

## 5. Staged plan (each stage = one signed PR; strictly ordered)

Ordered so every load-bearing edit lands *after* its model, and each stage is provable by a red→green
witness before the next begins.

- **Stage 0 — this design PR (non-load-bearing).** This document + an open-thread bullet in DESIGN.md
  registering the milestone. No behavior change. *This is the PR this session opens.*

- **Stage 1 — resolve the `DescentEvidence` fork (§3 / FLAG A).** Ground the v2 descent path on the
  single authority (the `dag/std/termination.dag` lattice), deleting the bare `cardinality.dag:108-111`
  copy or making it a re-export, per the operator's single-authority ruling on the v2↔dag layer split.
  Witness: `descent_evidence_meet` over the lattice, red control = a perturbed meet. **Model-before:
  Stage 2 imports the consolidated authority; do it first.**

- **Stage 2 — node-aware Loop multiplicity (T1).** Introduce `loop_multiplicity(n: Node)` deriving
  `Multiplicity` from the measure via the Stage-1 authority; route `node_multiplicity` for
  `ComputationNode { behavior: Loop }` through it (Branch/Value/etc. stay tag-only — their multiplicity
  genuinely is tag-determined). Wire `loop_bound_witness_for_node` as the consumer (kills the dead
  scaffold). **Discriminating witness:** a Loop with a `Strict`-descent measure infers
  `ProvenTermination` (was `Violates`); the *same* Loop with the measure perturbed to `DescentUnknown`
  infers `Violates` — the verdict is a function of the measure. Load-bearing (`cardinality.dag`):
  higher bar, execution-proven, escalate if it touches beyond the Loop arm.

- **Stage 3 — Loop body arm in `04_infer` (T2).** Add the `ComputationNode { behavior: Loop }` arm to
  `infer_gather_fold_init`, typing the iteration fold as `τ → τ` and asserting the fixpoint
  unification, reusing `infer_unify_branch_arm_types`. **Discriminating witness:** a loop whose body
  preserves the carrier type infers a clean iteration type; a body whose output type diverges from its
  input infers a located mismatch (was silently accepted as opaque conjunction). Load-bearing
  (`04_infer.dag`): higher bar.

- **Stage 4 (optional follow-on) — body-producer fixture realism.** Replace the degenerate
  single-literal loop body (`03_body_producer.dag:851-874`) with a body that threads a binder across
  the iteration, so the fixtures *exercise* T2's fixpoint relation with a non-trivial carrier. Only
  after Stages 2–3 make the relation observable. May be split to a separate owner.

Each of Stages 1–3 is independently mergeable and independently valuable; Stage 3 does not require
Stage 2 to *land* but reads cleaner after it. Stage 2 requires Stage 1.

## 6. Fail-closed guarantees (§5) and construction-vs-validation

- The `DescentUnknown`/absent-measure → `Violates` mapping is **construction-preserving**: it is the
  lattice bottom, not a post-hoc check. A loop cannot be typed "terminating" without a measure that
  *derives* the proof — the bad state (a loop admitted with no descent evidence) stays unwritable,
  exactly as today, but now for the *right reason* (evidence absent) rather than the wrong one (all
  loops rejected).
- **No fabricated proof:** `NonIncreasing`-only never fabricates a `Strict` dimension; it stays
  `RequiresTerminationProof`. `promote_to_strict` (`termination.dag:52`) is applied only where a
  strict dimension genuinely exists in the fold.
- **Discriminating red control is mandatory per stage** — the design is not "done" until a consumer
  runs green *and* a perturbed measure/body runs red (DESIGN.md §5 spec-without-execution trap; my
  standing anti-overclaim culture — the honest bar is the deliverable).

## 7. Open flags for parent/operator sign

- **FLAG A (blocks Stage 1):** which layer owns the single `DescentEvidence` authority — the v2
  `src/v2/std` tree or the `dag/std/termination.dag` tree? Both exist; the richer lattice is in
  `dag/std`. Need the single-authority ruling before consolidating (this is the recurring v2↔dag
  cross-tree grounding question, not new to this milestone).
- **FLAG B (Stage 2):** `behavior_multiplicity` currently takes `Behavior`; making Loop node-aware
  means Loop's multiplicity is decided one layer up (`node_multiplicity`) while the other behaviors
  stay in `behavior_multiplicity`. Is asymmetry acceptable, or should `behavior_multiplicity` be
  retired in favor of a uniformly node-aware `node_multiplicity`? I lean asymmetric (the other
  behaviors' multiplicity *is* genuinely tag-determined; forcing them node-aware is ceremony), but
  flag it because it touches a load-bearing dispatch.
- **FLAG C (Stage 3):** the fixpoint relation `τ → τ` assumes the loop carrier is invariant across
  iterations. Some faithful loops *narrow* a type monotonically (refinement). First cut requires
  strict invariance (fail-closed); refinement-typed loops are a named follow-on, not silently
  admitted.
```
