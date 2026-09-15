# Demand-engine program: derived demand, lawful decomposition, realization and materialization

> **Status: DRAFT for operator decision and side-chat review (2026-09-15, session fierce-seal-607).** No implementation lands from this document. It is the single design authority for one program: running ordinary sequential `.dag` programs at the loosest lawful structure their facts allow -- derived demand, one demand engine, lawful decomposition, and realization/materialization chosen outside program semantics -- without any program authoring dependency rows, widths or a private scheduler. Every milestone is a whole replacement migration (DESIGN section 3 standing rule): its root is deleted in the change that lands its replacement, with a named production consumer.
> **Operator direction carried (2026-09-14/15):** concurrency comes through dependency structure, demand-driven like Bazel's execution phase; one shared ready queue drained by leased capacity -- no widths, waves, batches or shard plans; every existing width/concurrency mechanism is deleted, not frozen or kept behind a serial fallback; future programs inherit this without knowing DependencyView or DependencyRelation; realization is how things happen and materialization is how things are presented/stored; key definitions and hashing are a conformance concern; local processes are fabric supply bounded by an observed offer (in CI, the runner's caps); remote execution is a later strategy over the same graph and keys; no half migrations.
> **Third-party review folded in (side chat, 2026-09-15):** the universal unit is a semantic demand over a node and its context, not a raw node; demand identity and materialization identity are two keys; semantic result contracts are not derived from cost; algebraic laws must be established under an observation contract before they license decomposition; occurrence identity is genuine identity at occurrence grain; dynamic dependencies across effects need a chosen protocol; the order below names its first authority cut. Section 7 lists what that review left for the operator to decide.
> **Operator rulings folded in (2026-09-15, second round):** D1 is ruled -- eval is the semantic judgment. Demand derivation and demand minimization are deterministic authorities over the admitted program and its declarations: alternate implementations must derive the same semantic demand graph, and policy never chooses one. Materialization and realization are interfaces with multiple admissible handlers selected under policy, so trying another strategy is binding another handler, never a second eval; a realization may refine a judgment into internal work, regroup it under applicable law evidence and choose order, placement and transport, but it cannot change the semantic judgment population. The first production realization this program selects is emitted Rust; once that route implements the complete judgment and refusal contract the interpreter becomes its offline differential oracle, and another handler may later replace it without changing eval. The demand engine sees graph execution only: it is opaque to effect payloads -- it never interprets a shell invocation, a file write or a request -- and consumes only generic prerequisite relations and generic share/admission verdicts that effect analysis projects from declarations one layer below it. The demand graph is planned at compile time as far as the language allows: every demand template, relation constructor, conditional alternative and typed membership slot is derived by the compiler; runtime never invents a dependency kind. `Branch`, `Match` and `Loop` are static demands with a runtime selector; a host call is a demand at its call site whose result is an ordinary value, opaque inside; the members of a collection that arrives from outside instantiate concrete demands only through their declared template. Laws are typed evidence consumed only by decomposition, established at an honestly reported rung and applied only within their carried bound.

## 1. Goal and displaced cost

**Goal.** An author writes ordinary, sequential-looking `.dag`. The compiler derives the semantic demands the program makes and their dependencies from program structure, effect contracts, external-input observations and established algebraic laws. A demand engine evaluates only what requested targets transitively need, releases dependents the moment a required result is available, computes each demand once, serves repeated demands from a store, and places heavy work on leased local processes (later, remote executors). None of this is authored per feature.

**Displaced cost.** The v2 native fold took about two hours. gunbc#11401 removed specific cost-shape defects -- a per-module rebuild of corpus-wide resolution state (R1, the one fix that is this program's class: a validated fact not carried, so every consumer recomputed it), an unread unary digest (C1), a whole-index provenance merge on memo hits (C2), per-token choice dispatch (C3) and per-position lexer rule scans (C4) -- byte-identical in driver rows at every step. What remains is structural: one sequential process over stage-wide folds, no reuse across runs, no lawful decomposition. The instruments that re-derive the numbers are the fold's `[native-cost-partition]` and `[native-prepare-split]` stderr lines and the bounded concurrent A/B harness recorded on #11401.

## 2. The model (terminal architecture)

