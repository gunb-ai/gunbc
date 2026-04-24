# Substrate Carrier Port Program

**Status:** Execution started. Lane E-T staged 2026-04-23 in PR #682; Lane E-C staged on an E-T-stacked branch; remaining lanes stay queued in dependency order.
**Scope:** Program-of-work scoping for porting v2's `DescentEvidence` / `CallPattern` / `SubValueRelation` / `MethodSemantics` carrier families into the v3 substrate so that v3 lenses over termination, cost, and method dispatch can reach `BEHAVIORALLY COMPLETE` per `docs/v3-lens-capability-register.md`.
**Remaining scope:** no lens migrations in E-T; remaining carrier families stay separate lane placeholders below.

## 1. Why this document exists

The 2026-04-21 v3 lens capability audit (`docs/v3-lens-capability-register.md`) identified a common root cause for four lenses sitting at `PROXY` / `STUB`: v3's substrate (`src/v3/std/substrate.dag`) does not carry the termination/induction/computation facts that v2's analyses consume. v2 imports these from `dsl/std/termination.dag`, `dsl/std/induction.dag`, `dsl/std/computation.dag`, and `src/v2/00_core.dag` (`MethodSemantics`); v3 has none of them on the reflected substrate and therefore cannot derive symbolic bounds or dispatch over typed method semantics.

`docs/v3-lens-capability-register.md § Common root causes` names this bundle as "a program of work, probably bigger than all of Lane 2 as currently scoped." Before dispatching execution lanes, we need per-carrier shape analysis, dependency ordering, and a receipt-producing lane decomposition. This doc provides that.

The audit also flagged P4 Decidability as "mostly unchanged" by the 2026-04-21 wave — termination/induction facts are exactly what a decidability lens would consume. Completing this port unblocks P4 progress too.

## 2. What lives in the v2 carriers today

Full inventories (names only — read source files for shapes):

**`dsl/std/termination.dag` (344L)** — proof-theory for well-founded descent.
- `DescentEvidence = Strict | NonIncreasing | DescentUnknown` (+ lattice fns `merge_evidence`, `join_evidence`, `promote_to_strict`, `evidence_rank`, `optional_evidence_meet`, `map_evidence_merge_at`).
- `RankingDimension = TreeSize | ListLength | ArithmeticValue | TokenPosition | SetCardinality` (each wraps `param: String`).
- `DescentSource = ChildAccessor | ListShrink | ArithmeticDecrease | ParserAdvance | SetRemoval | FoldIteration`.
- `TerminationProof { dimensions: List<RankingDimension> }`, `ProofEdge { caller, callee, evidence: List<DescentEvidence> }`.

**`dsl/std/computation.dag` (384L)** — syntax-to-primitive lowering table.
- `SizeBound = CollectionSize | TreeSize | ArithmeticParam | ExplicitCount | Forever`.
- `CallPattern = ChildAccessorCall | CollectionShrinkCall | ArithmeticDescentCall | ParserAdvanceCall | WorklistDrainCall | FoldBodyCall | SameArgumentCall`.
- `ShrinkFactor = UnitShrink | ConstantShrink | ProportionalShrink`.
- `IterationPrimitive = Fold | Descend | Repeat`, `LoweringTarget { primitive, bound, evidence, factor }`.
- `IterationDimension = TreeDescent | CollectionFold | ArithmeticRepeat`.
- Fns: `lower_call_pattern`, `size_bound_param`, `is_constant_bound`, `constant_bound_value`, `algebra_profile_to_dimension`, `type_iteration_dimension`.

