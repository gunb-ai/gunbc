# Machine-shape orthogonal scheduling — one realization kernel over faithfully modeled machine facts, with locality

Status: DRAFT for operator review (2026-07-15, session sunny-deer-248). No code lands from this doc; every phase below is model-before-implement and names its trigger. Receipts cite the live tree as of 9011600a92.

## 0. Displaced cost (§6 — what pain this removes)

- **Fusion is modeled but unpriced.** The accelerator demo's `RunnableKernelWorkload { fused_op_count }` is the on-chip form of cache shaping (operand stays in registers across ops), but its cost is literally `cost_account_predicted_zero()` (`dag/gunbc/accelerator_demo_plan.dag:87-89`): fused and unfused have identical (zero) cost, so nothing can *justify* fusion. A decision the model makes but cannot price is a decision made by convention (§1).
- **The reduce spine on parallel hardware.** `docs/plans/execution-spine-design.md:67` measures 1/128 serialism from non-monoidal accumulator-threading; its enforcement item "every combine inhabits Monoid" is design-level only — no carrier a scheduler could dispatch on exists (`Monoid`/`CommutativeMonoid` are field-identical records, `dag/std/algebra.dag:14-23`; no law is machine-carried).
- **Placement is modeled but inert.** `dag/gunbc/roadmap_authority.dag:285`: "`Placement`/`Materialization` are modeled but inert — CI consuming them is the lane's real deliverable." The demo stamps `LocalAccelerator` as inspectable data; nothing routes execution by it. The WGSL handler is `DormantAwaitingOperator`.
- **Locality handled reactively, with a declared dissolve-on.** Runtime AIMD memory governor note (`src/v2/workflow/ci_floor_plan.dag:225`): "Dissolve-on: graph-derived per-node demand (CostAccount.space measured) replaces the reactive estimator." `CostAccount.space` is first-class but constructor-zeroed (`dag/std/realization_schedule.dag:39-46`); populating it is already declared P1 (`docs/plans/bounded-input-cost-envelope-scheduling.md` §7.2).

## 1. The two design axioms

**A. Machine shape is data — and there is no machine *kind*.** One scheduling kernel; machine shapes are rows — never two schedulers (the §4 N×M-adapter trap). Stronger (operator directive, 2026-07-15): "CPU vs GPU" is not a modeled distinction, the same way "batch vs streaming" is not — a nickname pair for points on shared axes (§3). No `MachineKind = Cpu | Gpu` discriminant may exist, and no `match` over one can, because the types carry no such field — the fork is unwritable by construction (§5), not merely discouraged. Every scheduling difference must *emerge* from faithful facts:

- **"serial folds are bad on GPUs"** derives as idle-lane cost — a loop-carried dependency utilizes 1 of `lane_count` lanes; at `lane_count = 1` the *same formula* says the serial fold wastes nothing. A scalar core, a SIMD core (8–16 lanes), and a GPU SM cluster (thousands) are three fills of one axis, and the schedule's response is monotone in it — no threshold, no kind test.
- **"transfer cost to the device"** derives as interconnect-crossing in the machine graph — and vanishes *by topology* on unified-memory shapes (the cited `apple_m5` row shares the DRAM level between execution domains), never by a special case. A discrete card and an integrated one are two topologies of one model.
- **"batch vs streaming"** is already dissolved by the substrate's bounded-and-forward execution: everything is a fold over a bounded prefix; residency windows (`transfer_grain`) express streaming without a second mode.

The orthogonality claim: computation graph × machine facts × algebraic evidence → schedule, each axis independently variable, no cross-term hand-authored.

