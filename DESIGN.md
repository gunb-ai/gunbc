# gunbc — Design

`README.md` and `CLAUDE.md` symlink here; this is the single source of truth. It is a projection of `gunbc.design_document` and is never hand-edited. v1 is the `gunbc` CLI and v2's seed; v2 is active.

The document is reasoned **serially**: §1 fixes the axioms, and each later section is a consequence of the ones before it, or an independent peer, never a restatement. The principles apply to this document too.

It carries reasoning only, because it is loaded in full on every turn of every session while ledgers grow without bound. The ledgers live apart: recurring failure modes one class per file under `dag/gunbc/recurring_failure_mode/` (combined view: `tools.docs_projection_gate` `regen`); declared rung drops in [docs/design-rung-drops.md](docs/design-rung-drops.md) (`gunbc.rung_drop`); dated operator rulings in `gunbc.design_ruling`, cited below by symbol. Plans live under `docs/plans/`, linked from the section that governs them.

---

## 1. The objective (the axioms)

Three **axioms**, assumed rather than derived:

- **A1 — there is a goal.** To *solve a problem* presupposes an agent with a goal; with no goal there is nothing to optimize.
- **A2 — time is the value.** Every agent intrinsically values time: it is finite and is the substance of acting at all. **Time is life**, and it is the one value we may assume is *shared*.
- **A3 — agreement is temporal.** Intersubjective agreement is possible only *across* time, and only on what stays stable under it.

Three things follow, in order:

- **From A1 and A2 — the solution is minimal, safe, efficient.** A solution is good exactly insofar as it spends less of the only thing valued, and time is spent three ways: **cost** (time to run), **safety** (time to recover from a silent wrong answer, paid later at interest), **complexity** (time to change). These are A2 applied to A1, not preferences, and the project optimizes them jointly.
- **From A2 and A3 — grounding is intersubjective.** Because time is the only assumed-shared value, a fact is grounded only by pointing at a shared, time-stable framework (§4), never an internal taxonomy.
- **At the limit — reduce intersubjectivity to physics.** The deepest such framework is physics, so the aim is to replace convention with necessity until nothing arbitrary survives. This is why §3 models the universal frameworks and real upstream rather than re-coining them: a nickname is convention standing where physics was available.

Safety has its own measure, used throughout: **how far each error class has climbed from runtime harm toward structural impossibility** — the guarantee ladder of §4b.

## 2. Minimize redundancy (the master move for cost and complexity)

Redundant work — **duplicated, unnecessary, or irrelevant** — loses on all three of §1's quantities at once: it costs more to run, widens the surface where harm hides, and adds complexity to maintain. A perfectly DRY process is therefore the minimal one. Through §1's lens, redundant work **defers** cost onto a later fixer or into a process destined to be thrown out; DRY is the refusal to spend someone's future time to buy the author's present convenience.

Redundancy is removed along two directions of one move:

- **Horizontal — one concept, every scale and breadth.** Model a concept once and derive every use; at the right layer nanosecond memoization and broad infra deployment are the same concept. *e.g.* `dag/std/integer.dag`: `Int8`…`UInt128` are 10 `Compose<Int, MachineWidth<N>>` rows, one axis rather than 10 types.
- **Deep — every concept decomposed to grounded atoms.** Nothing is opaque that is not *genuinely* atomic. The move is `decompress → map → reduce`: reveal the structure the source names, map each part onto the concept that already exists (DFS the concept DAG first), reduce duplicates. A `String` leaf hiding named parts is anemic modeling. *e.g.* `"LGA4926"` → `CpuSocket { package: LandGridArray, contact_count: 4926 }`, the number a grounded `Int` rather than a fresh enum.

**Minimize the demand graph before materializing its answers.** A repeated computation is first evidence about why the same semantic fact is demanded again, not yet a cache obligation. The order of questions, whose structural authority is `std.materialization_ladder`:

1. **Is the repetition authored duplication?** When several demands have a shared-state least common ancestor, carry, rewire or share the first value. Caching a later request only makes redundant work cheap and suppresses the signal that would rank it for deletion.
2. **Is there a reuse obligation at all?** One undeclared pure demand may recompute. Isolated consumers, declared replay, or unbounded future siblings instead create one reuse obligation at their least common visible ancestor.
3. **Does a provider discharge it?** One computation identity joins those demands to a provider whose scope reaches that ancestor, whose coverage includes the identity, whose key represents every declared input the result depends on, and whose retention spans the obligated lifetime; otherwise refuse with a typed cause.
4. **Only then, the realization (§6).** Demand minimization decides whether another production may exist, and cache purity decides whether a materialized value denotes the same fact. After both hold, unavoidable recurrence may explicitly recompute below the cost floor, and a provider is an optimization only when measured total serving cost is below recomputation.

A cache may discharge unavoidable recurrence; it may never excuse authored duplication.

The test that an edit *reduced* redundancy rather than moving it: **net concepts must not grow by re-invention.** Decomposing a leaf by minting a fresh authority for a concept that already exists is a failed decomposition.

## 3. Single authority (what keeps §2 from being undone)

Minimization holds only if each fact lives in exactly one place. The recurring violation is **nicknaming — a second name for one concept** — which duplicates work at the meaning layer and again in everything generated from it. A fork is always consolidated later, so it is a correctness concern, not a style one. Until enforceable this is diligence: model the accepted universal frameworks (classical logic, set theory, algebra) faithfully, and in `extdeps/` the real upstream spec — cite the source, keep its real names, declare its version, model what the API actually returns.

Single authority governs meaning, not only symbols. Nicknaming's dual is a **meaning fork**: one name, two materially different meanings. Within one naming surface and one declared version or epoch, holding a name constant may not silently change obligations, quality floor, refusal behavior, billing consequence or remedy; the same spelling in distinct scopes, or across a declared version transition, is legitimate. A materially different contract needs a materially different name.

