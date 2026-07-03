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

2. **The faithful carrier exists but its termination field is computed measure-blind, and no
   *inference* consumes it.**
   `loop_bound_witness_for_node` (`cardinality.dag:278-289`) already builds the right shape —
   `LoopBound { measure: Node, termination: Witness<TerminationProof> }` (`cardinality.dag:119-122`)
   — pairing the extracted measure (`loop_measure_node`, `cardinality.dag:161-181`) with a termination
   witness. It *is* consumed today — but only by the **cost/complexity** lenses
   (`src/v2/lens/cost.dag:168` `base_cost_for_loop`, and the complexity-budget roster test
   `src/v2/test/claim/complexity_gate/subject_complexity_budget_roster.dag:45`), and those read only
   the `measure` field (via `loop_bound_measure`) to size cost — `linear_in_node(measure)`
   (`cost.dag:170`) — never the `termination` field. The gap is two-fold: (a) **no inference
   consumer** — `04_infer.dag` never reads this carrier, so the loop's termination fact never reaches
   the inferred tree; and (b) the carrier's `termination` field is itself computed by the same
   `termination_proof_witness_for_node` → tag-only path (`cardinality.dag:284`), so even the field that
   *names* the measure→proof binding **derives its proof without reading the measure**. The carrier
   names a fact it does not actually compute (§5 parallel-representation debt): a `LoopBound` whose
   `termination` is measure-blind is a second, hollow representation of a proof that the measure
   already determines.

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
termination verdict changes — the acyclic encoding is now interpreted, not decorative. `LoopBound`'s
`termination` field (gap 2) stops being measure-blind: it is *derived from* the measure, so the
carrier's two fields become consistent, and `04_infer` gains the **first inference / measure-aware
termination consumer** of it (the cost/complexity lenses already read its `measure` for sizing, but
never the `termination` fact — this closes gap 2's hollow half).

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

- **Stage 1 — resolve the `DescentEvidence` fork (§3 / FLAG A — direction RULED, landing HELD).**
  Ground the v2 descent path on the single authority: **`dag/std/termination.dag` is the authority**
  (parent ruling 2026-07-03, §7 — `.dag` is truth, `src/v2/std` is the transient seed that shrinks to
  zero; and the richer lattice already lives there). Consolidate the bare `cardinality.dag:108-111`
  `DescentEvidence` **into** `dag/std/termination.dag`, then repoint/delete the v2 copy. Witness:
  `descent_evidence_meet` over the lattice, red control = a perturbed meet. **Model-before: Stage 2
  imports the consolidated authority; do it first.** **LANDING HOLD:** design against this direction
  now (unblocked), but do **not** land the consolidation until parent relays the operator's *standing*
  confirmation that `dag/std` wins v2↔dag forks as a durable rule (covers this, the pending
  `QualifiedName` de-fork, and future forks — imminent, direction very unlikely to flip).

- **Stage 2 — node-aware Loop multiplicity (T1).** Introduce `loop_multiplicity(n: Node)` deriving
  `Multiplicity` from the measure via the Stage-1 authority; route `node_multiplicity` for
  `ComputationNode { behavior: Loop }` through it (Branch/Value/etc. stay tag-only — their multiplicity
  genuinely is tag-determined). Make `loop_bound_witness_for_node`'s `termination` field derive from
  the measure (not the tag-only path) and route inference through it — the **first measure-aware
  termination consumer** (the existing cost/complexity readers at `cost.dag:168` /
  `subject_complexity_budget_roster.dag:45` use only the `measure` field and are unaffected).
  **Discriminating witness:** a Loop with a `Strict`-descent measure infers
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

## 7. Flag dispositions (parent-signed 2026-07-03)

- **FLAG A — RULED (direction), landing HELD.** The single `DescentEvidence` authority is
  **`dag/std/termination.dag`** (§7: `.dag` is truth, `src/v2/std` is the shrinking seed; the richer
  meet/join/`promote_to_strict` lattice already lives there). This is one instance of the recurring
  v2↔dag fork class (same as the pending `QualifiedName` de-fork). Stage 1 is designed against this
  direction now, but its **consolidation landing waits** on the operator's *standing* confirmation
  (parent relaying) that `dag/std` wins these forks as a durable rule — so we don't re-escalate per
  fork. Direction very unlikely to flip.
- **FLAG B — SIGNED (accept the asymmetry).** Loop's multiplicity is decided one layer up
  (`node_multiplicity`, node-aware) while the other behaviors stay tag-only in `behavior_multiplicity`
  — and that is **principled, not sloppy inconsistency**: Loop is genuinely different because its
  termination *depends on the measure*, which is a node-level edge the `Behavior` tag structurally
  cannot carry. The other behaviors' multiplicity **is** tag-determined; forcing them node-aware would
  be false uniformity (ceremony over a distinction that isn't there). The Stage-2 implementation must
  carry this rationale in-code so the asymmetry reads as a real distinction.
- **FLAG C — SIGNED, with one §5 requirement.** First cut handles carrier-invariant `τ → τ` loops
  (typed + green). Refinement-typed loops (carrier narrows monotonically across iterations) are a
  named follow-on — but they must be **fail-closed in the first cut**: a refinement loop is typed
  `DescentUnknown` / **refused with a located diagnostic**, *never* silently fabricated as `τ → τ`. A
  refinement loop mis-typed as invariant is a fabricated wrong answer (§5). So: `τ → τ` → proven;
  detected-refinement → refused, with the refinement-typed follow-on as its named dissolution trigger.
  The Stage-3 discriminating witness must include a refinement-shaped loop that goes **red/refused**
  (not silently accepted).
```