**`dsl/std/induction.dag` (708L)** — type-level inductive structure and cost bounds.
- `RecursionShape = DirectRecursion | ListRecursion | OptionalRecursion | SetRecursion | MapValueRecursion`.
- `InductiveField { type_name, variant_name, field_name, shape, element_type }`.
- `SubValueRelation = StrictSubValue | IteratedSubValue | ArithmeticDescent | PreservedValue | SubValueUnknown` (+ lattice fns `meet_sub_value`, `join_sub_value`, `compose_sub_value`, `compose_sub_value_relations`, plus projectors to `DescentEvidence` / `CallPattern` / `LoweringTarget`).
- Cost algebra: `PolynomialExponent`, `AtomicCost`, `CostBound = ConstantBound | AtomicBound | ProductBound | SumBound | MaxBound`.
- Master-theorem machinery: `RecurrenceForm`, `master_theorem`, plus named bounds (`bfs_dfs_bound`, `dijkstra_bound`, `mergesort_bound`, …).

**`src/v2/00_core.dag` (compiler-side)** — method dispatch metadata.
- `MethodSemantics = PlainMethodSemantics | AlgebraMethodSemantics { method_def, fold_accumulator_type, size_effect, cost_shape, algebra_template } | ServiceMethodSemantics { service_name, op_params }`.
- Attached to `ExprMethodCall { method_semantics: MethodSemantics? }`; consumed by `src/v2/04_lookup.dag` and complexity.
- Transitive carriers: `CollectionSizeEffect`, `CostShape`, `AlgebraFieldTemplate` — already in `dsl/std/algebra.dag:408`, `:418`, `:424` (not compiler-internal). Only `MethodSemantics` itself is compiler-internal.

## 3. Per-carrier analysis

For each family: **Shape** (sum/product/record/constraint) · **Consumers** (v2 today → which v3 lens promotions it unlocks) · **Dependencies** (other carriers / v3 substrate that must port together) · **Blockers** (port vs capability) · **Lane size estimate**.

### 3.1 Family T — `DescentEvidence` + proof structure

- **Shape.** `DescentEvidence` is a flat 3-variant coproduct. `RankingDimension` is a 5-variant coproduct each carrying `param: String`. `TerminationProof` and `ProofEdge` are records. `DescentSource` is a 6-variant coproduct. All forms are decidable, bounded, already phrased as pure algebra with a named lattice structure (`BoundedLattice<DescentEvidence>` — meet/join pair).
- **Consumers.** v2: complexity.dag proof construction, cost composition, the termination checker (`std.graph.is_valid_proof`). v3 promotions unlocked: `complexity.dag` PROXY → COMPLETE (partial — also needs family I/C), `cost.dag` PROXY → COMPLETE (partial — needs family I for `SubValueRelation`).
- **Dependencies.** Internal-only: `DescentEvidence` requires `Ordering` from `std.algebra` (already v3-reachable as `dsl/std/algebra.dag`). `RankingDimension.param: String` is a bootstrap-constraint bridge — file header explicitly says "When .dag supports function references, these should become structural." Port with `String` for now; do not widen scope to "teach substrate function refs" in this lane.
- **Blockers.** None structural. This family is the cleanest port — it's pure data + pure lattice fns. Port target is either `src/v3/std/termination.dag` (mirroring v3 std layout) or direct consumption of `dsl/std/termination.dag` from v3 lenses once the v3 grammar subset covers it. The file-preference-rank scaffold (ROADMAP) means `src/v3/std/*` vs `dsl/std/*` is currently a routing call, not a shape decision.
- **Lane size.** S. Mostly declarative — carriers + lattice fns. No substrate-level additions required.

### 3.2 Family C — `CallPattern` + lowering (`computation.dag`)