Two corollaries. The import graph's only structural law is **acyclicity**: the `std`/`extdeps`/`compiler`/`workflow` folders are browsing conventions, not a direction rule. And a fact's home is its *layer*, not its file.

The load-bearing consequence: **interface, realization and policy are three facts.** `extdeps/` owns a dependency's *interface shape*; *transport* (shell, REST, SDK) is one of N handlers bound to it; *business policy* (which base ref, which flag) is a workflow fact. Policy has leaked when an argv carries a literal it should receive as a parameter; transport has fused when an operation is forked per transport. **The dispatch that selects a realization is itself realization**, so it sits peripheral.

*e.g.* one vendor concept once forked three ways by rigor — `CpuVendor`/`DramManufacturer` closed enums beside a stringly `GpuFacts.vendor` — dissolved into one generic `Vendor<Domain>` whose cited company entities live one file each in `extdeps/vendor/`. Likewise `dag/std/os.dag` projects per-vendor rows with no `match`, while the rows and the product-to-vendor dispatch live in `extdeps/os/`.

**Cite the symbol, not the position.** A `file:line` pointer is a second, positional naming scheme that any edit above the line silently invalidates. Name the module and symbol; a position is carried only where no symbol exists (a generated artifact, a data row, a line inside a blob), and then beside the symbol, never as the citation. A stale symbol is decidable from the `Node` tree; a line is not reachable from it at all.

### A witness discriminates at one interface

A claim's subject has a layer too. A witness establishes what ONE interface returns for inputs of a given shape, so its inputs are SUPPLIED VALUES at that boundary; deriving them re-executes production the claim is not about, once per claim — §2's duplicated work in a test. The tell: the claim's cost does not move when its assertions change. Reach is usually transitive (a fold over fifteen producers gives its claims their union), and its sharpest form is a production module importing `test.claim.*` (receipt gunbc#11457: 1,212,647 eval steps, approved twice). Supply the values; never buy the reach with a debt row.

**The pairing obligation:** every supplied boundary owes ONE INHABITANCE CLAIM that the real producer emits that shape, asserting the ROUTE and not only the answer, and SUPPLYING INPUTS MAY NEVER REMOVE THE LAST EXECUTION OF THE REAL PATH — deleting the integration must make a control fail. A designed fixture can stay green after its world moves by reaching the same verdict another way; the inhabitance claim is what turns a supplied hypothesis into a reading.

### Replacement migrations cut over at the root

When two structures answer one semantic question and one is to disappear, the change is a **replacement migration**: editing X leaf-upward keeps its root alive while every intermediate state becomes load-bearing, and a surviving X is an **attractor** whose vocabulary keeps shaping decisions premised on an assumption scheduled to die.

So the default is **delete-first**: uproot the root early, then fix forward minimally, solving each surfaced problem from first principles rather than restoring its old binding. In a fail-closed substrate **the deletion is the census — but only over the population a run compiles**. Outside the required gate (`gunbc.emitted_subject_build_gate emitted_subject_build_rows`, the closures of the entries those rows emit, so nearly every root is outside it) a dependent can stop resolving while the required job reports SUCCESS, so **for a root outside the gate, enumerate the consumers by name before you delete**. What cannot break loudly is covered by a declared, bounded §4b rung drop, never by silence.

Two carve-outs:

- **A gap-intolerant boundary keeps the staged form:** Y in shadow, then one transition that switches the root and deletes X.
- **Where no Y can hold the boundary, X stays frozen:** no new investment, no new rows on its growth surfaces.

**Atomic describes the authority transition, not the effort:** X's authority ends in one motion, Y never falls back to, resolves through or derives from X (X may serve as an offline differential oracle), and the minimum Y preserves every required refusal.

Frozen is the wrong state for a production-critical X whose replacement paused indefinitely, which is the v1 seed's situation; v1 is semantics-frozen, maintenance-active, under a PURPOSE test: a change is admitted when it serves the v2 self-host program. The authority is `gunbc.v1_maintenance_standing v1_seed_standing`, which also records that an active-maintenance arm becomes absorbing if every proposal classifies as admissible, and that its vocabulary is enforced by review, not by a gate.

The full doctrine (operative loop, consumer-relative root, disposition census, root-first cut search, terminal receipt): → [replacement migration doctrine](docs/plans/replacement-migration-doctrine.md); cut programs → [floor-cut](docs/plans/floor-cut-replacement-plan.md) · [namespace-cut](docs/plans/namespace-cut-replacement-plan.md).

### External upstream decomposition

One concept, one authority unifies a shared interface or formal shape; it does not merge independently governed entities that inhabit it. Each independently versioned upstream product, implementation, engine, vendor or specification has its own extdeps module. A generic hub may define agnostic shapes but may not enumerate concrete products, dispatch among them, carry product version rows, or store consumer coverage state. Observations this repository makes are receipts in the observing layer, not facts of the observed upstream; a missing observation is a downstream coverage obligation, never an `Unobserved` property inside the upstream module.

```
extdeps.whatwg.html_navigation        shared standard
extdeps.browser.chromium              Chromium implementation
extdeps.browser.google_chrome         Chrome distribution
extdeps.automation.playwright         automation dependency
gunbc.served_surface_browser_support  support policy and evidence join
```

A Chrome change must not require editing Safari’s module. Adding Opera must not widen a generic browser-product enum.

## 3b. Conformance (§2–§3 applied to a change)

**Conformance is §2–§3 applied to a change in motion** (ruling `gunbc.design_ruling conformance_needs_attention`). A change touches concept domains — it caches, holds something exclusively, places work, restates an upstream fact — and for each the corpus already has a home module. The question is: **does this change inhabit the existing model of each domain it touches, and where it does not, does it say why?** The answer is three-valued:

- **Conforms.**
- **Diverges with a stated reason** — admitted, and often correct.
- **Diverges with no reason** — the only red, because an unstated divergence forks the authority (§3) and its author was the one person who could cheaply say whether it was intended.

A two-valued reading loses the middle and either blocks every deliberate departure or admits every accidental one.

A domain is added by adding a row with its home; the leasing row's authorization-pattern clause was added that way (ruling `gunbc.design_ruling authorization_pattern_is_a_selection`). A domain whose home does not yet exist is a modeling obligation (§6), not a row; a home that does not resolve is a §3 citation defect; a home that owns only part of the row's claimed scope is a scope mismatch on the row. Single authority (§3) keeps naming and citation, minimize-redundancy (§2) keeps copied work, and conformance owns *inhabitance*. Criterion keys are distinct (`gunbc.roadmap.roadmap_review_criteria` `review_criterion_identities_are_unique`) but not semantically exclusive, so two criteria may report one defect from two perspectives.

**On the ladder (§4b)** a reviewer over a finished diff is *mitigation*. The climb: which domains a change touches is derivable before any worker runs — from the modules its plan imports and the vocabulary it introduces that a home already owns — so plan admission can name the homes in the brief and spawn only the reviewers the change reaches, leaving the roster as the safety net.

The roster is modeled in `gunbc.design_argument` `conformance_domains`, and `gunbc.roadmap.roadmap_review_criteria` derives one reviewer from each row; the reviewer's question and tells live in its brief. Each domain below states:

- **Owns** — the part of a change reviewed under this domain.
- **Boundary** — a nearby responsibility owned elsewhere.
- **Existing authorities** — declarations that already carry the model. A conforming change reuses or extends them rather than introducing a parallel representation.

### External facts

**Owns.** Facts governed by an independently versioned upstream: a specification, product, implementation, vendor, protocol, unit or tool. Each such upstream has its own extdeps module (section 3, external upstream decomposition).

**Boundary.** Product policy, observations this repository makes, dispatch among upstreams and coverage state belong to the consuming layer, never to extdeps.

**Existing authorities**

- **Upstream ownership** — what an external model may own
  - `extdeps.external_authority::ExternalModelScope`

### Materialization / realization

**Owns.** Faithfully storing and executing an admitted identity and result contract: completeness, readback, retention, transport, refusal and binding, with no substituting, truncating or bypassing the admitted key. Cache purity is one discriminator, not the whole domain.

**Boundary.** The key relation itself belongs to keys / hashing; this domain begins once the identity and contract are admitted.

**Existing authorities**

- **Materialization contract** — what is requested and what result is promised
  - `std.realization::Materialization`
  - `std.materialization_provider::ArtifactRequest`
- **Storage contract** — how admitted results are stored and read back
  - `std.artifact_store::ArtifactStore`
  - `std.cache_interface::StorageSurface`

### Leasing, locking, grants and privileged access

**Owns.** Exclusive or bounded use of a resource, and the authorization a privileged effect requires.

**Boundaries.**

- A slot whose generation line is an audit trail keeps it whole; a slot whose only fact is its head uses the distinct windowed operations, a separate contract rather than a mode.
- Which authorization pattern performs a privileged effect is a section 3d selection, not a call-site choice; its census classifies every current site by executing the selection.

**Existing authorities**

- **Lease and exclusion** — time-bounded possession and atomic updates of shared state
  - `std.temporal_effect::HeldLease`
  - `std.durable_compare_and_set::CasExpectation`
  - `gunbc.durable_cas_file_store::file_compare_and_set_windowed`
  - `std.durable_exclusive_hold::DurableHoldState`
- **Grant and resource scope** — what may be used, by whom, under what authority
  - `std.scoped_authorization::ScopedAuthorization`
  - `std.effect_grant::Grant`
  - `std.resources::ResourceHandle`
- **Authorization-pattern selection** — chooses the admissible mechanism for an effect
  - `gunbc.auth.authorization_pattern_selection::select_authorization_pattern`
  - `gunbc.auth.access_token_source::select_access_token_source`
- **Recurring automated access** — workload identity and scoped secret access
  - `gunbc.auth.github_gcp_federation::github_wif_provider`
  - `gunbc.auth.gcp_secret_access::SecretAccessGrant`
- **One-off approved access** — requests, capabilities and redemption
  - `gunbc.auth.access_request::AccessRequest`
  - `gunbc.auth.approval_capability::ApprovalCapability`
  - `gunbc.auth.approval_broker::redeem_capability`
- **Coverage evidence** — classifies the privileged-effect sites that exist today
  - `gunbc.auth.privileged_effect_census::privileged_effect_census`

### Fabric / compute

**Owns.** Selection, allocation, placement and admission of compute resources. product.capacity and product.fabric are the shared authority, consumed by compute cells and by inference serving.

**Boundary.** Compute cells and serving share the priced selection fold but not one occupancy model: a compute cell is stateless between grants and settles money, while a serving seat holds a loaded, quality-conditioned replica ([serving capacity home ruling](docs/plans/serving-capacity-home-ruling.md)).

**Existing authorities**

- **Shared demand and supply selection** — what work needs and which supply serves it
  - `product.fabric.selection::select_supply`
  - `product.fabric.work::ExecutionRequirements`
  - `gunbc.compute.work_request::WorkOperation`
- **Compute cells** — layout, reservation and admission
  - `gunbc.compute.work_provider_local::ComputeLayout`
  - `gunbc.fabric_control_plane::CellReservation`
  - `gunbc.fabric_executor_class::CapacityClassAdmission`
- **Seats and serving** — seat demand, grants and occupancy
  - `product.capacity.lease::LeaseGrant`
  - `product.capacity.pool_events::SeatRequest`
  - `gunbc.fabric_event_log::fabric_seat_acquire`
  - `gunbc.harness.harness_seat::harness_bind_seat`
  - `gunbc.serving.turn_admission::serving_group_admission`