### 2.1 Semantics, demand, minimization, materialization, realization, policy

DESIGN section 3 keeps interface, realization and policy as three facts, and section 2 minimizes the demand graph before materializing answers. The run of a program is therefore composed, not identified with its execution strategy:

| Layer | Owns | Never owns |
| --- | --- | --- |
| Semantics (eval) | what judgment a program denotes, sequential as authored | order, concurrency, placement, storage |
| Demand derivation | which judgments requested targets require, statically and dynamically | how or where they are computed |
| Demand minimization | one production serving several demands at their least common ancestor; authored duplication is removed, not cached | cost-based choices |
| Materialization | for unavoidable results: recompute, share, memoize, encoding, retention, store | what outputs a judgment owes |
| Realization | order, concurrency, grouping, placement, transport, process or remote strategy | semantic identity |
| Policy | admissibility, trust, resource fit, retention, and the cost objective ranking admissible options | semantics or output obligations |

In the corpus's existing wording (`v2.std.materialize`): `run ≜ realize ∘ materialize ∘ dependency_view`, with minimization before materialize and eval supplying each demanded judgment's meaning. The operator's framing -- realization is how things happen, materialization is how things are presented and stored -- is exactly the two right-hand layers; eval names the semantic judgment (D1, ruled 2026-09-15).

### 2.2 The unit: a semantic demand, not a raw node

`Node` is the universal structural carrier; not every node is independently runnable, and one structural node answers differently under different environments, inferred facts, resolved bindings, interpretations, external observations and semantics versions. The unit of demand is a **judgment** over structure and context -- conceptually `Judgment<Kind, Subject, Inputs>` -- keyed through the existing node structure without minting a second graph. Semantic demand grain, realization dispatch grain and materialization storage grain are separate facts: thousands of tiny demands may run in one in-process continuation, be fused into one child-process request, or be persisted as one aggregate artifact.

### 2.3 Identity: five facts, each a named keying relation

Following `docs/plans/keying-relation-design.md` (a key is a named relation within a named scope):

| Identity | Denotes | Scope | Derived from | Existing authority |
| --- | --- | --- | --- | --- |
| structural node identity | what a node is | global | Merkle fold of kind, edge labels, child identities; occurrence excluded | `v2.std.node` `content_hash` |
| demand identity | the question asked | global | judgment kind + structural subject + explicit arguments + static semantic context | none general yet; `std.materialization_provider` `request_key` for today's families |
| dependency read set | the exact judgments and observations an answer consumed | one answer | sealed when the demand reaches its terminal value | none yet |
| materialization identity | this answer | global, location- and provider-free | demand identity + canonical read-set identities + external observation identities + evaluator/compiler semantics identity | `std.materialization_provider` `artifact_request_key` / manifests (partial) |
| occurrence identity | this authored occurrence | one allocator (one source graph) | the graph-scoped allocator; never content, span, spelling or path | `std.occurrence_identity` scope law; `ScopedOccurrenceRef` for cross-graph references |

**Keys are algorithmic.** Each is a total derivation into a canonical encoding injective over its domain, hashed by a family suited to its sharing scope: structural fingerprints (`Fnv1a64Structural`) within one run; collision-resistant digests (the SHA families of `std.content_hash`) across a run, host or store boundary. Hand-authored, positional or lossy keys are defects; the C1 token-stream digest, which mapped every value above 255 to one tag, is the recorded specimen. The `conformance-identity` domain asks these questions of every change (M0).

**Occurrence identity is not a computation key and is not demoted.** It distinguishes structurally equal occurrences (the two `x` in `x + x`) for binding, diagnostics and provenance. A result that depends only on occurrence-neutral structure is already reusable across graphs; a result containing or indexed by occurrence identities must declare one contract: semantic-only (persist occurrence-neutral values, rebind locally after load), scoped (persist `ScopedOccurrenceRef`s usable while their scope is addressable), or rebinding (persist structural anchors plus a proved mapping into the destination graph). A bare occurrence ordinal is never interpreted in another allocator.

### 2.4 Dependencies are derived; declarations of underivable facts stay

