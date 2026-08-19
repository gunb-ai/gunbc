# Discrete cost derivation — the work account between symbolic cost and measured wall

> **Status: DRAFT for operator review (2026-08-04).** Design-note-first: **no code lands from this note.** It declares the carrier shapes, the authority ruling, the phase order, and the discriminating controls for deriving a *discrete work account* from a DAG section plus an input plus a realization. Every carrier and symbol named below was verified against the live tree at this revision; the four corrections in section 4 are things the tree says that the originating sketch did not.
> **This note does not replace [`bounded-input-cost-envelope-scheduling.md`](bounded-input-cost-envelope-scheduling.md)** (operator-signed 2026-07-02) — it *fills that design's declared hole*. That doc rules bounded input, then cost, then schedule, and names `v2.lens.cost` as the symbolic-cost authority that must not be re-minted. It never says how a symbolic cost becomes a *number* at a declared input. This note is that step, and defers to the signed doc on every scheduling question.

## 1. The displaced cost (DESIGN §6 — denominate the benefit)

Per-PR witness admission is currently decided by a **directory name**. `gunbc_ci_fast_lane_rule_note` (`v2.workflow.ci_floor_plan`) rules that a witness whose own eval reaches `gunbc_ci_fast_lane_witness_eval_budget` (5 seconds) does not run in per-PR discovery; the fix is moving its file under `test/claim/long/`, which `gunbc.ci_layer_roots` excludes at dir grain. A wall-clock threshold on one host stands in for a cost model.

This is not a proposal to overturn that rule. **Both blanket-budget authorities already name this note's subject as their own dissolution trigger**, in identical words:

- `gunbc_ci_fast_lane_rule_note` — *'Dissolve-on: per-witness declared cost envelopes (the witness-cost-locality admission law) replace the blanket lane budget with declared rows.'*
- `gunbc_falsifier_substrate_long_lane_budget_note` — *'Dissolve-on: per-witness declared cost envelopes (the witness-cost-locality admission law, the same trigger `gunbc_ci_fast_lane_rule_note` names) replace every blanket lane budget with declared per-row rows.'*
- `long_lane_exclusion_note` (`gunbc.ci_layer_roots`) — *'Dissolve-on: per-witness declared cost envelopes (witness-cost-locality admission law) subsume the dir grain.'*