### Decision / selection

**Owns.** A choice that stays attributable to its subject, candidate field, constraints, evidence and policy; this domain exposes accidental forks of that authority.

**Boundaries.**

- The hard selection laws stay with the section 3d reviewer.
- Goal assessment (std.goal_assessment) checks an already chosen state; it is a boundary of this domain, not a member.

**Existing authorities**

- **Decision subject, result and receipt** — what is decided and the record of the choice
  - `std.decision::DecisionSubject`
  - `std.decision::RealizationSelectionResult`
  - `std.decision::SelectionReceipt`
  - `std.decision::select_realization`
- **Pareto evidence** — axes, readings and dominance; a frontier, never a winner
  - `std.pareto::SelectionAxis`
  - `std.pareto::ParetoEntry`
  - `std.pareto::DominanceVerdict`

### Keys / hashing

**Owns.** The key relation: what is identified, in what scope, from what complete canonical preimage, under what equality and collision rules, and whether its declared inputs are complete.

**Boundaries.**

- Faithfully realizing an admitted identity belongs to materialization / realization.
- Demand identity and the dependency read set are a declared frontier, entering when M1 of [demand-engine-program.md](docs/plans/demand-engine-program.md) lands their carriers. The relation model is [keying-relation-design.md](docs/plans/keying-relation-design.md).

**Existing authorities**

- **Hash family and structural identity** — how a node's identity is computed
  - `std.content_hash::HashFamily`
  - `v2.std.node::content_hash`
- **Request, computation and occurrence identity** — the keys materialization and source graphs use
  - `std.materialization_provider::request_key`
  - `std.computation_identity::ComputationIdentity`
  - `std.occurrence_identity::OccurrenceIdAllocator`
- **Equality and cache admissibility** — whether a key type's equality may be relied on (instantiation-blind at this rung)
  - `std.algebra::algebra_profile_equality_extensional`
  - `extdeps.realization.cache_purity::CachePurityVerdict`

### Process observability / reporting

**Owns.** Process reports traceable to the facts and occurrences they describe: the owning domain establishes the fact, the observation model represents its occurrence, a renderer encodes it, and a bound effect delivers it.

**Boundaries.**

- Producing bytes is not delivery.
- A repository population measured at a revision stays with gunbc.repository_census_observation.

**Existing authorities**

- **Observation and recording** — the occurrence and its record
  - `std.observation::ObservationEvent`
  - `std.observation::RecordedObservation`
- **Presentation** — how an observation is shown
  - `std.observation::ObservationPresentation`
- **CI encoding** — the rendered CI line
  - `gunbc.observation_ci_render::ci_event_line`
  - `gunbc.observation_ci_render::ci_render_line`

## 3c. Consumption (who consumes a change, and how)

Conformance asks whether a change inhabits existing models; **consumption asks the converse of every model it adds: who consumes it, and by what executing route** (ruling `gunbc.design_ruling consumption_interrogated_separately`). §5 already rules that a typecheck and a grep are not consumers, but folded into other questions it let a diff add a well-shaped type, a row and a witness that the row exists while nothing read the row. A declaration nothing consumes is §2's redundant work in its purest form.

For each added declaration, **name the consumer** (fold, entry point, realization, emitted artifact) **and the executing route**, or classify it: consumed in this change; consumed by a named later change (a declared frontier, with its trigger); or dangling — the only red, however well modeled. Tells: a `pub` declaration with no call site in the closure, a row no fold reads, a module nothing imports, a witness asserting existence rather than exercising behavior, consumption claimed in the PR body but absent from the diff.

**On the ladder (§4b)** the reviewer is *mitigation*. A declaration's consumers are already a fact of the namespace tree, so the unconsumed set is derivable at ingestion and plan admission can refuse a dangling set before any worker runs. Until then this is the one criterion that reads a change from its consumers inward.

## 3d. Selection precedes convergence

**Selection precedes convergence.** A convergence primitive (`std.goal_assessment`, `ensure`) neither enumerates subjects nor chooses a realization: it assesses a caller-supplied goal against an independent observation, or decides Noop/Apply/Refuse over a caller-supplied requirement with independent readback. Choosing inside convergence is the conflation forbidden here; the selection authority sits before those folds, in `std.decision`.

**Pareto assessment is not realization selection.** `std.pareto` computes dominance and the frontier: axis identities rather than positions, per-axis orientation, interval readings, `Incomparable` distinct from `DominanceUndecided`, and refusal on missing or duplicated funded-axis readings. A front of two is not a winner. Realization selection is the policy-bound operation that produces one desired realization; it **consumes** `std.pareto` and must not redeclare its vocabulary — a second decision algebra is the §3 fork.

The result is scoped, never `Optimal`: `SelectedWithin` binds a `SelectionReceipt` (subject, candidate-field identity and standing, constraint authority, funded axes, evidence snapshot, Pareto survivors, policy authority, selected identity). A change to any of those is a **different decision**, so a stale choice is detectable. The other outcomes:

- Unresolved evidence is `SelectionNeedsEvidence`.
- Two survivors without a policy is `SelectionNeedsPolicy`.
- A candidate violating a hard constraint is excluded *before* Pareto, never ranked.
- An incomplete subject — required (objective-term × decision-variable) cells are an identity join, and terms drop only with order preservation — refuses: absence is not irrelevance, and `SelectedWithin` may not mint over an incomplete closure.

Ranking work on a funded multi-axis field **is** this law applied to authorship, so §6 does not restate it. `std.decision` is consumed (`gunbc.spark.serving_deployment_selection` `select_serving_deployment`, `product.fleet_operating_point` `fleet_operating_point_selection`), so §3b's decision row names it; that row checks inhabitance only, and the hard laws above stay with this section's reviewer. The effectful cycle after selection — admission, actuation with uncertain completion, independent observation — has no consumed home yet (first consumer: the fleet admission spine); a receipt cannot substitute for readback, and an actuation return cannot mint convergence.