The engine's relation is `v2.std.dependency` `DependencyRelation` over demand identity; compiler `DependencyView`s project into it. Semantic dataflow and control flow come from the section 4 behaviors: a call demands callee and arguments; `Bind` orders later computation after its value; `Branch` and `Match` demand condition or scrutinee, then exactly one arm of a statically known set, chosen by that value -- a static demand with a runtime selector, never a dynamic edge; products demand components per construction; `Loop` demands its static body with a data-valued trip count under its descent evidence. A host call -- a shell invocation, a REST request -- is a demand at its call site and nothing more: its inputs are demanded, its result is an ordinary value, and the engine neither interprets nor expands what the call does inside. A list that arrives from such a call is consumed like any authored list: `map`, `fold` and per-item `Match` over it are static demands whose membership is data. The compiler therefore derives every demand template, relation constructor, conditional alternative and typed membership slot; runtime never invents a new dependency kind or an untyped relation. Selector values activate statically declared alternatives; runtime collection members and call arguments instantiate concrete demands and relations through their declared templates, and every instantiated subject is keyed, admitted, entered into the read set and scheduled by the one demand engine -- a realization handler that hid those child demands internally would be a second engine. Data edges carry operand bindings -- which producer output satisfies which consumer input. **Hand-authored rows that restate derivable dataflow or control flow are deleted** (today: `v2.program` `program_run_dependencies`). **Declarations of facts the language cannot derive stay**: effect declarations, host-call input/output contracts, idempotency and freshness contracts, external observation identities, explicit workflow barriers, resource and trust constraints, opaque implementation boundaries. The engine never interprets them. Effect and resource analysis projects their ordering consequences into the generic prerequisite relations `v2.std.dependency` already carries (`EffectDependsOn`, `ResourceDependsOn`, `BarrierBefore`, `PlacementDependsOn`), which the engine consumes exactly like data prerequisites -- so `write A; write B` with no data edge still orders B after A inside the one engine rather than in a private readiness graph in some handler -- and into generic share and materialization admissions (`std.materialization_ladder` already separates PureComputation, IdempotentEffect, WorldRead and FreshEffect; a fresh effect is neither attached in flight nor served from a prior result). Realization consumes the full effect and resource envelope to select placement and acquire grants; the host-effect boundary performs the operation (section 2.7). An undeclared or unresolved effect never reads as independence: an opaque host call without an established envelope couples conservatively, because not interpreting shell cannot mean treating shell as pure.

### 2.5 The demand engine

- **Demand closure.** Evaluation starts from requested targets and pulls transitive demands; undemanded judgments never enter lookup, readiness or realization.
- **Static demand schemas, dynamically admitted instances.** A demand's templates and relation constructors are planned before it runs; no dependency kind is discovered at runtime. What runs decides selector values (which arm, how many iterations) and the members of declared collections (which files the shell call listed), each instantiating a concrete demand through a slot the plan already carries with a typed subject shape; the engine keys it, attaches it to any in-flight same demand when the share admission allows, detects cycles, inserts its relations and records its read. A running demand whose selected arm or admitted member is not yet available suspends and resumes on availability; that is the whole dynamic protocol (D4). The exact read set -- which arms and members were actually consumed -- is sealed at the terminal value and only then yields the materialization identity.
- **Availability, not scheduling.** Per demand: waiting, ready, lease pending, running, suspended, commit pending, available, refused, blocked by a prerequisite. Only available -- the exact result contract admitted -- releases dependents. A verdict bundle full of red tests is available; a crash or a corrupt object is not.
- **One live ready queue.** A demand whose prerequisites are available enters the one queue immediately; any completion immediately re-evaluates its direct dependents. No layers, batches or waves. Discriminating case: with A1 -> B and an independent A2, B starts as soon as A1 completes while A2 still runs.
- **One dispatch policy.** When ready work exceeds capacity: stable order by demand identity, admit the next demand whose requirements obtain a lease, leave unadmittable demands ready. No feature supplies another policy.
- **Typed blocked states.** Cycle, missing edge, refused prerequisite, waiting for capacity, running, suspended, lookup pending, no demand and key conflict are distinct.

### 2.6 Lawful algebraic decomposition

