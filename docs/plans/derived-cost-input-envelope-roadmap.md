# Compute fabric: derived cost from a declared input envelope — roadmap

Status: DRAFT scoping note (sharp-stag-782, 2026-06-25). Not yet grounded into a `.dag` Plan
authority — §6 obligation noted at the end. Co-design with calm-carp-204 (CI profiling / supply).

## Thesis (one line)

The compute fabric offers compute as a **derived, deterministic product**: cost is *derived*
from program structure × a **declared input envelope**, never measured; measurement is confined
to the host's physical constants and to falsifying the derivation. CI is the first instance —
it declares its own corpus envelope and derives its own resource request.

## Why (the axiom chain)

- §4: execution is bounded-and-forward; recursion is sugar over `Loop` admissible only with
  `DescentEvidence` (bottom = fail-closed). An **unbounded input is the data-side of a loop with
  no descent evidence** — a §5 fail-open by construction. Physical memory is finite, so a bound
  always exists; refusing to declare it is the "never" trap (§5).
- §5: correctness by construction, not validation. A *measured* cost is empirical validation; a
  *derived* cost is construction. Measuring where you can derive is the fail-open smell, and it
  usually means an **unmodeled dependency** (the input) papered over instead of named.
- §3/§2: `MemoryRequirement.min_bytes` (compute_fabric.dag:456) is a stored number stated
  independently of the input that determines it — an unmodeled-dependency / parallel-representation
  that goes stale. Derive it; the stale state becomes unwritable.
- §1: reduce convention to necessity. The unbounded input is convention; the finite-memory bound
  is necessity. The only irreducible empirical residue is the host's physical constants (allocator
  size-classes, page granularity, `Value` overhead) — physics, §1's limit.

## The two lease modes (how the fabric offers compute)

Grounded on the real concept (k8s QoS / requests-vs-limits / reserved-vs-spot — §3, cite, do not
nickname), NOT minted fresh:

- **Reserved** (request == limit): requester's demand declares its input envelope; fabric derives
  cost via the symbolic fold, reserves exactly, admission-controls (reject beyond envelope,
  fail-closed), **guarantees a deterministic product**. `PerformanceReceipt.confidence` (variance)
  proves delivery; high variance = the lease is lying (fail-closed signal).
- **BestEffort** (request < limit): "use up to this host"; fabric fills to capacity, scheduler
  discovers width from live pressure; throughput-max, no guarantee. This is the *current*
  `memory_aware_spawn_width` path and calm-carp's fill-to-capacity work. CI floor uses this AND
  donates the physical-constant calibration the Reserved mode needs.

Determinism control is a separate, orthogonal axis: **performance isolation** `Shared | Exclusive`
(exclusive cores / no co-tenant), distinct from the existing `IsolationBoundary` (namespace only).
A Reserved lease on a Shared host can still be slowed by contention; Exclusive removes the variable.

## Phases

### P0 — Re-role the keystone (with calm-carp, IN FLIGHT — needs the role correction NOW)
- Keystone `PerformanceReceipt.cost: CostAccount<Nano>` fold proceeds (calm-carp authoring).
- BUT its role is **demoted**: measurement = (a) host physical-constant calibration, once per host,
  and (b) the §5 falsifier `measured == derived`. NOT the budget authority.
- `CostBasis = Predicted | Measured` (already on CostAccount) carries the derived-vs-measured split.
- Action: tell calm-carp before their worker wires budget := measured. (sent.)

### P1 — Input envelope as a first-class fact (UP-FRONT; operator wants this first)
- Model `InputEnvelope` — the data-side of `DescentEvidence`. Shape ~
  `Bounded { axis: SizeAxis, max: Magnitude } | EnvelopeUnknown` (bottom = fail-closed).
- Attach to `WorkDemand` (the demand declares what it is specified to serve).
- Fail-closed admission: actual input exceeding the envelope → typed, located refusal (a wall),
  never a silent OOM/swap.
- **First instance, modeled using our own CI**: the CI corpus envelope — roster/witness count, max
  per-file node count, max tree size. Derived from the already-finite discovered roster
  (`gunbc.ci_layer_roots` / marker-driven discovery). This is the bound CI already implicitly has;
  P1 writes it down.