## 4. The closed, grounded substrate (what makes §2–§3 decidable)

You can unify and decompose *mechanically* only in a closed, grounded system. A program is a dependency graph over two primitives (`Node` + `Edge`) and a closed vocabulary of 6 connectives and 6 behaviors — `Value`, `Transform`, `Branch`, `Loop`, `Bind`, `Match`. Surface syntax is sugar that adds no power. Execution is **bounded and forward** (cyclic relations via acyclic encodings, recursion as sugar over `Loop`), so decidability and termination fall out rather than being separately proved.

**Grounding is intersubjective**, and in a closed system **a heuristic is never necessary**: the richer source always exists or can be written. Because the substrate is closed, operations come from *inhabitance* (no per-type ops), and emission, ingestion and coercion are **one** total decision procedure run in different directions — N models, not N×M adapters, every refusal a located, typed mismatch.

- *e.g.* `dag/std/algebra.dag` derives `Int.add` from `Int` inhabiting a ring; termination is *checked, not discovered* — `DescentEvidence = Strict | NonIncreasing | DescentUnknown` inhabits a `BoundedLattice` with bottom = fail-closed.
- *e.g.* **one grammar, read in both directions.** Ingest selects a production forward to fold surface syntax into a core `Node`; emit selects from the *same* rows backward, so a new target language is rows in `extdeps/languages/`, never an edit to the fold. Coercion is the same move sideways: a homomorphism check.

**Coercion is the implicit-safe subset of declared conversions** (ruling `gunbc.design_ruling coercion_is_the_implicit_safe_subset`). An implicit route must be total, exact, semantics-preserving, unique, pure and consent-free; a phase that changes interpretation, needs proof, may lose information, needs a policy or has competing routes refuses implicit coercion, and the author names a conversion plan. Every phase stays represented in the typed program. Target realization may refuse to realize a plan but never decides whether a conversion is legal, so a target's cast table is not the source language's semantics. *e.g.* `Bool` does not implicitly inhabit `Int`; `true as Int` refuses unless it names a declared route such as `bool_indicator`.

**Text has two representations of one type, and every crossing goes through the declared unfold or refuses** (ruling `gunbc.design_ruling text_crossings_go_through_the_unfold`). `v2.std.text` `String` is `FreeMonoid<Char>`; the kernel `String` is host text; neither is a nickname for the other. The only route between them is the Unicode scalar unfold, `gunbc.structural_realization_bindings` `literal_homomorphism_rows` `UnicodeScalarSequenceUnfold`. Type compatibility keys on the exact declaration a position resolves to, never its leaf name; the unfold is not a property of literals, so a non-literal value at a crossing takes the same route or refuses; and the untranslated crossings inside the emitted compiler are measured and dispositioned under `gunbc.rung_drop.text_boundary_identity_wall`, whose standing follows the measurement.

## 4b. Safety: the guarantee ladder

Safety is the reduction of the state space in which a program can silently do the wrong thing — not the presence of diagnostics or the absence of crashes. Every discovered error class sits on one ordered ladder and is obligated to climb it:

1. **mitigatable** — the failure occurs; harm is contained by total operations, typed outcomes, bounds, rollback, isolation.
2. **mechanically preventable** — a generated test, lens or gate reliably exposes and blocks it, but the invalid state remains writable and safety depends on that mechanism executing and staying enrolled.
3. **structurally guaranteed** — the source can still describe the invalid state, but no `Accepted` program contains it: the compiler derives a proof or refusal from modeled structure.
4. **structurally impossible** — the invalid state has no constructor in the canonical model.

Higher subsumes lower: construction over proof, proof over validation, validation over mitigation. **Silent wrongness is not a rung**: it is below the ladder and forbidden outright (§5). Beside the ladder, deliberately not a fifth rung, is **outside the modeled guarantee** — external reality, undecidable properties, undeclared intent — observed, refused or mitigated at a declared boundary, never fabricated, so that not modeling something cannot masquerade as a weak implementation.

**Before writing a top-rung check, ask whether its RED is authorable.** A check whose forbidden state is expressible nowhere it runs is a decoration, green by construction and later cited as coverage. A state unrepresentable in the accepted corpus is often still writable as fixture source handed to the compiler, and then declining that red is specification-without-execution; where no harness can express it, the missing harness is the next-rung trigger.

Every class has an **attainable ceiling, derived rather than aspirational**: a decidable, fully modeled class may reach structural impossibility; a decidable class missing its authority is §5's wall after grounding; a class blocked on a language capability names it as its trigger; an undecidable property remains a ratchet or validator; an external fact remains a boundary obligation. **Below ceiling is a correctness gap, never optional elegance.**

Four meta-obligations make the ladder operational:

1. **Rung honesty, at a declared subject grain.** The reported rung equals the rung established by executed evidence — a discriminating RED refused on the real acceptance path plus an accepted positive control. Source→interpretation, source→each emission target and carrier→carrier are separate paths, and a class's rung is the **minimum across its in-scope paths**. A type name, diagnostic variant, inert lens or plan establishes nothing. **Rung inflation** is worse than sitting low, because an inflated class never ranks for climbing.
2. **No untracked stall.** A class below ceiling names its next-rung trigger, separating *cannot climb further* from *can climb after one grounding* from *can climb now but unbuilt*. Only the first is permanent.
3. **No silent regression.** A rung may be lowered only by declaring previous rung, temporary rung, reason, bounded population and restoration trigger. **The trigger names the CAPABILITY restored**, because a drop is retired by its trigger and nothing else; a trigger naming an artifact states what the artifact must be sufficient for. The review tell is a grain mismatch between loss and trigger: plural loss with singular trigger, corpus loss with per-module trigger.
4. **Dissolution on climb — production handling only, never the evidence.** A climb deletes the lower-rung production machinery it obsoletes, but the discriminating RED and positive control **stay enrolled**; an expecting-red probe that greens when its wall lands becomes a permanent regression control.