A dependency relation alone leaves the commonest sequential shape -- a fold whose step reads the previous accumulator -- a chain. Established algebraic laws break chains: associativity with identity licenses regrouping; commutativity, order-free combination; idempotence, collapsing duplicates; a homomorphism, pushing a map through a split; independent product dimensions, concurrent components. Three conditions bound this:

- **A law is evidence, not a record shape.** Today `Monoid` and `CommutativeMonoid` in `std.algebra` are field-identical: inhabiting them establishes an operation and an identity value, not associativity. Decomposition needs named law evidence such as `AssociativeUnder<ObservationContract>`, `CommutativeUnder`, `IdempotentUnder` and `HomomorphismBetween`, established over the complete observable result -- value, diagnostic population and order, effect requests and order, provenance, refusal shape, termination. An unestablished law leaves the sequential dependence intact.
- **Associativity permits many trees; the canonical one is chosen.** Stable incremental reuse needs an append-stable decomposition (content-defined chunks under a Merkle hierarchy, or a persistent sequence with canonical branching) so that appending invalidates a bounded path by construction.
- **Granularity is priced by total cost**: semantic work plus dependency bookkeeping, queueing, encoding, lookup, lease acquisition, process startup, transfer, commit and reduction -- otherwise splitting wins on paper and loses in execution.

This consumes the reduce-spine and idle-lane direction of `docs/plans/machine-shape-orthogonal-scheduling.md` (computation graph x machine facts x algebraic evidence -> schedule) rather than restating it.

### 2.7 Realization: placement from fabric supply

- **Placement** gains a truthful local child-process arm; current `std.realization` `Placement` has none, and `LocalInProcess` is not a child process.
- **Supply** is `product.fabric.supply` `SupplierOffer` over a local-process executor: one offer per conserved pool, quantity derived from the observed execution scope (effective cgroup CPU, memory and PID limits and usage) against the measured work footprint and named policy. An unobservable axis refuses; a footprint that does not fit yields zero, never one. In CI the runner slot is the scope.
- **Selection** goes through `product.fabric.selection` `CandidateOffer` and `select_supply`, with capacity-class admission, even with one offer. Selection is not occupancy.
- **Occupancy is two-level**: a host-global grant reserves each realizing run's scope (in CI the runner slot already is that grant); an in-parent atomic ledger subdivides it per demand by resource vector. No committed lease, no spawn; release on every terminal arm, only after quiescence.
- **Nested schedulers** (cargo, make) take tokens from the demand's committed claim and never choose their own width.

### 2.8 Materialization: share, recompute, memoize, store

- **Share** within a run is demand minimization: one demand identity requested twice is computed once, and later requesters attach in flight.
- **Recompute** is an explicit decision below the materialization ladder's cost floor (`std.materialization_ladder`), not a default. Today the floor is an exempt roster; M4 replaces it with a measured serve-versus-recompute decision.
- **Memoize and store** under collision-resistant materialization identity (`extdeps.realization.materialization_store_local` today). Lookup precedes capacity: a verified hit takes no lease and releases dependents; an unavailable or corrupt store refuses and never reads as a miss.
- **Semantic result contracts are interface, not cost.** The `std.materialization_provider` `ArtifactRequest` families define judgment family, correctness-relevant inputs, required outputs and completeness. Their terminal form is a generic semantic query contract (judgment, input schema, output obligations, identity derivation) from which a materialization request is derived when persistence is selected -- contract first, then optional materialization, never cost first.
- **Encoding.** Results crossing a process or run boundary use a canonical, versioned encoding with decode refusal and identity readback; shared context is materialized results consumed by dependents, never an inherited process address space.

## 3. Relationship to existing authorities and plans