- **Shape.** `CallPattern` is an 8-variant flat coproduct; amount-carrying variants reuse the shared `std.types::PositiveInt` and `std.termination::DivisionDescentFactor` authorities instead of raw `Int`, and the arithmetic cases split into `ArithmeticSubtractCall { by: PositiveInt }` / `ArithmeticDivideCall { by: DivisionDescentFactor }` (mirroring `DescentSource::ArithmeticSubtract` / `ArithmeticDivide`) so zero / negative / divide-by-one witnesses are unrepresentable and no String `op` tag is authored. `SizeBound` 5-variant flat. `IterationPrimitive` 3-variant pure enum. `LoweringTarget` is a record over the above plus `DescentEvidence` + `ShrinkFactor?` (with `ConstantShrink { amount: PositiveInt }` / `ProportionalShrink { divisor: DivisionDescentFactor }`). `IterationDimension` 3-variant pure enum. `lower_call_pattern` is a pure total function — one match, no recursion.
- **Consumers.** v2: complexity/cost derivation of asymptotic class. v3 promotions: `cost.dag` gains the "which primitive, which bound" fact for each self-call; pairs with family T to give `complexity.dag` the recurrence form.
- **Dependencies.** `CallPattern` depends on nothing beyond its own file. `LoweringTarget` depends on family T (`DescentEvidence`). `ShrinkFactor` currently lives in `computation.dag` but is used by family I — see `induction.dag:147` comment explaining the move to break a circular import. Keep the placement (`ShrinkFactor` stays in computation).
- **Blockers.** None structural. Port target same routing call as family T.
- **Lane size.** S. Port + one pure total fn (`lower_call_pattern`). The real value unlocks only when families T and I are both present.

### 3.3 Family I — `SubValueRelation` + inductive fields + cost bounds

- **Shape.** Three tiers.
  - *Tier 1 (structural):* `RecursionShape` 5-variant enum; `InductiveField` record; `SubValueRelation` 5-variant coproduct with nested `InductiveField` / `ShrinkFactor` payloads.
  - *Tier 2 (lattice):* `meet_sub_value` / `join_sub_value` / `compose_sub_value*` — five functions, each pure, each a total match.
  - *Tier 3 (cost algebra):* `CostBound` 5-variant recursive-in-product-slots sum (`ProductBound`, `SumBound`, `MaxBound` carry `List<CostBound>`), `AtomicCost`, `PolynomialExponent`, `RecurrenceForm`, plus `master_theorem` and ~15 named bound constructors.
- **Consumers.** v2: complexity.dag cost derivation, the named-bound library for analysis receipts. v3 promotions: this is the tier that gets `cost.dag` and `complexity.dag` from PROXY to COMPLETE — v2's symbolic `CostExpr` / `SizeExpr` / work-span / asymptotic class are all phrased in terms of `CostBound` and `SubValueRelation`.
- **Dependencies.** Family T (`DescentEvidence`, `RankingDimension`), family C (`ShrinkFactor`, `CallPattern`, `LoweringTarget`). Both must land first.
- **Blockers — potential substrate additions.** `CostBound`'s `ProductBound { terms: List<CostBound> }` / `SumBound` / `MaxBound` are self-referential through `List<_>`. This is the same structural pattern as `FieldValue` in `src/v3/std/substrate.dag:68-76` (`Record(List<FieldEntry>)`, `List(List<FieldValue>)`), which v3 already supports. **Verification task in the execution lane:** confirm v3's current grammar subset (post-SG-4b, post-1e) can parse and lower `CostBound`. If yes → pure port. If no → surface as capability-gap, separate blocker lane. Today's evidence (`FieldValue` lowers and emits) says it should work but has not been proven on these specific carriers.
  `master_theorem` uses `int_pow_bounded` / `ceil_log` — decidability-safe bounded recursion. Confirm the v3 pipeline accepts the shape.
- **Lane size.** M. Largest of the three, both by LOC and by verification surface.

### 3.4 Family M — `MethodSemantics`