At a service boundary rung honesty is commercial: an opaque dimension needs a falsifiable quality floor with a named consequence (billing, remedy or refusal), or the claim is marketing. Below-floor delivery refuses or discharges the remedy; materially different delivery is its own named, priced product; losing the ability to verify a floor is a declared drop that may force refusal and never licenses silent below-floor delivery.

Every newly discovered error class — incident, review finding, runtime exception, falsifier divergence — files or updates one row under `dag/gunbc/recurring_failure_mode/` (`gunbc.recurring_failure_mode`): invalid state, harm, distinguishing facts, rung found at, ceiling with reason, next trigger. Declared drops are rostered in full in [docs/design-rung-drops.md](docs/design-rung-drops.md) (`gunbc.rung_drop`). The two are separate because they are appended separately, and this document names neither population so that an append never regenerates it.

**The floor first, the differentiator above it.** gunbc must hold the ordinary compiler floor — names resolve, applications bind in exact bijection, values inhabit declared types, fields exist, closed variants eliminate exhaustively — and no higher capability compensates a failure there. Above it, because the substrate carries causal, cardinality, algebraic, effect, ownership, cost and realization facts, the ladder reaches classes ordinary compilers leave to tests and postmortems: an empty collection into a nonempty consumer, recursion without descent proof, a non-idempotent effect under retry, a blown complexity bound, a realization that does not preserve behavior.

The promise is **not** that every property becomes impossible, but that every modeled class climbs to its highest honest rung with ceiling and residual risk explicit. The compiler does not invent intent, prove unmodeled reality or lift arbitrary predicates to proof, and a brand or `Validated<T>` is cosmetic until construction enforces it. Rung census and climb plan: → [compiler-guarantee recovery gap analysis](docs/plans/compiler-guarantee-recovery-gap-analysis.md).

## 4c. Source annotations (prose the substrate can see)

**Source annotations are captured authored-source data — neither semantic program data nor discarded trivia.** Annotation syntax goes through an annotation-specific lexical channel and may not produce a semantic token or binding; adding, deleting or moving an annotation cannot alter any occurrence identity, semantic graph, resolution, semantic hash or target bytes, and semantic passes see only the annotation-erased projection. The `.dag` realization admits only standalone leading `//` blocks on module-scope declarations; other forms refuse until modeled.

**Prose is not forbidden; unclassified prose is.** An invariant, receipt, event, ruling, citation, status, count or dissolution condition belongs in a typed carrier, and a `String` declaration whose only purpose is commentary is misplaced or dead data. This was measured: making `//` a parse error made comment syntax unwritable but not commentary, which moved into `data …: String` rows indistinguishable from program data. `//` is the explicit quarantine boundary. An annotation may record irreducible rationale for *why* a construction has its shape; it must not restate what the declaration says, and it is never evidence that a machine claim holds, since no `Accepted` program can read it.

## 4d. Epistemic humility (what the corpus may assert about the world)

A consequence of A2 and A3 applied to authored claims about the world, and a peer of §4b: §4b governs what the compiler can guarantee about a program; this section governs what the corpus may assert about the world.

The obligation is symmetric (ruling `gunbc.design_ruling epistemic_humility_is_symmetric`):

- **Do not assert as deduced what is only inferred.** An inference is a bet: typed as a bet, neither deleted nor promoted. A reader who consumes it as fact is the defect.
- **Do not over-generalize a prohibition.** Forbidding more than the evidence supports costs opportunities nobody sees, so it looks like rigor. This is not §5's never-trap (claiming construction for an undecidable class); it is forbidding more of the world than the cited variables support.
- **Do assert what you are confident in.** A fact declined is a wall that does not exist. Under-asserting is not humility.

The homes: `extdeps.external_authority` `CitedFigureStanding` = `CitedToAuthority { authority }` | `TranscribedUncited { read_obligation }` carries a figure's evidentiary standing in its type, an uncited figure naming what would ground it; and `strategy.node_rental` `SupplierHypothesis`, whose only arm answering true requires a cited discriminator. The cautionary specimen: `product.node_power_envelope` `altra_two_dpc_ceilings` were self-declared `TranscribedUncited`, and `effective_data_rate` consumed them as a wall — a typed bet consumed as deduced fact, whatever the sources later say.

## 5. Fail-closed (§1's safety axis)

Minimizing cost and complexity is worthless if a wrong thing passes silently. A wrong answer is a **loud error, never a warning**: every path succeeds fully or fails with a typed, located diagnostic, and no plausible output is fabricated. Relax toward application-layer leniency only under protest, and lean to infra so others can build on your work.

Stronger than catching a wrong state is making it **unwritable — correctness by construction, not validation.** A check restating a constraint the model already carries is a second representation of it (§2/§3). The tell that validation stood where construction was available: it can be satisfied by editing the *declaration* while the realization still lies.

Construction works only where membership is **decidable**, so every class is a *wall now* (decidable and grounded), a *wall after grounding* (decidable, awaiting its authority), or a *ratchet forever* (undecidable — optimality, by Rice). The word **never is the trap**: it lets a ratchet masquerade as a wall.

### Done means executed

The deepest trap is **specification-without-execution**: a typecheck and a `.contains()` grep are not consumers. Done means a real consumer **green by execution** plus a discriminating input that goes red when the behavior is wrong. Fluent, type-checking, grep-passing output is exactly what looks finished without running.

A test's oracle needs an independent referent. **A merge-blocking test may compare a live repository population to a numeric literal only when the literal is grounded in a controlled fixture, an external or versioned authority, an explicit policy budget, or a monotone debt contract over a closed, independently discovered universe checked at identity grain.** A count copied from the current tree is a change detector: if automating its update collapses the test to `measure() == measure()`, it checked nothing. Completeness is an identity join, not a count equality.