**B. Every quantity carries evidence; the abstract shape is a first-class citizen, not a degraded one.** We cannot always know the concrete hierarchy (cloud VM, unknown SKU, future device). The intersubjective grounding (§3/§4 — cite the accepted framework, don't re-coin) is the **external-memory / ideal-cache model** (Aggarwal–Vitter 1988; Frigo–Leiserson–Prokop–Ramachandran 1999): a memory level is characterized by capacity M and transfer grain B, and the cache-oblivious result proves shape-level decisions (recursive blocking, fusion, affinity) are correct for *all* (M, B) simultaneously — only *pricing* needs concrete fills. So:

- **abstract shape** = the level structure with `Unknown`/`Bounded` magnitudes — sufficient to *derive legality and preference order* of locality decisions;
- **concrete shape** = the same structure with `Cited`/`Measured`/`Derived` magnitudes — sufficient to *price* them (roofline).

Unknown magnitudes fail closed to "no claim" (no affinity asserted, no accelerator placement chosen), never to a fabricated default (§5). Precedent for exactly this posture: operator ruling 2026-07-12 (`dag/std/realization_schedule.dag:82`) — "Memory class is a structural marker, not a quantity... Quantified per-runnable demand returns when it is derivable from the graph, not as authored literals."

## 2. Phase 1 — model the machine shape (std + extdeps; no scheduler change)

New std authority (name open: `std.machine_shape`):

```
type Quantified<T> = Cited { value: T, source: DeclarationRef }
                   | Measured { value: T }
                   | Derived { value: T }
                   | Bounded { lo: T, hi: T }
                   | UnknownQuantity
```

(One evidence coproduct for all magnitudes; mirrors `CostBasis = Predicted | Measured` and the `DescentEvidence` fail-closed-bottom shape. If a `Quantified`-like carrier already exists when this lands, converge — do not mint.)

```
type MemoryLevel {
  capacity:       Quantified<ByteSize>
  transfer_grain: Quantified<ByteSize>      -- the ideal-cache B: cache line / page / DMA burst
  bandwidth:      Quantified<Bandwidth>
  latency:        LocalityTier               -- see unification below
  sharing:        PerLane | PerCoreCluster | PerDevice | PerHost | Global
}
type MemoryShape = List<MemoryLevel>         -- ordered outward; the recursive tower
type ExecutionShape { lane_count: Quantified<Int>, grouping: LaneGrouping }
    -- scalar core / SIMD core / GPU SM: one axis, three fills — no kind tag
type ExecutionDomain { execution: ExecutionShape, memory: MemoryShape }
    -- a domain owns the private prefix of its tower
type InterconnectEdge = LinkEdge { link: PcieLink } | SharedLevelEdge { level: MemoryLevel }
    -- discrete device = LinkEdge; unified memory = SharedLevelEdge (apple_m5); same model, two topologies
type MachineShape { domains: List<ExecutionDomain>, edges: List<InterconnectEdge> }
```

A host is a *graph* of execution domains whose memory towers may share levels. "Where a computation runs" is a domain in this graph; "what a move costs" is the edge it crosses (zero edges crossed = zero transfer, emerging on unified-memory shapes with no special case). There is deliberately no field that names what a domain *is* — only what it *has*.

**§3 de-forks this phase performs rather than creating:**

1. **`LocalityTier` unifies with `cache_interface`'s lattice, not beside it** (decided 2026-07-15: the lattice lifts *out* to the std authority and `cache_interface` imports it). `dag/std/cache_interface.dag` already has `PersistenceLocality = InProcess | PerRunnerFilesystem | PerHostFilesystem | CrossHostNetwork` and `ReadLatencyClass = InProcessNs | LocalDiskUs | LanMs | WanTensMs` with a monotone-locality law (`cache_layer_ids_respect_locality`, :382-403). A register/L1/LLC/DRAM tier is **new rungs on that one lattice** (below `InProcessNs`), not a second hierarchy. The nanosecond memoization and the CAS artifact store are the same concept at two breadths — §2 horizontal, the same claim the Realization pattern already proved across ns→sccache→OS provisioning. Keying semantics (`CacheKeying`, `ProviderTier`) stay `cache_interface`'s own; only the locality/latency axis unifies.
2. **GPU interconnect de-forks `PcieLink` from storage.** `NvmeDeviceCatalogRow.pcie_link: PcieLink` with derived per-lane bandwidth already exists (`dag/extdeps/storage/types.dag:23-38,56`); `GpuModelCatalogRow` has no bus. Move/share `PcieLink` at the layer both can import and add `interconnect: PcieLink` to GPU rows — cited, like the NVMe rows.
3. **CPU catalog rows gain cited cache geometry** (`dag/extdeps/cpu/*`): L1d/L2/L3 capacity + line size per SKU, cited to vendor datasheets — same discipline as the DRAM organization axes. The Ampere row is the natural first fill (fleet hardware).
4. **Concrete GPU fill**: RTX 5090 / Apple M5 rows already carry `execution_lane_count`, `memory_bandwidth`, `boost_clock` (`dag/extdeps/gpu/nvidia.dag:18-38`) — enough for a roofline once shaped. Add SM shared-memory/register-file levels as cited rows. (The demo cites an "A100" with no catalog row — either add the row or repoint the demo at a cited device.)

**Doc-divergence repair (small, honest):** `CostAccount` is `{ time, space, power: Watt }` while DESIGN.md §2 says `Cost = Time|Space|Energy`; `measure.dag` has `Power` but no `Energy` quantity. Either add `Energy` (Watt·Time, derivable) or correct the doc — flagging for operator, not deciding here.

Acceptance (green-by-execution): witness that one `MachineShape` value with all-`UnknownQuantity` magnitudes is constructible and consumable (the abstract shape is a citizen); witness that the concrete RTX 5090 fill derives a roofline bound; RED: a hand-authored magnitude with no citation refuses (no authored literals — the 2026-07-12 ruling generalized).

## 4. Phase 2 — operand facts on edges (the crux the survey located)

Today `DependencyView` edges carry no operand: the scheduler collapses `DataDependsOn` to a Bool (`src/v2/workflow/scheduler.dag:55-73`), so shared-operand structure — N consumers of one value — is invisible to batch formation. Locality is a property of *edges* (data flowing), not nodes; this phase gives it a carrier.

- `OperandFlow { edge: DependencyView, operand: ContentHash, footprint: Quantified<ByteSize> }` — a derived row, not a stored field. `footprint` binds to the existing single authority `node_keyed_graph_transitive_bytes` (`src/v2/workflow/realization_runner.dag:60`: "transitive footprint is DERIVED, never stored... no parallel size walk may exist anywhere") — deriving through it is mandatory; a second size walk is a hard reject. Unsettled footprint → the existing `NodeKeyedGraphTransitiveBytesUnsettled` refusal arm, surfaced as `UnknownQuantity`, never a guess.
- The operand's `ContentHash` is the *same* identity `materialize` keys `Share` on — one identity, two schedule consequences: dedup (don't compute twice) and affinity (consumers run adjacent). This is the §3 unification that makes cache shaping fall out of machinery we already trust.
- Populate `CostAccount.space` from the same authority (discharges the declared P1 and the AIMD governor's dissolve-on trigger).

Acceptance: for a fixture graph with one Substantial shared operand and two consumers, the derived `OperandFlow` rows name the same `ContentHash` with equal derived footprints; RED: a graph whose footprint is unsettled yields typed `UnknownQuantity` rows and *no* affinity claim downstream.

## 5. Phase 3 — locality: affinity as derived topology (every level of the tower, any machine)

The repel primitive exists (`runnable_excludes_corpus_co_residence` → `ResourceDependsOn` edges, `ci_floor_plan.dag:322-352`); the attract dual does not. Constraint from the signed scheduling design (`bounded-input-cost-envelope-scheduling.md` invariant 4): "Schedule topology stays central... measured decisions must not break `schedule_eq` across hosts." Therefore affinity must be **derived centrally from graph facts** (OperandFlow rows — pure, deterministic, host-independent), exactly like the exclusion edges are today. It is topology, not a peripheral host tweak:

- Rule: consumers of one `Share`d operand whose footprint is `Substantial` relative to a memory level's capacity prefer co-batch placement within that operand's residency window; where the readiness frontier permits multiple legal batch compositions, the affinity rule selects among them. Where it permits only one, affinity changes nothing (schedule_eq preserved trivially).
- Pricing (what makes it §6-honest, not taste): a `TrafficAccount` per level boundary — bytes crossing each `MemoryLevel`, derived from schedule × OperandFlow × MachineShape. Roofline: `predicted_time = max(compute_time, traffic_i / bandwidth_i)` over levels with known quantities. Fused-vs-unfused kernels now price differently — the Phase-0 gap closes. With `UnknownQuantity` bandwidth the traffic term is absent and the prediction says so (`Predicted` basis, term-incomplete — a typed, countable degradation, never a silent default).
- Anti-goal (§4): no tunable knobs — no prefetch distances, no blocking factors, no thresholds. Every decision is derived from (footprint vs capacity) comparisons over evidence-carrying quantities; absent evidence = no claim.

Acceptance: a two-consumer shared-operand fixture schedules the consumers co-batch when capacity evidence admits both, with a `TrafficAccount` showing the operand loaded once; RED (discriminating): force the consumers into separate batches and the account shows the double load — the defect is *visible in the account*, which is the point.

## 6. Phase 4 — within-fold parallelization: licensed by algebraic evidence, priced by lane facts

A serial fold is optimal at `lane_count = 1` and leaves `lane_count − 1` lanes idle otherwise — one derived cost, no machine kinds. What licenses parallelizing *within* a fold (tree-reduce/scan) is associativity of the combine. MapReduce demands it by assertion; this substrate can demand it **by executed witness**. Note the first consumer is not a GPU: the demo's contiguous-loop/SIMD path is a CPU execution domain with `lane_count > 1`, and it is where the seam already lives ("Build the seam on CPU... same plan, one target row → GPU", accelerator-demo-roundtrip.md:110-111) — the GPU is the same recognition with a larger fill and an interconnect edge to price, not a second path:

- **Law carrier.** The mechanism already exists in two halves: law obligations consumed by testgen (`NatAlgebraLawObligation` rows incl. `^law_nat_add_associativity`, `src/v2/std/nat.dag:83-122`) and green-by-execution monoid law witnesses (`keyed_delta_fold_witness_test.dag:116-146`). Phase 4 joins them into a carrier a scheduler reads: `AssociativityEvidence = WitnessedAssociative { law: DeclarationRef } | NotWitnessed` — fail-closed bottom, mirroring `DescentEvidence`. Commutativity is **never** required: structural order is preserved by the DependencyView/tree-reduce (the survey's sharpest finding — Feinberg's commutative-monoid demand exists only because hash-shuffle destroys order; we never destroy order).
- **Recognition fold grows one class.** `ElementwiseFoldClass` gains `AssociativeFoldChain { combine, evidence }` beside `PureElementwiseChain`; recognized → a tree-reduce/scan kernel plan whose depth/width derive from the target domain's `lane_count`; `NotWitnessed` → typed refusal (the fold stays serial — a correct schedule at any lane count, merely priced with its idle-lane cost so the displaced value of *witnessing* the law becomes visible). `DataDependentGather` stays `Refused` — the honest wall, unchanged.
- **Float reassociation is a declared contract, reusing the FMA machinery verbatim.** Tree-reducing a float fold reassociates — not bit-exact against the serial oracle. The demo already models exactly this class: FMA-permitted is declared `Lossy` and proven non-bit-exact; FMA-refused + insist → `FmaContractionViolatesContract` (`accelerator_demo_realize.dag:393-435`). Reduction-order relaxation is one more `NumericalContract` axis: permitted → `Lossy` with differential bound; refused → the fold is not accelerator-schedulable, typed refusal. Int/Nat folds tree-reduce `BitExact`.
- **Placement is priced, not preferred.** A step's domain is chosen by comparing, per candidate domain in the machine graph, `edge_crossing_cost + roofline_time(domain)` over evidence-carrying quantities — the operand's current domain is just one candidate whose crossing cost is zero. Any `UnknownQuantity` in the comparison → stay at the operand's current domain with a counted `PlacementUndecidable{missing}` diagnostic. No "prefer GPU" flag exists; no domain-kind is consulted (none exists to consult).
- **Realization handlers are the existing seams**: simd contiguous-loop (live), WGSL kernel (dormant, `feature:dag-gpu-realization-handler` — waking it is this phase's optional closer, operator-gated). Differential witness against the scalar oracle stays the acceptance instrument.

## 7. The orthogonality proof (the DESIGN §7 deliverable)

One derivation, **three fills on one axis**, by execution: run the same computation graph (the demo's fused elementwise chain + one witnessed-associative reduction) through the one scheduling kernel against three `MachineShape` fills that differ only in facts — scalar domain (`lane_count: 1`, Ampere core), SIMD domain (`lane_count: ~8-16`, same tower), many-lane domain behind a `LinkEdge` (RTX 5090 fill, `lane_count: 21760`):

- the derived schedules are, respectively: serial fold / shallow tree-reduce / deep tree-reduce with priced edge-crossing — a **monotone response to `lane_count`**, with graph-independence parallelism and affinity batching identical across all three (those derive from the computation graph and memory tower, not the lanes);
- a fourth fill — same many-lane domain joined by `SharedLevelEdge` instead (the `apple_m5` topology) — prices crossing at zero and the placement flips accordingly, *with no code path knowing it is "integrated"*;
- all fills produce differential-equivalent results under their declared contracts;
- **discriminating REDs**: (i) strip the associativity witness → every fill degrades to the serial fold (never fabricates a parallel plan), and the idle-lane cost term makes the loss visible at high lane counts; (ii) an all-`UnknownQuantity` shape → still a *legal* schedule, no affinity/placement claims (the abstract shape suffices for correctness, not for pricing — by construction, visibly); (iii) the kind-fork RED is discharged by construction — there is no discriminant to match on, so the forked-scheduler state is unwritable rather than tested-for.

That set of REDs *is* the orthogonality claim made executable: the schedule is a function of machine facts — monotone where the facts are ordered — and every cross-term is either derived or a typed refusal.

## 8. Non-goals and standing walls

- No interpreter parallelization (Rc→Arc gate removed, operator-signed — parallelism stays at the plan/Runnable grain).
- No hash-partition-by-key primitive; sharding remains "an independent region of the DependencyView" (execution-spine-design §5).
- No heuristic knobs, thresholds, or authored magnitude literals anywhere (2026-07-12 ruling, generalized).
- `schedule_eq` invariant preserved: affinity is derived topology, never a per-host measured tweak.
- No new size walk: all footprints derive through `node_keyed_graph_transitive_bytes`.

## 8. Open questions for operator sign-off

1. Home + name for the machine-shape authority (`std.machine_shape`?) and whether `Quantified<T>` should subsume `CostBasis` (§3: they smell like one evidence concept — but that convergence touches every `CostAccount` consumer, so it needs its own runway).
2. The `LocalityTier` unification direction: extend `cache_interface`'s lattice downward vs lift the lattice out to std and have `cache_interface` import it (I lean lift-out — the lattice is not cache-specific).
3. `Energy` vs `Watt` (DESIGN.md §2 divergence) — add the quantity or amend the doc?
4. Whether Phase 4's law carrier should wait for the enforcement-intent lane (`StandingIntent` ⇄ `LensContract`) since "every combine inhabits Monoid" is precisely a standing intent — or land as a local carrier first and enroll later.
5. Sequencing vs the namespace/SymbolIndex lane: OperandFlow derivation wants the containment SymbolIndex for cheap whole-tree walks; Phase 2 may be gated the same way the general body producer is.
6. Fate of `Placement::LocalAccelerator`: under the no-kind rule it reads as a smuggled machine kind. Options: (a) keep as a coarse *locality tier* (defensible — it names a position, in-process vs across-a-link, not a device kind) or (b) dissolve into a domain-reference once `MachineShape.domains` exists. Leaning (b) eventually with (a) as the interim reading; either way the demo's stamp is data, so migration is cheap. Operator call — it is a std type.
7. `LaneGrouping` staging: lockstep structure (warp/SIMD masking vs independent lanes) is what makes control *divergence* a cost. Faithful modeling wants it eventually; does Phase 1 carry the grouping axis as structure-with-`UnknownQuantity` semantics, or defer the axis entirely until a divergent-control workload exists to price (§6 — no displaced cost yet, the demo class is divergence-free by construction)?
8. Multi-domain scope in Phase 1: the domain-graph shape above is cheap to declare, but the *scheduler* consuming multiple domains is Phase-4 work. Confirm Phase 1 lands the graph type with single-domain fills only (the tower + one domain), so no phase ships machinery without a consumer.