- **Shape.** 3-variant coproduct. Variants carry `Node?`, `String`, `List<Node>`, `CollectionSizeEffect?`, `CostShape?`, `AlgebraFieldTemplate?`. Transitive carriers (`CollectionSizeEffect`, `CostShape`, `AlgebraFieldTemplate`) are already `std/`-grade (`dsl/std/algebra.dag:409-430`); only `MethodSemantics` itself is compiler-internal (`src/v2/00_core.dag:168`).
- **Consumers.** v2: `04_lookup.dag` populates `MethodSemantics` on `ExprMethodCall`; complexity and emit consume it. v3 today: no `ExprMethodCall` and no `method_semantics` field exists — method-like calls lower to `TransformTarget::Callable` (for function-style dispatch) or `TransformTarget::FieldProject` (for field access), and resolution runs structurally over the substrate declaration graph. A v3 `MethodSemantics` port would sit alongside that path, not replace a Rust-side bridge.
- **Dependencies.** Two options — and with the corrected carrier locations, these are no longer symmetric:
  - **(M-a) Routine port.** `CollectionSizeEffect` / `CostShape` / `AlgebraFieldTemplate` port alongside `MethodSemantics` from `dsl/std/algebra.dag` (they're already there). Scope is ~4 carriers + the `ExprMethodCall` attachment point. Smaller than family I.
  - **(M-b) Structural subsumption.** v3's structural-resolution model (`TransformTarget`, typed transforms) already replaces `MethodSemantics` — no carrier crosses over; the register gains a footnote explaining the subsumption. Needs a design call, not a port.
- **Blockers.** The (M-a vs M-b) call is the only genuine design question — if v3's structural resolution already carries the equivalent facts, M-a is pure duplication and M-b is correct. If it doesn't, M-a is a routine port.
- **Lane size.** S–M (M-a path) or design-only (M-b path). Queue after T/C/I so v3 evidence informs the call.

## 4. Port order

Dependencies dictate a strict order:

1. **T** (`DescentEvidence` + proof) — no deps, unblocks C and I.
2. **C** (`CallPattern` + lowering) — depends on T, unblocks I and the first partial lens promotion.
3. **I** (`SubValueRelation` + cost) — depends on T and C. Ports the carrier *types*. On its own does not promote any lens row: the types are inert until a producer attaches them to call sites.
4. **P** (per-call descent-evidence provenance) — depends on T, C, I. Decides where/how v3 produces the per-call witness v2 stores on `ExprCall.descent_evidence`. Carrier parity for `cost.dag` / `complexity.dag` on the non-method-dispatch slice closes here, not at E-I. See §6 Lane E-P.
5. **M** (`MethodSemantics`) — **design-decision lane**, not a port lane. Run after T/C/I/P so v3's evidence ("does structural resolution subsume method semantics?") informs the shape. M closing clears the **carrier-parity** portion of `cost.dag` / `complexity.dag`'s drop-lists. Full `BEHAVIORALLY COMPLETE` for `cost.dag` also requires the non-carrier blockers the register records (`Dimension<SymbolicCost>` wiring on grammar/data-body gaps; named `SizeVariable` value semantics) — those are grammar/emit work, outside this program.

**Shortest path to first Band C receipt:** T + C land → `cost.dag` can adopt `CallPattern` + `LoweringTarget` as its carrier vocabulary. I + P close carrier parity on the non-method-dispatch slice (types + producer both present). M closes carrier parity on the method-dispatch slice. Full `BEHAVIORALLY COMPLETE` for `cost.dag` closes separately on the grammar/emit side (`Dimension<SymbolicCost>`, named `SizeVariable`).

## 5. Success per carrier

Each lane closes with a receipt per `docs/v3-lens-capability-register.md § Discipline`:

| Lane | Register rows that move | Behavioral axis | Cementing test |
|---|---|---|---|
| T | — (no direct lens depends on T alone) | `complexity.dag` / `cost.dag` partial progress | Staged 2026-04-23 in PR #682: `src/v3/std/termination.dag` bootstraps `DescentEvidence`, `RankingDimension`, `DescentSource`, `TerminationProof`, `ProofEdge`, and lattice helpers; `m2_substrate_inhabitance_test` covers bootstrap shape + lattice mirror behavior. |
| C | `cost.dag` begins PROXY → partial | PROXY still, but "What v2 has that v3 drops" column shrinks | Staged on E-T branch: `src/v3/std/computation.dag` bootstraps `SizeBound`, `CallPattern`, `ShrinkFactor`, `IterationPrimitive`, `LoweringTarget`, `IterationDimension`, and lowering helpers; `m2_substrate_inhabitance_test` covers bootstrap shape + `lower_call_pattern` / bound-helper behavior. Full cost/complexity parity still waits on E-I/E-P producers. |
| I | types ported; no lens row moves yet — producer still missing | PROXY still (pending E-P) | carrier round-trip tests only |
| P | `complexity.dag` / `cost.dag` advance on the non-method-dispatch slice; "What v2 has that v3 drops" column shrinks to `MethodSemantics` + grammar/emit items | PROXY still (pending E-M for method-dispatch) | v2-oracle-vs-v3 per-call descent-evidence golden for non-method-dispatch inputs |
| M | either a ported `MethodSemantics` surface (M-a) or a register footnote explaining v3's structural subsumption (M-b). Closes carrier-parity for `complexity.dag` / `cost.dag`. Full `BEHAVIORALLY COMPLETE` still requires `cost.dag`'s separate non-carrier blockers to clear (`Dimension<SymbolicCost>` wiring deferred on grammar/data-body gaps per the register; named `SizeVariable` with v2's value semantics). Those live outside this program. | carrier-parity column goes to N/A; the remaining `cost.dag` drop-list lives on after E-M | design receipt (M-b) or cementing test covering method-dispatch (M-a) |

`idempotency.dag` (STUB) and `parallelism.dag` (STUB) are **not** unblocked by this program — their blocker is the emit-side gap (match-on-user-sums), tracked separately in ROADMAP's receipt-closure wave.

## 6. Lane placeholders

Each placeholder below gets promoted to a full brief when dispatched. Stop-signals apply per `feedback_checkpoint_dissolution_default`: any carrier found to require new L1 behavior / type connective is a C1 stop and escalates before the port lands.

### Lane E-T — Port `DescentEvidence` + proof structure `(S)`

- **Work:** port `DescentEvidence`, `RankingDimension`, `DescentSource`, `TerminationProof`, `ProofEdge` + lattice fns into v3-reachable `std/termination.dag` (routing per file-preference-rank decision). Preserve `String` bootstrap-constraint fields; do not widen to structural refs.
- **Acceptance:** carriers parse, lower, emit in v3. Lattice-fn tests ported. Port-progress receipt recorded in this doc's §5 table (not in the lens capability register — that register is lens-only per its own contract; carrier-port receipts stay here).
- **STOP-AND-ESCALATE:** any carrier requires substrate connective not already present → C1 lane.

### Lane E-C — Port `CallPattern` + lowering `(S)`

- **Work:** port `SizeBound`, `CallPattern`, `ShrinkFactor`, `IterationPrimitive`, `LoweringTarget`, `IterationDimension`, `lower_call_pattern`, plus the helpers (`size_bound_param`, `is_constant_bound`, `constant_bound_value`, `algebra_profile_to_dimension`, `type_iteration_dimension`). Requires E-T landed.
- **Acceptance:** `lower_call_pattern` is the v3 lowering authority. `cost.dag` can begin consuming `CallPattern` (partial progress recorded in register).
- **STOP-AND-ESCALATE:** profile-lookup (`kernel_algebra_profile`) has a v3 gap → surface, don't paper over.

### Lane E-I — Port `SubValueRelation` + inductive fields + cost algebra `(M)`

- **Work:** port Tier 1 (structural carriers) + Tier 2 (lattice fns) + Tier 3 (cost algebra). Requires E-T and E-C landed.
- **Pre-flight verification (first step of the lane):** prove `CostBound`'s self-referential-through-`List` shape lowers/emits on v3 today (mirrors `FieldValue`). If it does not → stop, scope a substrate-capability lane, don't hack the carrier.
- **Acceptance:** carrier types (`SubValueRelation`, `InductiveField`, `RecursionShape`, `CostBound` + supporting algebra) parse/lower/emit in v3 with round-trip tests. **No lens row moves on E-I alone** — without E-P's producer, `cost.dag` / `complexity.dag` cannot read per-call `SubValueRelation` from v3 and their drop-lists do not shrink. The v2 complexity lens depends on `ExprCall.descent_evidence` (`src/v2/00_core.dag:199`), which is an E-P deliverable, not an E-I deliverable. Drop-list shrinkage, lens advance on the non-method-dispatch slice, and carrier parity all land at E-P. Full `BEHAVIORALLY COMPLETE` also depends on E-M (method-dispatch) and the separate grammar/emit non-carrier blockers.
- **STOP-AND-ESCALATE:** master-theorem machinery doesn't lower → surface as decidability/emit gap.

### Lane E-P — Per-call descent-evidence provenance `(M)`

- **Context:** v2 stores per-call witness facts as `descent_evidence: List<SubValueRelation>?` directly on `ExprCall` (`src/v2/00_core.dag:199`), populated by the v2 complexity/lookup passes. v3's call-site node (`TransformNode`, `src/v3/std/substrate.dag:264-270`) has no analogous attachment and v3 has no pass that produces the evidence. Porting the carrier *types* (E-I) without scoping *where v3 produces and stores/derives the per-call witnesses* is a modeling hole — the types are inert vocabulary without a producer.
- **Work:** decide the attachment shape and producer. Three options to scope:
  - **(P-a) On-substrate attachment.** Add `descent_evidence: List<SubValueRelation>?` (or equivalent) to `TransformNode`. Substrate change; parallels v2's shape. Requires a pass that populates it.
  - **(P-b) Lens-derived.** A lens walks calls and computes `SubValueRelation` on demand from argument structure + declared inductive fields. No substrate change; evidence lives in the lens result.
  - **(P-c) Side-table.** A `Map<NodeId, List<SubValueRelation>>` installed by a dedicated pass alongside `lane2_workflow_at`-style reflected accessors. Intermediate between (a) and (b).
- **Decision gate:** pick one based on how `cost.dag` / `complexity.dag` want to consume the facts. Option (b) aligns with v3's "analyses are lenses" principle; option (a) aligns with v2's shape. Option (c) keeps the substrate minimal while giving the evidence a named home.
- **Acceptance:** a v3 call in a test fixture produces per-call `SubValueRelation` readable by a lens, verified by a cementing test that compares against v2's `expr_call_descent_evidence` oracle on the same input.
- **Dependencies:** E-T, E-C, E-I (carriers must exist before a producer can emit them). Runs **concurrently-with or immediately-after** E-I; E-I's carrier-parity acceptance is not met until E-P lands, because `cost.dag` cannot consume evidence that does not exist at call sites.
- **STOP-AND-ESCALATE:** option (a) requires new substrate connective → C1 lane; option (b) requires a lens capability v3 does not yet have → surface emit gap; any option reveals v3's `TransformTarget` distinctions collapse information v2's `ExprCall` preserved → modeling discovery, escalate.

### Lane E-M — `MethodSemantics` port-or-subsume `(S–M)`

- **Work:** decide M-a vs M-b based on whether v3's structural-resolution model (`TransformTarget::Callable` + `FieldProject` + typed transforms) already carries the facts `MethodSemantics` carries in v2. If M-a: port `MethodSemantics` + its three `dsl/std/algebra.dag` transitive carriers into a v3-reachable module, plus the `ExprMethodCall` attachment point (~4 carriers, routine). If M-b: add a footnote row to the register explaining the structural subsumption, no port. Queue after E-T/E-C/E-I land so v3 evidence informs the call.
- **Acceptance:** either a ported `MethodSemantics` surface with a cementing test over method-dispatch inputs (M-a), or a register footnote + ROADMAP update receipt explaining the structural subsumption (M-b). Either way, the **carrier-parity** portion of the "What v2 has that v3 drops" column clears for `cost.dag` and `complexity.dag`. Full `BEHAVIORALLY COMPLETE` for `cost.dag` also requires the separate non-carrier blockers the register already records (`Dimension<SymbolicCost>` wiring deferred on grammar/data-body gaps; named `SizeVariable` with v2's value semantics) — those are tracked elsewhere (grammar/emit work) and are **not** in scope for this program.
- **STOP-AND-ESCALATE:** v3 structural resolution turns out to carry *some* of the facts but not all → hybrid shape, escalate before papering over.

## 6a. Per-method metadata — related carrier question

Surfaced by Lane G (PR #654) during `algebra.dag` template-vs-declaration reconciliation: the `*_templates()` functions in `dsl/std/algebra.dag` (`ordered_ring_templates()`, `partial_function_templates()`, etc.) are **not** redundant with their type declarations. They carry per-method metadata — `size_effect`, `cost_shape`, `callback_element_position` — that lenses (complexity, cost) read. The type declarations have no field-level slot for this metadata; attaching it would require substrate extensions.

This is the same shape as the core port program at a different layer: **per-method contract metadata consumed by the same lenses as `DescentEvidence` / `CallPattern` / `SubValueRelation` / `MethodSemantics`**. Not included in the core four because the carriers above are v2-authored structural facts that already exist; the metadata question is about where per-method contract data *should* live structurally. Worth scoping here so a port-program execution doesn't rediscover it mid-flight.

**Four options (one of which is "keep lens-local"):**

0. **Metadata stays in lens-local lookup tables.** Nothing moves. `*_templates()` (or its successor) stays as the authoritative per-method metadata surface; lenses consume it directly. No substrate change, no std/ change. Valid if the E-I evidence shows the metadata is cleanly separable from the cost/descent carriers — then co-locating isn't a modeling win.
1. **Extend type declarations with method-metadata annotations.** Needs substrate support for field-level refinements (currently partial per DB-11). Largest substrate change; smallest std/ surface change.
2. **Separate metadata carrier per algebra.** Declare `OrderedRingMetadata`, `PartialFunctionMetadata`, etc. paired with the type declarations. No substrate change; additional std/ surface (one carrier per algebra).
3. **Unified `MethodContract` carrier.** One generic carrier indexed by `(algebra_id, method_id)` that lenses query by id. No substrate change; minimal std/ surface growth (one carrier total). Closest in shape to a `TemplateArgumentBinding`-style lookup.

**Current consumers:** `ordered_ring_templates()`, `partial_function_templates()` and their siblings in `dsl/std/algebra.dag:447-569`; the complexity / cost analyses that read `size_effect` / `cost_shape` / `callback_element_position` off the returned templates.

**Recommendation (deferred):** this scoping doc does not pick a direction. The call interacts with family I (`SubValueRelation` + cost) — a lens that consumes both per-method metadata and `CostBound` may want them co-located, which favors option 3. But if lane E-I's pre-flight evidence shows the metadata is cleanly separable from cost/descent, option 0 ("keep lens-local") is also live. Defer the pick until that evidence lands.

**Cross-reference:** see PR #654's investigation receipt for the full Lane G findings that surfaced this question.

**Does not block the core four-carrier port program.** Queued as a design follow-up once lane E-I lands.

## 7. What this program explicitly does NOT touch

- **Emit gaps** (`match` on user-defined sums). Separate blocker bundle for `idempotency.dag` / `parallelism.dag`. Tracked in ROADMAP's 2026-04-21 receipt-closure wave.
- **File-preference rank routing** (`src/v3/std/*` vs `dsl/std/*`). The lanes inherit whatever the rank-scaffold decision is at the time they run. They do not re-open that call.
- **Lattice-consolidation** (ROADMAP P2 "four hand-rolled `BoundedLattice<T>`"). `DescentEvidence` and `SubValueRelation` are two of the four. This program ports them as-is; the algebra-declaration consolidation is a separate lane that can run before, during, or after this program without coupling.
- **SG-2c / parser cutover / any self-hosting-cycle work.** Orthogonal.

## 8. Related

- `docs/v3-lens-capability-register.md` — the receipt this program closes.
- `INVARIANTS.md` P1 Modeling Faithfulness — the invariant the PROXY/STUB markers flag as at-risk.
- `feedback_checkpoint_dissolution_default` — governs stop-signals in each lane (named principle, not a tree file).
- `feedback_substrate_principle_audit` — pre-port audit checklist each lane runs before adding/modifying a substrate field/variant (named principle, not a tree file).
- `ROADMAP.md` — new row in the "v3 lens capability honesty pass" vicinity pointing at this doc plus the four lane placeholders.