### Stop visibly; never conceal the deficit

**When the precise answer or the final construction is unavailable, stop visibly. Do not hide the deficit by widening, bypassing, or installing a second path.** The concealment erases the only signal that the precise mechanism has a deficit. It takes four forms:

- **Runtime uncertainty → typed refusal.** Substituting the superset (rerun everything, scan all keys) is the *absorbing fallback*: it conflates ⊤-as-answer with ⊤-as-ignorance and its cost scales with the corpus rather than the change. A failure arm refuses, never widens, and every degradation is typed, located and countable. Not this pattern: an over-approximation computed *as* the answer, or a loud, budget-bounded interim fallback landing with its dissolution trigger.
- **A refusal that fires → the line stops.** A toggle that proceeds as if a refusal had not fired is an *escape hatch*. The discipline is the **factory model**: a deficit stops the line and is analyzed before restart; the only second mode is an audit replay that ledgers deficits and reports rather than greens.
- **An authoring obstacle → root cause.** Routing around a parse error you do not understand, or a check that will not green, is the *workaround*. The concealed deficit is usually in the language layer (§6), so **noticing you are implementing a workaround IS the line-stop signal.**
- **A temporary deviation → operator approval outside the diff.** A dissolution condition describes how admitted debt ends; it does not authorize creating it. A scaffold's benefit is zero where the modeled path exists. → [scaffold admission doctrine](docs/plans/scaffold-admission-doctrine.md).

**A diff landing a silent widen, fabricated default, uncounted degradation or escape hatch is a hard reject.**

Distinct from these, because it crosses a principal boundary: moving deferred cost onto someone else — customers, employees, neighbors, future maintainers — is **externalization**. The honest arms are to absorb and reserve for the risk, or to expose the transfer as a separately named and priced contract; keeping the old name, price or contract while someone else bears the burden is **externalized degradation**.

## 6. How to work (given §1–§5 — these coexist)

- **Model:** DFS the concept DAG before inventing vocabulary; invent or reuse on proven coincidence, never bare-alias; a finished stage is one fold (any residue is a named irreducible kernel or un-migrated modeling); model just-in-time and let the mark on the carrier be the authority, not a parallel ledger.
- **Intellectual sustainability:** do not spend future author, reviewer, operator or maintainer time for present convenience. Work expected to be thrown away is *presumed redundant*. The reviewer's test, whatever the author labelled: **will this artifact survive the terminal architecture substantially unchanged, and be consumed by it?** If not, presume scaffold and stop the merge; a missing dissolution condition makes that more severe, and adding one after review only makes the proposal eligible for a decision.
- **Tells, each a presumption:** a hand-authored workflow, deployment or migration script (out-of-band actuation); a second path beside a modeled route (parallel authority); bridge, shim, for now, temporary, until, later (deferred refactor); a model to be deleted whole when the real one lands; a hand-authored projection the model should generate; raw shell implementing semantics expressible in `.dag`; a broad wrapper around a type or modeling deficit; a condition whose terminal is rewrite this properly; a new artifact with no final consumer.
- **Rank on a funded multi-axis field (§3d).** A 5ms step gets no pass for not being the 80s one; it might be a 5ns step. Hence **bare minimum cost**: a proven cost-shape defect — a copied accumulator, a quadratic fold — is *always fixed*, regardless of realized n, because n is small here is not time-stable.
- **Denominate the benefit:** the deliverable is a *displaced cost* someone pays to remove; the lens or substrate is the mechanism. Priced in elegance, the work is unbounded — the purity trap, economic twin of §5's never.
- **Enforce with lenses, not grep — but construction first.** A lens is validation: make the class unwritable where you can and keep lenses for the residue, where a pure reader over the same `Node` tree costs no substrate edits. A lens nothing gates on is itself a lie.
- **Root-cause to the language layer** and fix related systems together; a local subsystem patch is the forked-logic trap (§6b). *e.g.* one catamorphism `fold_node` serves every v2 stage, and #4699 dissolved `06_translate` from 4,912 to 3,973 lines by using it.
- **Name the instrument, never transcribe its output.** Cite a measurement by the producer that re-derives it — run, flag, entry point — for the reason §3 cites symbols: a transcribed number rots unreachably. A measurement not worth an entry point is a one-off, not an instrument.

## 6b. Chain re-derivation (how a defect is worked)

§2, §3 and §5 applied to debugging, and how §6's root-cause rule is discharged (ruling `gunbc.design_ruling chain_re_derivation`). A defect is observed at one link `c` of a dependency slice `a → b → c → d`. Patching `c` where it misbehaves, when the fault is upstream, encodes an undeclared fact about `a` or `b` — validation where construction was available (§5), a second authority (§3), redundant with the derivation it shadows (§2) — and the defect recurs at the next consumer of the same fact.

The method:

1. Identify the producer-to-consumer slice the defect sits in.
2. For each link, state what it means, what it may assume, and what its owning contract and every relevant consumer (`d` included) establish that it owes.
3. **Stop at the earliest boundary where the derivation cannot be justified** — a contract violated, missing, under-specified, duplicated, or insufficient for its consumer.
4. Repair there: declare the missing assumption on its owning carrier, consolidate duplicate answers, and delete work no admitted consumer demands. Work required by the interface, another consumer or a safety obligation is not redundant because one consumer ignores it.
5. Re-realize the downstream links rather than patching them; leave sound upstream links alone.
6. Use measurement to falsify or confirm the derivation, never to choose the repair.

If the earliest unjustified boundary is `c`, the local fix *is* the root-cause repair. *e.g.* resolve once rebuilt corpus-wide state per module, and the repair was upstream: one shared `ResolutionContext` per native ingest in `v2.compiler.compile`. By contrast, when the emitter's own tail-call lowering was the earliest unjustified boundary, the local repair was the right one.