So the displaced cost is concrete and already priced by the tree: **three live scaffolds are blocked on one missing derivation**, and each additional witness that disappears into `long/` is a row that per-PR CI stops proving while a green CI keeps implying it did (the frontier per-module probe's exclusion note stated this outright — *green CI says nothing about survey numbers unless the local recipe below ran* — and that note has since been deleted with the roster it surveyed, which is the pattern rather than a counterexample to it). The benefit is not an elegant cost algebra; it is retiring a proxy that silently removes coverage.

**Priced honestly:** the threshold is also doing real work today and must not be removed before its successor exists. The 5s deadline is a *construction* wall (`long_lane_exclusion_note`: 'the 5s deadline makes an over-budget fast-lane witness unwritable-in-place, so the roster cannot silently grow stale'), and it was the tourniquet for the run-29183446733 wedge — 243 of 270 minutes burned in billed silence. A replacement that is merely more principled but not fail-closed is a regression.

## 2. Three quantities, never one number

The originating question asks for 'a literal number of cycles x reads'. **Cycles and reads are different dimensions and must not be multiplied into one scalar** — that product has no unit and no falsifier. Three quantities wear the word 'cost', and conflating any two is the DESIGN state-space conflation class:

| Quantity | Question | Epistemic status | Example |
| --- | --- | --- | --- |
| **Semantic work** | how many modeled operations must happen? | **Derived** from graph + input — exact where the input is known | 65 source reads, 65 content hashes, 931 calls, 64 loop iterations |
| **Realization cost** | what does that work cost under one implementation and machine? | **Predicted** — a projection requiring a calibrated model | interpreter dispatch vs native dispatch, hash cost per byte, cache hit vs miss |
| **Observed physical cost** | what happened during one execution? | **Measured** — falsifier only, never authority | 13.2 s wall, 3.1e9 cycles, 1.08 GiB peak RSS |

The first is the stable quantity and the one this note is mainly about: **it can be exact, and it is machine-independent.** The second is honestly a prediction. The third is an observation that calibrates or refutes the second — never overrides the first. This ordering is not new; it is `bounded-input-cost-envelope-scheduling.md` invariant 2 ('Predicted cost is authority; Measured is falsifier') applied one layer deeper, at the work-account grain rather than the `CostAccount` grain.

**Consequence for the original phrasing:** derive a *work vector*, then project it. A projection to cycles is a separate, target-qualified operation that may legitimately be unavailable — and when it is unavailable it must refuse, not fall back to a plausible number.

## 3. What already exists (verified against the tree)

Four load-bearing pieces exist. This note reuses all four and mints no parallel cost taxonomy (DESIGN §3).

| Piece | Authority | What it gives | What it lacks for this lane |
| --- | --- | --- | --- |
| Symbolic cost fold | `v2.lens.cost` — `SymbolicCost`, `symbolic_cost_of_node`, `cost_lens`, `asymptotic_class_of_cost` | total fold over the closed Node kernel via `fold_node`; `UnknownCost` is lattice top (fail-closed); loop cost reads the modeled bound, never a guess | **scalar, not a vector**; no span; no valuation environment; no binder-carrying sum |
| Loop bound + termination | `v2.std.cardinality` — `loop_bound_witness_for_node`, `loop_bound_measure`; `std.termination` `DescentEvidence` | a loop must carry one unambiguous bound measure or the fold yields `UnknownCost` with a `Diagnostic` — exactly the substrate a concrete iteration count needs | nothing — this is the piece that already works |
| Concrete cost expressions | `v1.compiler.complexity` — `CostExpr`, `ComplexitySummary`, `Certainty` | `CostSum { binder, upper, body }`, `CostMax`, `CostLog`; and a summary carrying `work` / `span` / `output_size` / `peak_space` / `certainty` | lives in the **v1 seed**, not the shared authority; not reachable from v2 consumers |
| Predicted vs measured realization | `std.realization_schedule` `CostAccount` / `CostBasis`; `std.realization_measurement`; `gunbc.witness_row_cost` | time/space/power with `Predicted | Measured`; and a **populated** dated basis corpus (`dag/gunbc/witness_row_cost_basis.tsv`, host_class `srv_fleet_arm64`) with drift verdicts | nothing feeds Predicted from graph structure — `cost_account_predicted_zero()` is the tell named by the signed scheduling doc's P1 |

There is also a live admission law: `v2.lens.witness_cost_locality` already classifies a witness `Local | CouplesToAmbient` from its import closure and routes it `AdmitContinuous | RouteScheduledLane` against a `WitnessInputEnvelope`. That is the consumer seam this note's output plugs into — see §11 C5.

## 4. Four corrections the tree forces

Each was checked by reading the carrier, not by recalling it. These change the work, so they are stated before the model.

1. **The missing piece is the valuation environment, not the type shape.** `v2.lens.cost` `SizeVariable` is `{ source: Node }` — a size variable *is a node*, and **no binding environment exists in v2** that maps one to a `Nat`. So 'evaluate the cost at length(xs) = 65' has no v2 seam today. Adding richer cost *atoms* without adding valuation buys nothing; C1 is principally the environment, and the atom vector is secondary. **Corrected 2026-08-05 by the capability census** (`gunbc.v1_complexity_capability_census` `concrete_input_evaluation`): an earlier revision said no such environment existed *anywhere in the tree*. The seed has one — `v1.compiler.complexity` `eval_size_expr_concrete` and `eval_cost_expr_concrete` both take `env: Map<String, Int>` and return `Int?`, refusing with `Absent` on `CostUnknown` / `CostExtern` / `CostLog` rather than fabricating a number. The narrower claim is the useful one: the seam is absent from **v2**, and v1's is **`String`-keyed**, which is exactly the anemic identity the parity note's §6 forbids carrying across. So C1 inherits a working refusal posture to preserve and a key to re-ground, not a blank page. The correction is carried by the census row `gunbc.v1_complexity_capability_census` `concrete_input_evaluation`, whose citation resolves exactly through `v2.std.decl_ref_resolution` in `test.claim.long.v1_complexity_capability_census_resolution_test` `census_every_declaration_ref_resolves_exactly`.
2. **`v2.lens.cost` already fixes branches as an upper bound, and cannot select an arm.** `compose_child_cost` composes `AlternativeCost` with `symbolic_max`, so a `Branch` / `Disj` / `Match` charges the dominating arm regardless of input. 'Charge only the selected arm' is therefore **a second evaluation mode over one expression**, not a fix to the existing fold. Keep one symbolic authority; add two evaluators (exact-at-input, bound-at-envelope). Changing the existing fold's meaning in place would silently move every current consumer, including the `WallNow` quadratic detector.
3. **v2's `SymbolicCost` cannot express an element-dependent fold body.** It has `SumCost` and `ProductCost` but **no binder-carrying sum**; v1's `CostExpr` has `CostSum { binder, upper, body }`. So `Σ over elements of cost(body(element))` — the shape the fold case needs — is representable in the seed and *not* in v2. This is a concrete instance of the DESIGN §7 direction (useful machinery migrates out of the seed), and it is a prerequisite for C2, not an optional cleanup.
4. **A hardcoded, uncited realization cost table already sits inside the agnostic symbolic lens.** `v2.lens.cost` `llvm_instruction_cost` returns a bare `Int` per `LlvmInstruction` (`Load` 4, `Call` 5, `AtomicRmw` 8, `Fence` 4, bitcast 0). That is a *machine model* — with no machine identity, no unit, no citation, no calibration, and no basis date — living in the module that is supposed to be realization-free. It is the §3 interface/realization fusion this note's C4 exists to undo, and it is **already in tree**, so C4 has a deletion target rather than only an addition.

## 5. The subject: what a cost derivation is *about*

A cost number with no bound subject is the provenance failure this repo keeps finding: a figure measured on a fixture, attached to a live-tree run. The subject is therefore structural, and every derivation carries it:

```
CostSubject {
  root: Node
  graph_identity: ContentHash
  input: ValueIdentity
  realization: RealizationIdentity
  cost_model_revision: CostModelRevision
}
```

`graph_identity` and `input` are content identities (`std.content_hash`, the `Fnv1a64Structural` family member `v2.std.node` `Hash` already uses); `realization` names the materialization/target decisions of §8; `cost_model_revision` versions the atom-cost table so a re-calibration is a visible revision rather than a silent restatement. **Two derivations may be compared only when their subjects agree on the axes being compared** — the same construction wall shape as `gunbc.econ.scm_serving_model` `VendorMismatchAcrossRows`, which makes 'S3 storage with R2 egress' unwritable rather than merely wrong.

## 6. The work account

Semantic work is a **multiset of cost atoms**, not a scalar. A closed atom vocabulary keeps the fold total by construction, the same property `base_cost_for_connective` / `base_cost_for_behavior` have today:

```
type WorkAtom
  = StructuralWork { kind: StructuralWorkKind }
  | PrimitiveWork { primitive: PrimitiveIdentity }
  | EffectWork { operation: DeclarationRef }

type StructuralWorkKind
  = EvaluateNode
  | TraverseEdge
  | InvokeFunction
  | DecideBranch
```

**Shape ruled 2026-08-05 (see section 15).** The outer sum is CLOSED so the fold stays total; the primitive and effect populations are OPEN through exact identities so a new measurable operation is a row, not a substrate edit. An unknown identity **refuses** — it never becomes zero-cost, which is the arm that would otherwise re-open the fabricated-plausible-output class one level down. The byte axes that an earlier revision carried as bare atoms (`AllocateBytes` / `ReadBytes` / `WriteBytes`) do not belong in this sum at all: they are quantities in a unit, not kinds of work, and the ruling is explicit that operation counts and byte quantities must not both be anonymous `Nat`s.

`DiscreteWork` is then a `Map<WorkAtom, Nat>` — one carrier, extensible by row rather than by field (DESIGN §2 horizontal: the record-of-named-counters shape re-forks every time an axis is added). The summary retains the axes v1's `ComplexitySummary` already proved useful:

```
type DiscreteWork { atoms: Map<WorkAtom, Nat> }

type DiscreteCostSummary {
  work: DiscreteWork
  span: DiscreteWork
  peak_space: SizeExpression
  output_sizes: Map<OutputIdentity, SizeExpression>
}
```

**`work` and `span` are separate and neither derives the other.** `work` is total operations; `span` is the critical path under perfect parallelism. A graph can have large work and small span or the reverse, and the scheduling consumer needs both — `std.realization_measurement` already models exactly this asymmetry for the measured side (sequential time sums / parallel time maxes; sequential peak space maxes / parallel peak space sums), so the derived side must match it or the two cannot be compared.

## 7. Four honesty states — the result is not always a literal

The derivation is total and fail-closed. It never returns a number it cannot justify, and it never silently contributes zero for something it failed to model (`zero_cost()` for an unmodeled construct is the fabricated-plausible-output class, DESIGN §5):

| State | When | Carries |
| --- | --- | --- |
| `ExactCost` | every size variable is bound to a literal by the input valuation | one `DiscreteCostSummary` |
| `BoundedCost` | sizes are declared as ceilings, not values (the `BoundedInput` case) | lower and upper summary + the `InputEnvelope` they were evaluated at |
| `UnresolvedCost` | sizes remain free variables | the authority's `v2.lens.cost` `SymbolicCost` expression, **carried unchanged**, plus the unresolved variables, named |
| `CostRefused` | unbounded loop, unresolved descent, unmodeled host call, absent effect model, unknown cache posture | a **NonEmptyList** of typed, located causes |

**Naming constraint (review 48133).** These four states are named for *valuation completeness*, and none of them may reuse a name the cost authority already owns. An earlier revision of this note called the third state `SymbolicCost` — colliding with `v2.lens.cost` `SymbolicCost`, the same word in the same domain for a different concept (an honesty state vs an asymptotic expression). That is precisely the §3 nicknaming class this note argues against, committed inside the note arguing it. The relationship is containment, not synonymy: `UnresolvedCost` **carries** the authority's `SymbolicCost` expression unchanged, which is the shape that keeps §13's do-not-re-mint ruling true rather than quietly forking it.

The refusal arm is the load-bearing one. It must **refuse, never widen and never narrow**: an unmodeled construct may not silently cost zero (narrow — the empty-observation class), and a refusal may not be absorbed into 'assume the maximum' (widen — the absorbing-fallback class). Both are named recurring failure modes in DESIGN. Causes are counted so their frequency ranks the next round of modeling, which is the entire mechanism by which this lane finds its own next work.

## 8. Sharing is a realization fact, and it changes the answer

A node appearing once in the source graph is **not** evaluated once. `std.realization` already models the decision — `Materialization = Recompute | Memoize | Share` — and `std.materialization_ladder` is the state x decision law over redundant computation. The same semantic graph therefore has several legitimate work accounts:

- `Recompute` — calls x body work
- `Memoize`, cold — body work + one cache write
- `Memoize`, warm — a lookup and a read, and **not** the body
- `Share` — body work once, plus reference projections

This is why `realization` is part of `CostSubject` (§5) and not an afterthought: a work account quoted without its materialization decisions is unfalsifiable. It is also the axis with the most immediate diagnostic value — a witness whose derived cost is dominated by repeated identical sub-derivations has a materialization defect, which is a strictly more actionable finding than 'this test is slow'. The warm-vs-cold distinction here is the same one the open `disk-tier repeat-resolve` thread is still owed a skip-counter proof for.

## 9. Projecting to cycles, time, and energy

Only after the work vector exists does a target-qualified projection make sense, against a model that names its machine:

```
type RealizationCostModel {
  target: MachineIdentity
  runtime: RuntimeIdentity
  cache_posture: CachePosture
  atom_costs: Map<WorkAtom, CostDistribution>
  basis: DatedBasis
}
```

Predicted cycles is then the sum over atoms of count x calibrated cost, and predicted time is a critical-path projection over `span` plus external-effect latency. Three constraints keep this honest:

1. **A projection with no calibrated row for an atom refuses** — it does not assume 1, and it does not drop the atom. This is where `llvm_instruction_cost`'s bare `Int` table (correction 4) is re-homed: per-target, cited, unit-carrying, dated, and *outside* `v2.lens.cost`.
2. **Exact cycles on real hardware is not claimable.** Cache behaviour, branch prediction, instruction scheduling, co-tenancy, frequency scaling and I/O latency are outside the modeled guarantee — the DESIGN §4b column that is deliberately *not* a ladder rung. An exact deterministic cycle count is claimable only against a specified abstract machine or simulator, and that is a different `RealizationTarget`, declared as such. For native hardware, cycles are an **observation that calibrates**, and the honest ceiling for the projection is a distribution, not a scalar.
3. **The basis is dated and cited, per `gunbc.witness_row_cost`'s existing rule** — a basis row names an arm64 srv-fleet run id, never a local x86 measurement, and a re-basis is an appended dated row, never a silent widen.

The measured corpus for calibration exists but its provenance is NOT sound, and C4 must not be planned as though it were. `dag/gunbc/witness_row_cost_basis.tsv` carries 1,173 rows seeded before `ClaimOutcome::TimedOut` could distinguish a killed run from a completed one, so an unknown subset of them are right-censored figures — a deadline ceiling recorded as if it were a cost. For exactly those rows the 2x comparator returned `WithinBasis` against the ceiling, which is not merely unvouched but wrong in the fail-open direction. C4 therefore joins to a corpus of unknown provenance, not to a measurement pipeline. Its real prerequisite is the completion axis reaching the basis rows and the provenance_unknown population draining; calibrating before then calibrates against censored values.

## 10. Why a large input is or is not honest

Cost alone cannot answer whether a witness is merely proving that the computer can compute. That is a second, orthogonal question — **what semantic fact does the extra work establish?** — and it is the question the directory heuristic never asks. A large input is justified only when it is one of:

```
type CostJustification
  = MinimalCounterexample
  | BoundaryCrossing { modeled_boundary: DeclarationRef, chosen_input: ValueIdentity }
  | PopulationTotality { population: DeclarationRef }
  | ExternalFidelity { effect: DeclarationRef }
  | ResourceContract { resource: ResourceKind, bound: Measure }
```

If increasing N crosses no modeled boundary, enlarges no population whose totality is the claim, exercises no external effect that is itself the subject, and tests no declared resource bound, then a larger N is just more computation. Worked example, on a real shape from this tree: for a closure-matrix admission law whose assertions are 'duplicate ref refuses', 'wrong digest refuses', 'entry absent refuses', the semantic cost is O(refs) and the discriminating fixture is two or three rows. Where an inline cap of 64 is the modeled boundary, the smallest discriminating input is **65** — not an arbitrarily large corpus — and the real 91-read compiler closure is then a *separately scheduled* integration receipt. Per-PR gets tiny algebra controls plus one minimal boundary fixture; the scheduled lane gets the corpus. No arbitrary scale anywhere.

This decomposition is the existing W/C/F test-decomposition practice made derivable rather than judged: wet/external work is honestly scheduled, corpus totality splits mechanism fixtures from corpus execution, and **fixture-shaped work that is nonetheless slow is a cost defect, not a long test** — today it is indistinguishable from an honest one, because both merely exceed 5 seconds.

## 11. Phases

Ordered; each names its acceptance. **This note lands the design record only.** Phases dispatch as separate work items. The lane runs parallel to the self-host survey chain and blocks none of it.

**Scope seam (added 2026-08-04).** [`v2-complexity-capability-parity.md`](v2-complexity-capability-parity.md) now owns the *engine* half of this program — the richer `SizeExpr` / `CostExpr` algebra, the restored `ComplexitySummary` with work/span/output-size/peak-space, and the interprocedural and recursion capabilities missing from v2. The C0 authority ruling below is stated in full **there** and is not restated here; C2's binder-carrying sum and span derivation are **that note's C1 and C2**. What stays this note's own: the valuation environment (C1), effect demand (C3), calibration (C4), and the admission consumer (C5). Neither note may grow into the other.

1. **C0 — consume the authority ruling (owned by the parity note; nothing to sign here).** The ruling that fixes which module owns cost expressions, asymptotic projection, realized accounts, and the v1 seed's disposition is stated in full at [`v2-complexity-capability-parity.md`](v2-complexity-capability-parity.md) §5 and is **deliberately not repeated in this note** — a second home for one ruling is the §3 violation both notes exist to avoid. This note's only C0 obligation is to *depend* on it: no phase below may begin while the ruling is unsigned, because each assumes `v2.lens.cost` is the single home its output attaches to. **Accept:** the parity note's C0 is signed, and correction 4's `llvm_instruction_cost` has a declared destination under it.
2. **C1 — the valuation environment and exact pure work.** The seam correction 1 says is absent: bind `SizeVariable` to values, and evaluate to `ExactCost | BoundedCost | UnresolvedCost | CostRefused` over sequential nodes, branches with known conditions, known-finite lists, bounded folds and direct calls. **Accept:** doubling a bounded list doubles the derived fold-body work, by execution; a missing loop bound yields `CostRefused`, not zero; pilot on a real admission function whose cost is independently obvious.
3. **C2 — sharing, and consumption of the engine's loop/recursion/span work.** The binder-carrying sum (correction 3), `span` derivation, and the SCC/descent machinery are the parity note's C1/C2/C4 and are **not duplicated here**; this note's own C2 is the materialization axis — make `Materialization` a derivation input so the same semantic graph derives different work under `Recompute` / `Memoize` / `Share` (§8). **Accept:** memoized and recomputed realizations of one graph derive different work; unknown descent refuses (consumed, not re-derived).
4. **C3 — effect demand, counted exactly.** Count filesystem reads/writes and bytes, subprocesses, network operations, host compiler invocations, as `ExecuteEffect` atoms keyed by `DeclarationRef`. **Assign no latency where no model exists** — counts stay exact and the projection refuses, which is strictly better than a fabricated duration. **Accept:** changing a fixture's file size changes derived read-byte demand; an unmodeled effect refuses rather than costing zero.
5. **C4 — realization calibration against the existing basis.** Project the work vector to predicted time/space under a pinned machine model; compare against `dag/gunbc/witness_row_cost_basis.tsv`. Re-home `llvm_instruction_cost` here, cited and dated. **Accept:** a planted omission in the model makes predicted and instrumented counts disagree and the falsifier goes red.
6. **C5 — the admission consumer (this is what retires the proxy).** Replace 'path contains `test/claim/long/`' with a declared cost envelope plus a `CostJustification` plus a named cadence, consumed by floor admission through `v2.lens.witness_cost_locality` `per_pr_admission` — whose own precision-frontier note already names this construction successor and the hand rosters it subsumes. **A missing derivation or a missing consumer refuses; it never silently becomes offline.** **Accept:** the three dissolve-on triggers in §1 fire together, and `witness_exclusion_substrings` hand-rows retire with them.

## 12. Discriminating controls (each must go red when the behavior is wrong)

Specification-without-execution is the trap this lane is most exposed to, because a cost model is exactly the kind of artifact that type-checks and reads plausibly while computing nothing. Every phase lands with its RED:

- Doubling a bounded list's length doubles exact fold-body work — and a fold whose per-element cost is charged once fails it.
- A loop with no modeled bound yields `CostRefused`, never a zero or an assumed iteration count.
- Sequential and parallel graphs of identical work have **different span**; a span that equals work in both cases proves span was never derived.
- `Memoize`-warm and `Recompute` realizations of one semantic graph derive different work accounts.
- A branch with a known-true condition charges only the taken arm; with an unknown condition it returns `BoundedCost` or `UnresolvedCost` — **never a fabricated exact number**.
- A filesystem read counts one effect atom plus exact bytes; changing the fixture's size changes the derived demand.
- A redundant second resolve *increases* derived work unless sharing structurally removed it — the discriminating control against a model that silently assumes ideal caching.
- A witness routed to a long lane with no cost envelope and no executing consumer **refuses admission** rather than being quietly excluded.
- A predicted atom count planted wrong disagrees with the instrumented execution count and the falsifier reds — the C4 calibration oracle.
- A projection asked for an atom with no calibrated row refuses; it must not silently assume unit cost.
- Two derivations whose `CostSubject` disagrees on realization or input cannot be compared — the comparison refuses by construction, mirroring `VendorMismatchAcrossRows`.

## 13. Single authorities — do not re-mint (DESIGN §3)

- **Symbolic / discrete work** — `v2.lens.cost` (extend; do not create a sibling `dag_cost_lens`)
- **Loop bound / termination** — `v2.std.cardinality`, `std.termination` `DescentEvidence`
- **Cost bound from sub-value structure** — `std.induction` `CostBound`, `catamorphism_bound`, `derive_bound` (added 2026-08-05 by the capability census, C0(b) finding 2: this vocabulary was absent from this list while already carried by shared substrate that the seed imports, so a richer cost algebra could have re-coined it)
- **Call graph / SCC** — `std.graph` `graph_multi_node_scc_members`, `is_valid_proof`; **recursion shape** — `std.computation` `LoweringTarget`, `lower_call_pattern` (same finding: shared substrate awaiting a v2 consumer, not seed machinery awaiting migration)
- **Input envelope** — `gunbc.ci_input_envelope` `InputEnvelope`
- **Realized time/space/power** — `std.realization_schedule` `CostAccount` with `CostBasis`
- **Measured observation** — `std.observation` `ObservationEvent` via `gunbc.witness_row_cost` (falsifier only)
- **Materialization decision** — `std.realization` `Materialization`, ruled by `std.materialization_ladder`
- **Admission** — `v2.lens.witness_cost_locality` `per_pr_admission` (extend its envelope; do not fork the matrix)
- **Scheduling** — `bounded-input-cost-envelope-scheduling.md` remains the signed authority; cost informs scheduling, never selection

## 14. Non-goals

- No exact physical cycle claim on native hardware (§9.2) — only against a declared abstract-machine target
- No new timer inside `claim_executor`; this is a derivation over the modeled graph, not more instrumentation
- No change to selection — cost may not decide *whether* a result depends on what changed
- No removal of the 5s deadline before C5's consumer is green; the tourniquet outlives the proxy
- No parallel cost taxonomy, and no scheduler-private cost record

## 15. Operator rulings (2026-08-05)

The four questions this note raised are ruled. Where a ruling changes a carrier shape, the shape is stated here and the carrier is the authority once it lands.

1. **`WorkAtom` — closed OUTER algebra with typed OPEN identities.** Neither a fully closed enum (every new measurable axis becomes a substrate edit) nor an open `DeclarationRef` key (loses totality). The shape is a three-arm outer sum — `StructuralWork` carrying a closed `StructuralWorkKind`, `PrimitiveWork` carrying a `PrimitiveIdentity`, and `EffectWork` carrying a `DeclarationRef` for the operation — where `StructuralWorkKind` is closed and the primitive and effect populations expand through exact identities. **An unknown identity REFUSES; it never becomes zero-cost** — the fold stays total at the outer layer while the inner populations grow without substrate edits. **Units are retained:** operation counts and byte quantities must not both be anonymous `Nat`s.
2. **v1 `CostExpr` migration belongs to the parity note's C1**, not to the generic seed-shrink lane. Seed deletion consumes the migrated authority; it does not decide its semantics.
3. **Span stays in C2.** Deferring it would create an incomplete summary immediately and prevent derived and measured parallel composition from agreeing.
4. **C5's floor flip is TWO changes, never one.** (i) Shadow or audit the derived per-witness admission and MEASURE the resulting floor; (ii) separately retire the directory-grain exclusion, and only after every moved witness has an executing consumer. A same-PR 'new cost model plus delete the `long/` policy' flip is refused.
5. **Three-valued `CostBasis` — approved PER RESOURCE AXIS.** `Derived | PredictedFromCalibration | Measured` applied to time, space and power independently, not once to the whole account: one account may legitimately carry time `PredictedFromCalibration`, space `Derived` and power `Measured`. The receipt that this is a real gap rather than a speculative widening is the census row `gunbc.v1_complexity_capability_census` `cost_account_space_bridge` — the seed already writes down an intent for a derived basis that the two-valued authority cannot represent.

## Dissolution trigger (DESIGN §6)

Delete this doc when the C0 authority ruling is stated on the carriers themselves (v2.lens.cost owning discrete work, the v1 CostExpr machinery migrated, llvm_instruction_cost re-homed to a cited per-target model), a DAG section plus a declared input plus a realization derives ExactCost or BoundedCost or refuses with typed counted causes by execution, the projection to predicted time calibrates against dag/gunbc/witness_row_cost_basis.tsv with a planted-omission RED, and floor admission consumes declared per-witness cost envelopes so gunbc_ci_fast_lane_rule_note, gunbc_falsifier_substrate_long_lane_budget_note, and long_lane_exclusion_note all fire their own dissolve-on together — at which point the carriers state the policy and this note retires.