- Witness: an input at the envelope admits; one node over → fail-closed refusal (RED-on-revert).

### P2 — Symbolic cost derivation (depends on P1 + substrate vocab that already exists)
- Inhabit `SymbolicCost = Node` (compute_fabric.dag:87) and `CostShape` (algebra.dag:185,
  ShapeConstant/LinearScan/IterateBody/SortBody) — derive per-node cost shape from the graph.
- The peak-live-set fold: liveness (when a value frees) × `ParallelismShape`
  (IndependentShards → sum; sequential → max) × per-node `CostShape` → a symbolic cost function C(n).
- Evaluate C at the envelope (worst-case / Reserved guarantee) or at the actual input size (tight,
  known for batch CI). Soundness from the bound, tightness from the actual.
- Honesty: a *tight* bound is a §5/§6 ratchet; start at CostShape granularity, sound
  over-approximation always available (never under-reserves → no OOM), calibration tightens it.

### P3 — Dissolve `min_bytes` into the derivation (depends on P1+P2)
- `MemoryRequirement.min_bytes` becomes derived from `InputEnvelope × symbolic_cost`, not a stored
  field. The stored-vs-derived disagreement becomes unwritable (§5 construction).
- This dissolves the §3 unmodeled-dependency fork directly.

### P4 — Lease modes + performance isolation (parallel-designable)
- `LeaseMode = Reserved | BestEffort` on the offer/lease; `satisfies()` matches it.
- `PerformanceIsolation = Shared | Exclusive` axis; `satisfies()` enforces Exclusive for Reserved
  determinism requests.
- Ground on k8s QoS literature (§3).

### P5 — Swap policy + physical-constant calibration (independent, early-startable)
- `SwapPolicy = Disabled | ...` host fact: swap = fail-open (silent slow product); prefer OFF, OOM
  as the loud error, `oomd` as eviction authority (REGIME-2 already moving this way). Costs density
  (no swap cushion → more headroom reserved) — modeled, not implicit.
- Host physical constants (allocator/page/Value-overhead) calibrated once — the ONLY legitimate
  measurement. calm-carp's measurement work re-homes here + P0(b) falsifier.

### P6 — Determinism witness + budget-tree consumer (my carrier; depends on P2–P5)
- `ci_budget_tree` reserves *derived* requests (P2/P3); co-residence packs them.
- Witness: variance across measurements at fixed context (PerformanceReceipt.confidence) =
  delivered determinism; perturb cache_state / co-tenant → variance must move (load-bearing tag).
- Preserve the §3-deliberate granularity split (within-run per-shard vs inter-run whole-run,
  `gunbc_ci_footprint_granularity_disposition` Terminal) — do not collapse.

## Dependencies (critical path)

P0 (keystone, in flight) → P1 (envelope, up-front, mostly independent) → P2 (symbolic cost) → P3
(dissolve min_bytes). P4 (lease modes) and P5 (swap/calibration) parallel-startable. P6 (my budget
consumer) integrates P2/P3 + P5. calm-carp's Nodes A–D re-role: their measurement → P0(b)+P5; their
fill-to-capacity → P4 BestEffort. Nothing wasted, roles clarified.

## Division of labor (proposed)
- **Me (sharp-stag-782):** P1 (input envelope + CI corpus instance), P2 (symbolic cost fold),
  P3 (dissolve min_bytes), P6 (budget consumer). The derivation / general-fabric half.
- **calm-carp-204:** P0 keystone, P5 calibration + falsifier, P4 BestEffort fill-to-capacity
  (their existing scheduler/throughput lane, re-roled).
- **neat-board track:** P4/P5 host facts (srv1/2/3 ComputeHost offers, SwapPolicy) — coordinate
  the boundary so host facts aren't forked.

## Risks / open
- Tight symbolic bounds hard in general — mitigated by sound-loose-then-calibrate (P2 honesty).
- Streaming/external input — declare a bounded window, fail-closed beyond (still P1, no exception).
- §6 grounding: this roadmap must ground into a `.dag` Plan authority before work lands (mirror
  the host-effect plan grounding, #5764). Scoping-stage markdown is the interim.