| Authority / plan | Relationship |
| --- | --- |
| `v2.std.materialize` (`run ≜ realize ∘ materialize ∘ dependency_view`, analysis-side) | **Consumed and made live.** Its structural identity grade and spine wording are this program's; M3 is its runtime. |
| `docs/plans/keying-relation-design.md` | **Consumed.** Its named-relation-in-scope model is section 2.3; M1 is its first large consumer. |
| `docs/plans/realization-measurement-loop.md` | **Consumed.** Its gate (realize(T) content-addressed at minimal placement; uncached non-redundant work is an error) is M6's acceptance. |
| `docs/plans/machine-shape-orthogonal-scheduling.md` | **Consumed.** Its reduce-spine, idle-lane and algebraic-evidence direction is section 2.6. |
| `docs/plans/resource-aware-scheduler.md`, `bounded-input-cost-envelope-scheduling` (authority-only) | **Superseded on width.** `spawn_width` formulas are replaced by lease admission against an observed offer. Bounded-input admission and predicted-versus-measured cost roles are retained. |
| `docs/plans/serving-capacity-home-ruling.md` | **Consistent.** Shared selection, subject-specific occupancy; local processes get their own producer and linearization. |
| `docs/plans/content-hash-family-grounding.md` | **Consumed.** This program adds the family-by-sharing-scope rule. |
| `v2.lens.cost`, `std.realization` `CostAccount` | **Consumed.** Symbolic cost and measured/predicted axes exist; M4 joins them to demand and realization identities. |
| `v2.workflow.scheduler` / `executor` / `realization_runner` / `runtime_run`, `module_resolution_plan` | **Replaced (M3).** The readiness predicate survives; the layer planner, batches and sequential runners are deleted; `module_resolution_plan` and its mirrored host realization move onto the engine. |
| `std.materialization_provider` `ArtifactRequest` | **Retained as semantic contracts; generalized in M2.** |

## 4. Current census: what each root's deletion breaks

Static dependents by module (production / test), counted from file mentions; the loud census is the fail-closed floor on a delete-first branch, run per milestone before its detailed plan.

| Root | Production | Tests |
| --- | --- | --- |
| scheduler layers, executor, realization_runner, runtime_run | 6 (v2.program, module_resolution_plan, operand_flow, a plan row, doc_graph_roots, resolved_graph_cache.rs) | 6 |
| hand-authored DependencyView rows | 5 | 21 |
| v2.lens.parallelism (no independent arm; coupling evidence always unresolved) | 1 | 2 |
| eval as a recursive child fold | 3 (00_compile native driver, v2.program, runtime_run) | 5 |
| floor discovery width (DiscoveryWidthPolicy, CONTROLLED_WIDTH) | 3 Rust | 0 |
| derived_realization_schedule.rs | 12 | 0 |
| std.realize_pack advisory width | 10 | 2 |
| std.realization_width fallback / minimum-one | 15 | 4 |
| parse sweep unbounded threads | 16 (claim_executor, declaration_index, module_path_index, namespace_wave_admission) | 4 |
| build parallelism (ci_compile_jobs, jobserver, CARGO_BUILD_JOBS) | 23 | 12 |
| results indexed by occurrence identity crossing graphs | 48 files touch the allocator, minted ids or span index | 22 |

## 5. Milestones

Order: M0 -> M1 -> M2 -> M3 -> M4 -> M5 -> M6 -> M7. Each lists what it makes true, what it deletes in the same change, its production consumer, its qualification and its exit.

### M0 — Identity is a conformance domain; this program is the authority

**Makes true.** `gunbc.design_argument` `conformance_domains` gains `conformance-identity` (keys and hashing); `gunbc.roadmap.roadmap_review_criteria` derives its reviewer; this plan is registered and projected.

**Deletes in the same change.** Nothing (additive authority).

**Production consumer.** The derived review criterion, read on every change.

**Qualification.** Every home resolves; the reviewer roster and the generated DESIGN.md include the domain (`roadmap_review_role_witness_test` `production_conformance_domains_appear_in_generated_design`, `roadmap_review_function_witness_test` reviewer list).

**Exit.** Operator decisions D1-D8 recorded in this document, and side-chat sign-off.

### M1 — Semantic demand and identity contract

**Makes true.** The judgment vocabulary; demand identity, dependency read set and materialization identity as named keying relations with injective canonical encodings and family-by-scope hashing; per-judgment result contracts classifying semantic value, diagnostics, provenance, effects and refusal, with each occurrence-bearing output's contract (semantic-only, scoped or rebinding); evaluator and compiler semantics identity; external observation identity. Existing keys (`content_hash`, `request_key`, `typed_module_key`, the parse-table subject, store keys) inhabit the relations.

**Deletes in the same change.** Every second key derivation for a subject that already has one; lossy digest arms; any production key read across a scope its relation does not declare.