**Measurement is the adversary, not the selector.** It locates the failing route, prioritizes the slice and confirms the old computation is gone; a sound reading states beforehand which multiplicity collapses, which route disappears and which outputs stay equal, and a contradiction from a reconciled instrument falsifies the claim (a failed reconciliation falsifies the instrument). But a number underdetermines the chain. Targeting one axis (make resolve 10% faster) yields a cache, batch, pool or knob at the measured link that moves cost rather than removing it; such a mechanism is right only when derived from the demand graph, a complete identity and an admitted provider (`std.materialization_ladder`).

- **Owned components get opened.** An external upstream stays a black box with an honestly modeled contract; an owned component whose contract is missing, violated, duplicated or insufficient must be opened.
- **No level is presumed correct**, including the structure the slice sits in (§4d turned inward). Touching code is taking responsibility for it.
- **There are no special layers.** The seed, the emitter or a load-bearing stage raises the evidence bar for landing, not an exemption from the read.

The tells: a symptom-link fix with no statement of why that link is the earliest unjustified boundary; a guard for a state earlier links could have made unwritable; a second fix to one link in a short time; a percentage on one axis as justification; knowing *that* `c` is wrong without being able to say what `a` and `b` established — the line-stop. The class is `gunbc.recurring_failure_mode` `symptom_link_patched_without_the_earliest_unjustified_boundary`.

## 7. Self-hosting (the principles applied to the compiler itself)

The compiler is a pure transform and an ordinary substrate fact, analyzable by its own lenses. The `.dag` graph is the truth and Rust is one realization — a seed that shrinks toward zero. v2 emits its own sources, proven **by execution**: the emitted module compiles and holds its own `.dag` contract on a discriminating corpus. **The bar is that v2 is better than the seed, not that it resembles it** (ruling `gunbc.design_ruling v2_better_than_the_seed`). The seed is a regression oracle, never a specification: a divergence is either a *v2 regression* (a defect) or a *seed defect* v2 does not carry forward and records as a correction. The seed reading something one way is evidence of what v2 must not lose, never by itself why v2 reads it that way — so neither a byte-identical fixed point nor behavioral equivalence is the goal; both would cement the seed's warts.

The seed shrinks across a **typed self-host frontier**: each module is *self-emitted* (green by execution) or *seed-retained*, and a retained module is a declared row with reason and migration trigger, never a silent escape hatch. Its tests are data; its ontology dissolves into `std/`.

The payoff is that **language design opens up.** It is normally locked by cost: a new check means owning a compiler fork, a new language means an adoption problem. Here a wall is a **row** (no fork), and because the substrate is medium-agnostic, that row applies *on top of an existing language* (no adoption): a domain's bug class can be made extinct in Rust or TypeScript without forking their compilers. It is sound exactly where ingest is lossless and fail-closed where it is not — any language, with a typed honesty boundary about where the wall holds. This is §1's convention-to-necessity at the meta-level, bounded by §5 (decidability) and §6 (displaced cost, not elegance).

## Recurring failure modes (instances of §3–§5, kept for pattern-matching)

One file per class under `dag/gunbc/recurring_failure_mode/` (`gunbc.recurring_failure_mode`), each with its recognition rule and receipts. They are a ledger, not a consequence, so they live there and are not indexed here: an index in this file would regenerate it on every append.

## Building & checks

The CI contract. From a bare clone, start at [docs/onboarding.md](docs/onboarding.md) (`gunbc.contributor_onboarding_path`): the walked path to a first landed change, the CLIs, local checks, hooks and regeneration, and a reading order for this document.

- **CI is one required job.** `gunbc.compiler_gate_workflow` emits `.github/workflows/witnesses.yml`; the required context is the job `witnesses`, run on the fleet for every pull request and merge_group (ruling `gunbc.design_ruling v1_withdrawn_one_required_job`). It builds the seed, then runs one `gunbc test <label>` step per row of `gunbc.emitted_subject_build_gate` `emitted_subject_build_rows` (today `//gunbc/instruments:self-host` and `//gunbc/instruments:v2-native-cli`): the seed emits a v2 closure, cargo builds it fresh, and the built binary runs against its controls. A fork pull request has no `witnesses` check — unobserved, not excused.
- **The memory envelope is part of the verdict** (ruling `gunbc.design_ruling memory_envelope_is_part_of_the_verdict`). Each subject step refuses a run no cgroup `memory.max` bounds, prints a memory receipt at exit, and refuses on any OOM kill or any swap whatever the producer answered (`gunbc.memory_envelope`; slot shape `gunbc.runner_slot_desired`).
- **What a green `witnesses` does not say.** Nothing on the required path evaluates a witness, refuses a module outside the two emitted closures, runs clippy or the v1 unit tests, or compares a generated artifact — this document included — with its authority. That loss is one declared drop, `gunbc.rung_drop` `v1_required_lanes_withdrawn`, whose population is re-derived by `gunbc test //gunbc/instruments:required-lane-resolution-census` and whose trigger is the native compiler judging that population. A new verdict joins the gate as a row judged by a binary built from the seed's emission, never as a lane handing the corpus to the v1 interpreter.
- **The job roster is closed to growth** (ruling `gunbc.design_ruling job_roster_closed_to_growth`): adding a job needs operator sign-off; the reasoning an author owes is stated at `witness_floor_lane_jobs`.
- **Measurements are `gunbc test <label>`** (ruling `gunbc.design_ruling instrument_is_a_row_not_a_flag`). A label is a row in `gunbc.instrument_targets` bound through `gunbc.target_binding` `TargetProducer`, never a flag on `claim_executor`. Exit 0 means the observation held, 1 that it did not, 2 that there was no observation (an unknown label refuses with 2).