**Production consumer.** The native fold's existing `ArtifactRequest` families re-expressed as judgments with result contracts, keyed through the relations (no scheduling yet).

**Qualification.** Per relation: an injectivity witness over boundary cases with a collapsing mutation that reds; a family-by-scope refusal of a structural fingerprint as a cross-run key; a witness that two parses of one program with different occurrence ids yield equal demand and materialization identities while their occurrence-scoped outputs refuse cross-scope interpretation.

**Exit.** No production key outside a named relation.

### M2 — Derived dependencies and the dynamic-demand protocol

**Makes true.** Static dataflow and control-flow dependencies derived from section 4 behaviors, with operand bindings; Branch/Match/Loop as static demands with runtime selectors; host-call sites as opaque demands whose collection results fill declared membership slots; suspension on an unavailable selector or member, member admission and read-set sealing (D4); `v2.lens.parallelism` answers independence from admitted relations. `ArtifactRequest` generalizes to semantic query contracts.

**Deletes in the same change.** `v2.program` `program_run_dependencies` and every fixture row that restates derivable dataflow.

**Production consumer.** `v2.program`'s run and `module_resolution_plan`'s module dependencies, derived rather than written.

**Qualification.** Derived relations equal the hand rows on every existing specimen before the rows are deleted; a missing-edge mutation makes admission refuse instead of reading as independence; an undeclared effect refuses concurrent realization at the realization layer, not in the engine; a selected arm's read set differs by branch and yields different materialization identities.

**Exit.** No production dependency relation restates derivable program flow.

### M3 — One live demand engine (first authority cut)

**Makes true.** Section 2.5 with Share and explicit Recompute: demand closure, suspension, availability states, one ready queue, one dispatch policy, typed blocked states, in-flight attachment. The native fold's driver becomes a `.dag` demand over its universe; the emitted main reduces to the host-effect boundary.

**Deletes in the same change.** `v2.workflow.scheduler` layer planning, `v2.workflow.executor` batches, `v2.workflow.realization_runner`, `v2.workflow.runtime_run`'s sequential fold, `module_resolution_plan`'s antichain-batch walk and its mirrored host realization, and the Rust module loop in `v1.emit_rust`'s SourceRootEvalDriver main. The current evaluator survives only as an offline differential oracle, never a fallback.

**Production consumer.** The v2 native fold (landing receipts) -- the first production root that switches to the engine and loses its manual scheduling authority.

**Qualification.** Byte-identical driver rows against the pre-cutover fold on the bounded concurrent A/B harness and one capped exact-head full fold; undemanded judgments never run; the A1/A2/B immediate-readiness discriminator with a mutation deleting the completion unlock; two requests of one demand compute once (mutation computes twice); a refused prerequisite never unblocks; each typed blocked state reachable.

**Exit.** No production program run outside the engine for the native fold; no layer or batch executor remains.

### M4 — Measured demand economics

**Makes true.** Receipts join to demand and realization identities: semantic work, realization overhead, encoding and lookup, expected reuse, retention and resource footprint, as `CostAccount` axes per demand class; the ladder's exempt roster is replaced by a measured serve-versus-recompute decision.

**Deletes in the same change.** `cost_floor_exempt` roster entries replaced by measured decisions.

**Production consumer.** The M3 engine's materialization choice for the native fold.

**Qualification.** A planted demand whose dispatch overhead exceeds its work is not split; a planted recurrence above the floor is memoized; a cost receipt missing its realization identity refuses admission to the decision.

**Exit.** Materialization and grouping choices on the native fold are measurement-derived.

### M5 — Law-bearing algebraic decomposition

**Makes true.** Law evidence carriers under observation contracts, their establishment by inhabitance and witnesses, the canonical append-stable decomposition, and realization applying decomposition only where laws are established and economics favor it.

**Deletes in the same change.** Any convention-level parallelism assumption about folds (none may be introduced before this milestone).

**Production consumer.** Stage-wide folds in the native fold's path whose combine carries established evidence.

**Qualification.** Decomposed and sequential results identical over the full observation contract, including diagnostic order; an unestablished-law mutation stays sequential; an observation-changing combine (diagnostic order) is refused decomposition; appending one element recomputes a bounded path (counted).

**Exit.** Every production decomposition cites its law evidence.

### M6 — Realization and materialization selection; widths deleted

**Makes true.** Section 2.7 and 2.8 in full: local child-process placement through the fabric with two-level occupancy and resource vectors; memoize and store under collision-resistant materialization identity; canonical encodings; the realization-measurement-loop gate (uncached non-redundant work is a typed error). The floor's parse sweep and discovery run as engine demands; nested schedulers take tokens from claims.

**Deletes in the same change.** `DiscoveryWidthPolicy` and `CONTROLLED_WIDTH`, `derived_realization_schedule.rs`, `std.realize_pack` advisory width, `std.realization_width` fallback and minimum-one helpers, the unbounded parse-sweep thread scope, `ci_compile_jobs` refusal-to-serial and independent cargo or jobserver width, dead memory-governor constants.

**Production consumer.** The native fold and the required floor.

**Qualification.** Observation refusal (missing CPU, memory or PID evidence yields no offer); subunit refusal; conservation (running claims never exceed the grant); two-run conservation on one host; release on spawn failure, refusal, timeout and signal; canonical output under shuffled completion; a verified hit takes no lease; store unavailable refuses; same-key concurrent commit converges by create-new and readback; an M-versus-T receipt recomputes only changed materialization identities (counted); a repository census finds no production consumer of a retired width mechanism; byte-identical rows at every width the offer yields.

**Exit.** No width integer in production; every concurrent realization holds a committed lease; cross-run reuse on the landing-receipt path.

### M7 — Compiler stages as vertical judgment cuts

**Makes true.** One complete path at a time -- source demand, parse result, resolution result, inference and eval result, native verdict -- moved onto judgment-shaped demands, its stage-wide fold root deleted, then widened to the next path.

**Deletes in the same change.** Each stage-wide fold as its judgment form lands.

**Production consumer.** The native fold, path by path.

**Qualification.** Byte-identical results per cut; a one-file change recomputes only its dependents (counted); each cut's deleted root has no remaining production consumer.

**Exit.** The fold's cost is proportional to what changed.

## 6. Program-wide qualification

- **Identity of results.** Every engine milestone is byte-identical in driver rows against its predecessor on the bounded concurrent A/B harness, then on one capped exact-head full fold; a difference stops the line.
- **Discriminating controls.** Every new witness names its mutation and why it reds, and fits the floor's per-identity step budget (hand specimens, not the corpus).
- **Deletion is the census.** Each milestone runs a delete-first branch through the fail-closed floor before its detailed plan, and lands with a census showing no production consumer of what it retired.
- **Conformance.** Every change is read by `conformance-identity`, `conformance-realization`, `conformance-compute` and `conformance-decision`; departures state reasons.
- **Cost is reported, not asserted.** Receipts name the instrument that re-derives them, never transcribed numbers.

## 7. Operator decisions required before M1

1. **D1 — What does eval name?** Option A: eval is the semantic judgment and the run engine composes demand, minimization, materialization and realization (DESIGN-consistent as written; the side chat's recommendation). Option B: eval names the whole run (operator's 2026-09-15 framing), which requires restating DESIGN section 3's three-facts wording so semantics remains a separate authority. Recommendation: A, keeping the operator's realization/materialization split verbatim as the two outer layers. **Ruled 2026-09-15: A.** Eval is not pluggable; demand derivation is a deterministic function of the graph; minimization, materialization and realization are each an interface with N bound handlers selected by policy, and a strategy that shapes work from graph facts (cost, law evidence, size) is a realization or materialization handler that may regroup, order, place and retain but never changes which judgments are demanded. The production realization is emitted Rust; the interpreter is the offline differential oracle and not a target.
2. **D2 — The universal demand shape.** `Judgment<Kind, Subject, Inputs>` keyed through existing node structure, or another equivalent that mints no second AST graph. Recommendation: Judgment, with Kind a closed vocabulary grown per milestone. **Ruled 2026-09-15:** semantic demand and `product.fabric.demand` `Demand<P>` stay distinct carriers with no common parent; a semantic demand placed out of process produces Work and a fabric demand through a named realization projection, not inheritance, and the grain is not one-to-one (in-process realization and materialization hits produce none; retries produce a new fabric attempt for one semantic demand).
3. **D3 — Key vocabulary.** Adopt the five identities of section 2.3 (structural node, demand, read set, materialization, occurrence scope) as named keying relations, with collision-resistant families for cross-run materialization identity. Recommendation: adopt. **Ruled 2026-09-15: adopted**, with the side-chat carry-forward that the materialization row must separate the input-derived evaluation key from a result-content identity before M1.
4. **D4 — Runtime facts in a planned graph.** Ruled 2026-09-15, refined by side-chat review: static demand schemas, dynamically admitted instances. The compiler derives every template, alternative and typed membership slot; runtime invents no dependency kind. `Branch`, `Match` and `Loop` are static demands with a runtime selector; a host call is a demand at its call site whose result is a value, opaque inside; members of a collection that arrives from outside instantiate concrete demands through their declared template and are admitted by the one engine. The engine is opaque to effect payloads and consumes only the generic prerequisite relations and share admissions projected from declarations. The only runtime protocol is suspension on an unavailable selector or member. What remains open is one layer down, at the host-effect boundary in realization: how a demand that performed an effect and then suspends on a member is resumed. Options: resume in place (a continuation in the emitted program), replay under a declared idempotency contract, or refuse the shape with a typed diagnostic. Recommendation: refuse the shape until resume-in-place exists in emitted Rust; replay only under a declared idempotency contract; restart across a performed effect is never admitted. Admission is a capability join -- the judgment's effect contract with the selected realization's continuation capability -- decided before the shape starts executing; the engine receives only the resulting verdict.
5. **D5 — Law authority.** A law is a typed evidence value (`AssociativeUnder<Obs>`, `CommutativeUnder<Obs>`, `IdempotentUnder<Obs>`, `HomomorphismBetween`) that only the decomposition rule consumes; semantics never depends on one, and an absent law leaves the chain sequential -- a speed loss, never a correctness loss. Each is established at an honestly reported rung (DESIGN section 4b): a witness over a discriminating corpus with a planted non-associative red establishes it on the tested inputs (rung 2, the route for user-declared operations); an exhaustive witness over a small finite carrier (rung 3); structural derivation from lawful parts -- product of monoids, pointwise lift, max and min over a total order, set union (rung 3-4, the route for std composites); or citation of the modeled framework for the standard structures (rung 3). A law-evidence value carries its operation, domain, observation contract, evidence rung and applicability bound: a sampled witness authorizes decomposition only within its carried bound (the tested corpus or a range it structurally contains); exhaustive finite evidence requires a total completeness authority and, for associativity, every triple; structural derivation consumes law evidence for every premise behind a construction wall, so the record is not publicly forgeable; a citation establishes the abstract law and needs a separate inhabitance route showing the live operator is that operation under this representation. When evidence is absent or does not cover the exact subject, execution stays sequential. The observation contract -- value only, or value plus diagnostic population and order, effect requests and order, provenance, refusal shape -- is owned by the judgment's result interface, never selected by a consumer: a consumer may demand a distinct value-only projection but cannot narrow the producer's contract by ignoring axes it still returns. Recommendation: land the evidence types and the refuse-without-evidence rule in `std.algebra` at M5, witnesses as the first producers, structural derivation for std composites.
6. **D6 — Canonical decomposition.** Content-defined chunks under a Merkle hierarchy, or a persistent sequence with canonical branching. Recommendation: decide in M5 with measurements; the requirement (bounded invalidation on append) is fixed now.
7. **D7 — Host-global grant home** on hosts running several realizing runs: reuse runner-slot allocation or fabric cell reservation, or model a host ledger. Recommendation: reuse, decided by M6's census.
8. **D8 — First authority cut.** The v2 native fold's driver (M3). Recommendation: confirm.

## Dissolution trigger (DESIGN §6)

Delete this document when M7 exit holds and every milestone's deletions are census-verified: program runs go through the demand engine, no production key lies outside a named keying relation, no production dependency relation restates derivable program flow, no width integer exists in production, the store serves cross-run repeats on the landing-receipt path, and the compiler's stages are keyed computations -- at which point the carriers state the design and this document is a second copy of it.
