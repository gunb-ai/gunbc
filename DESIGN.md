# gunbc — Design

`README.md` and `CLAUDE.md` symlink here — this is the single source of truth. (v1 ships the `gunbc` CLI and is v2's seed · v3 was removed, migrated into v1 · v2 is active.)

This document is reasoned **serially**: §1 fixes the axioms, and each later section is a consequence of the ones before it (or an independent peer) — never a restatement of it. The principles apply recursively, including to this document, which is a projection of `gunbc.design_document` and is never hand-edited.

It carries the REASONING only. The two rosters that used to live here — the recurring failure modes and the declared §4b(3) rung drops — are projected into [docs/design-ledgers.md](docs/design-ledgers.md), because a ledger grows without bound while these sections are fixed by §1, and this file is loaded in full on every turn of every session. Plans and analyses live under `docs/plans/` and are linked from the section that governs them.

---

## 1. The objective (the axioms)

Three **axioms** — assumed, not derived:

- **A1 — there is a goal.** To *solve a problem* presupposes an agent with a goal; with no goal there is nothing to optimize.
- **A2 — time is the value.** Every agent intrinsically values time — it is finite and is the substance of acting at all: **time is life**. It is the one value we may assume is *shared*.
- **A3 — agreement is temporal.** Intersubjective agreement is possible only *across* time, and only on what stays stable under it.

Three things follow, in order:

- **From A1 and A2 — the solution is minimal, safe, efficient.** A solution is good exactly insofar as it spends less of the only thing valued, and time is spent three ways: **cost** (time to run), **safety** (time to recover from a silent wrong answer, paid later at interest), **complexity** (time to change). These are not preferences layered on — they are A2 applied to A1, and the project optimizes them jointly.
- **From A2 and A3 — grounding is intersubjective.** Because time is the only assumed-shared value, a fact is grounded only by pointing at a shared, time-stable framework (§4), never an internal taxonomy.
- **At the limit — reduce intersubjectivity to physics.** The deepest such framework is physics. So the aim is to replace convention with necessity until nothing arbitrary survives. This is why §3 models the universal frameworks and real upstream rather than re-coining them: a nickname is convention standing where physics was available.

The safety axis carries its own measure, used throughout: **safety is measured by how far each error class has climbed from runtime harm toward structural impossibility** — the guarantee ladder of §4b.

## 2. Minimize redundancy (the master move for cost and complexity)

Redundant work — **duplicated, unnecessary, or irrelevant** — loses on all three of §1's quantities at once: it costs more to run, it widens the surface where harm hides, and it adds complexity to maintain. So a perfectly DRY process is, by the meaning of redundant, the minimal one. Through §1's time lens this is *why* DRY matters: redundant work **defers** cost into the future — it shoves a problem onto a later fixer, or builds a process destined to be thrown out — so DRY is the refusal to spend someone's future time to buy the author's present convenience.

Redundancy is removed along two directions — one move seen two ways:

- **Horizontal — one concept, every scale and breadth.** Model a concept once; derive every use. At the right layer there is nothing fundamentally different between scales, so the same concept spans nanosecond memoization and broad infra deployment. *e.g.* `dag/std/integer.dag`: `Int8`…`UInt128` are 10 `Compose<Int, MachineWidth<N>>` rows — one axis, not 10 types.
- **Deep — every concept decomposed to grounded atoms.** Nothing is opaque that is not *genuinely* atomic. The move is `decompress → map → reduce`: reveal the structure the source names, map each part onto the concept that already exists (DFS the concept DAG first), reduce duplicates. A `String` leaf hiding named parts is anemic modeling. *e.g.* `"LGA4926"` → `CpuSocket { package: LandGridArray, contact_count: 4926 }`, where the number is a grounded `Int` rather than a fresh enum.

**Minimize the demand graph before materializing its answers.** A repeated computation is first evidence about why the same semantic fact is demanded again, not yet a cache obligation. When several demands have a shared-state least common ancestor, the repetition is authored duplication: carry, rewire, or share the first value — caching a later request only makes redundant work cheap and suppresses the signal that would make it rank for deletion. One undeclared pure demand may recompute; isolated consumers, declared replay, or unbounded future siblings instead create one reuse obligation at their least common visible ancestor. One computation identity must join those demands to a provider whose scope reaches that ancestor, whose coverage includes the identity, whose key represents every declared input the result depends on, and whose retention spans the obligated lifetime; otherwise refuse with a typed cause. Demand minimization decides whether another production may exist, and cache purity decides whether a materialized value denotes the same fact. Only after both hold does §6 choose the economically optimal realization: unavoidable recurrence may explicitly recompute below the cost floor, and a provider is an optimization only when measured total serving cost is below recomputation. The structural authority is `std.materialization_ladder`; a cache may discharge unavoidable recurrence, and may never excuse authored duplication.

The test that an edit actually *reduced* redundancy rather than moving it: **net concepts must not grow by re-invention.** Decomposing a leaf by minting a fresh authority for a concept that already exists is a failed decomposition.

## 3. Single authority (what keeps §2 from being undone)

Minimization holds only if each fact lives in exactly one place. The recurring violation is **nicknaming — a second name for one concept** — which duplicates work at the meaning layer and, since we generate from concepts, duplicates it again in everything derived. A fork always gets consolidated later, so it is a correctness concern, not a style one. Until it is enforceable this is diligence: faithfully model the accepted universal frameworks (classical logic, set theory, algebra), and in `extdeps/` the real upstream spec — cite the source, keep its real names, declare its version, model what the API actually returns.

Single authority applies to meaning, not only to code symbols. Nicknaming gives one meaning two names; its dual, a **meaning fork**, gives one name two materially different meanings. Product names, service names, tier names, status names, and terms in every layer are semantic carriers: within one naming surface and one declared effective version or epoch, holding the name constant may not silently change the obligations, quality floor, refusal behavior, billing consequence, or remedy — while the same spelling in explicitly distinct scopes, or across a declared version transition, is legitimate reuse. A materially different contract needs a materially different name.

Two corollaries. The import graph's only structural law is **acyclicity**: the `std`/`extdeps`/`compiler`/`workflow` folders are browsing conventions, not a direction rule. And a fact's home is its *layer*, not its file — paths are discriminators, not gospel.

The load-bearing consequence is that **interface, realization and policy are three facts, not one row.** For a dependency: the *interface shape* is what `extdeps/` owns; the *transport* (shell, REST, SDK) is a §2 Realization handler bound to that shape, one of N, never a fact about the dependency; and *business policy* — which base ref, which flag, an idempotency protocol the dependency does not provide — is a workflow fact, and modeling it in extdeps is a layer inversion. The tells are mechanical: policy has leaked when an argv carries a literal it should receive as a parameter; transport has fused when one operation is forked once per transport instead of one shape with N bound handlers. Its direction is always the same: **the dispatch that selects a realization is itself realization**, so it sits peripheral, never in the interface.

*e.g.* one "vendor" concept once forked three ways by rigor — `CpuVendor`/`DramManufacturer` closed enums vs a stringly `GpuFacts.vendor` — now dissolved into a single generic `Vendor<Domain>` whose cited company entities live one-file-each in `extdeps/vendor/`. Likewise `std/os.dag` projects per-vendor rows with no `match`, while the cited rows and the product-to-vendor dispatch live in `extdeps/os/`.

**§3 standing rule — cite the symbol, not the position.** A `file:line` pointer is a second, positional naming scheme for something the containment tree already names, and it decays in a way a nickname does not: any edit *above* the cited line silently invalidates it. Name the module and symbol; add a position only where no symbol exists to name — a generated artifact, a data row, a line inside a blob — and where a position is carried, it is a convenience beside the symbol, never the citation itself. Grep-verifying a symbol is the same `Node`-tree read the namespace authority already performs, so a stale name is decidable and enforceable; a line is not reachable from that tree at all.

### §3 standing rule — replacement migrations cut over at the root

When two structures answer the same semantic question and one is intended to disappear, the change is a **replacement migration**, not a refinement. §2 prices an intermediate representation destined to be discarded as redundant work, and §3 forbids X and Y both answering for one fact. Editing X from its leaves upward preserves its root while every intermediate state becomes observable and load-bearing, so the migration creates more authority before it removes any. The deeper cost is that a surviving X is an **attractor**: while it stands, every nearby question is answered in its vocabulary, so decisions keep being premised on an assumption already scheduled for death — and A3 grounds agreement only on what stays stable across time, which a dead assumption by definition does not.

So the default is **delete-first**: uproot the root as early as you can, then fix forward minimally with the new solution, solving each surfaced problem from first principles rather than restoring it to its old binding. In a fail-closed substrate **the deletion is the census** — every real dependent refuses loudly, so what breaks is exactly what was load-bearing. What cannot break loudly is covered by a declared, bounded §4b rung-drop, never by silence. Two carve-outs: a gap-intolerant boundary keeps the staged form — Y built in shadow, then one transition that switches the root and deletes X together — and where no Y can hold the boundary at all, X stays but stays **frozen**: no new investment, no new rows on its growth surfaces.

**"Atomic" describes the authority transition, not the amount of implementation work.** Y has one coherent root, ordinary production never accumulates consumers that understand both representations, and X's authority ends in one motion. Y may freely reuse stable primitives, and X may serve from history as an offline differential oracle — but Y never falls back to X, resolves through it, or derives its answer from it. And the minimum Y is not the smallest thing that executes the happy path: it must preserve every required refusal, or it has erased a correctness distinction rather than completed the replacement.

The frozen-X carve-out is the wrong state for a production-critical X whose replacement was paused indefinitely, which is the v1 seed's situation. v1 is therefore reclassified — semantics frozen, maintenance active — under a PURPOSE admission test: a change is admitted when it serves the v2 self-host program, and v1 stays closed to growth for its own sake. The authority is `gunbc.v1_maintenance_standing v1_seed_standing`, which also records the standing danger that an active-maintenance arm becomes an absorbing one if every proposal classifies as admissible, and states its own rung honestly: the vocabulary is consumed by review diligence, not by any gate.

The full doctrine — the operative loop, the consumer-relative root, the disposition census, the greedy root-first cut search, the terminal receipt — is → [replacement migration doctrine](docs/plans/replacement-migration-doctrine.md), with its cut programs at → [floor-cut](docs/plans/floor-cut-replacement-plan.md) · [namespace-cut](docs/plans/namespace-cut-replacement-plan.md).

### External upstream decomposition

“One concept, one authority” unifies a shared interface or formal shape; it does not merge independently governed entities that inhabit that shape. Each independently versioned upstream product, implementation, engine, vendor, or specification has its own extdeps module authority. A generic hub may define or re-export agnostic shapes, but it may not enumerate concrete products, dispatch among their authorities, carry product-specific version rows, or store consumer coverage state.

Observations produced by this repository are receipts in the observing product or workflow layer, not facts owned by the observed upstream. Missing observations are coverage obligations downstream, never `Unobserved` properties authored inside an upstream module.

```
extdeps.whatwg.html_navigation        shared standard
extdeps.browser.chromium              Chromium implementation
extdeps.browser.google_chrome         Chrome distribution
extdeps.automation.playwright         automation dependency
gunbc.served_surface_browser_support  support policy and evidence join
```

A Chrome change must not require editing Safari’s module. Adding Opera must not widen a generic browser-product enum.

## 4. The closed, grounded substrate (what makes §2–§3 decidable)

You can unify and decompose *mechanically* only in a closed, grounded system. A program is a dependency graph over two primitives (`Node` + `Edge`) and a closed vocabulary: 6 connectives and 6 behaviors — `Value`, `Transform`, `Branch`, `Loop`, `Bind`, `Match`. Surface syntax is sugar that adds no power. Execution is **bounded and forward** (cyclic relations via acyclic encodings, never cyclic values; recursion is sugar over `Loop`), so decidability and termination *fall out* rather than being separately proved.

**Grounding is intersubjective** — point at a shared framework, not an internal taxonomy — and in a closed system **a heuristic is never necessary**: the richer source always exists or can be written. This is why the structure is acyclic, in the substrate and in this document: agreement holds only across time, which demands claims that stay stable under it, so each is written as a consequence-chain you can re-interrogate, never a cycle that could quietly redefine itself.

Because the substrate is closed and grounded, the wins of §2 fall out for free: operations come from *inhabitance* (no per-type ops), and emission, ingestion, and coercion are **one** total decision procedure run in different directions — N models, not N×M adapters, every refusal a located, typed mismatch.

- *e.g.* `dag/std/algebra.dag` derives `Int.add` from `Int` inhabiting a ring; termination is *checked, not discovered* — `DescentEvidence = Strict | NonIncreasing | DescentUnknown` inhabits a `BoundedLattice` with bottom = fail-closed.
- *e.g.* **one grammar, read in both directions.** Ingest selects a production forward to fold surface syntax into a core `Node`; emit selects from the *same* rows backward — the structural inverse, not a second emitter — so a new target language is rows authored in `extdeps/languages/`, never an edit to the fold. Coercion is that same move turned sideways: whether one model inhabits another is a homomorphism check. One procedure asked in three directions, not three procedures.

## 4b. Safety: the guarantee ladder

Safety is not the presence of diagnostics or the absence of crashes — it is the reduction of the state space in which a program can silently do the wrong thing. Every discovered error class sits on one ordered ladder and is obligated to climb it:

1. **mitigatable** — the failure occurs; harm is contained by total operations, typed outcomes, bounds, rollback, isolation.
2. **mechanically preventable** — a generated test, lens, or gate reliably exposes and blocks it, but the invalid state remains writable and safety depends on that mechanism executing and staying enrolled.
3. **structurally guaranteed** — the source can still describe the invalid state, but no `Accepted` program contains it: the compiler derives a proof or refusal from modeled structure.
4. **structurally impossible** — the invalid state has no constructor in the canonical model. Validation is unnecessary because the bad state cannot be written.

The higher rung subsumes the lower — construction over proof, proof over validation, validation over mitigation. Below the floor sits **silent wrongness, which is not a rung: it is outside the ladder and forbidden outright** (§5). Adjacent, and deliberately *not* a fifth rung, is **outside the modeled guarantee**: external reality, undecidable properties, undeclared intent. That column is observed, refused, or mitigated at a declared boundary, never fabricated — keeping it off the ladder prevents "we do not model this" from masquerading as a weak implementation that should climb.

**At the top rung, ask whether the check's RED is authorable before writing the check.** If the forbidden state cannot be expressed anywhere the check could run, the check is not a weak wall but a decoration — permanently green by construction, carrying no information, and worse than absent because it will be cited as coverage. That question has two boundaries and only the second decides it: a state unrepresentable in the ACCEPTED corpus may still be perfectly representable as SOURCE HANDED TO THE COMPILER BY A FIXTURE, and a compiler is precisely a thing whose regression probes are invalid programs. If the refusal is still authorable there, the evidence is enrollable there and declining it is specification-without-execution. Where no fixture harness can express the subject, that missing harness is the class's next-rung trigger, not a permanent ceiling.

Every class carries an **attainable ceiling — derived, not aspirational**: a decidable, fully modeled class may reach structural impossibility; a decidable class missing its authority is §5's *wall after grounding*; a class blocked on a missing language capability names that capability as its trigger; an undecidable property honestly remains a ratchet or validator; an external fact remains a boundary obligation. **Below ceiling is a correctness gap, never optional elegance** — this is what keeps §6's purity trap from pricing out mandatory walls.

Four meta-obligations make the ladder operational:

1. **Rung honesty, at a declared subject grain.** The reported rung must equal the rung established by executed evidence — a discriminating RED refused on the real acceptance path plus an accepted positive control — measured against a declared boundary. Source→interpretation, source→each emission target, and phase-carrier→phase-carrier are different paths with independently different rungs; a class's current rung is the **minimum across its in-scope paths**. Citing the strongest path while another stays silent is inflation, and a type name, diagnostic variant, inert lens, or plan establishes nothing. **Rung inflation** is the fabricated-plausible-output failure applied to the compiler's self-description — worse than sitting low, because an inflated class never ranks for climbing.
2. **No untracked stall.** A class below its ceiling must name its next-rung trigger, separating *cannot climb further* from *can climb after one grounding* from *can climb now but unbuilt*. Only the first is permanent.
3. **No silent regression.** A change may lower a rung only by declaring previous rung, temporary rung, reason, bounded population, and restoration trigger. A compatibility exemption is not bootstrap glue; it is a visible safety regression with a finite runway. **The trigger must name the CAPABILITY, not an artifact that would contribute to one** — a declared drop is retired by its trigger and by nothing else, so a trigger naming less than the capability it restores will be satisfied while the capability stays dead. Where a trigger names an artifact, the row must state what that artifact must be SUFFICIENT FOR. The review tell is a grain mismatch between the loss sentence and the trigger sentence: a plural loss with a singular trigger, a corpus loss with a per-module trigger, a route loss with a single-call trigger.
4. **Dissolution on climb — production handling only, never the evidence.** A climb deletes the redundant lower-rung *production* machinery it obsoletes, but the class's discriminating RED and positive control **remain enrolled** as the executing evidence that the higher rung stays real. An expecting-red probe that greens when its wall lands flips to a permanent regression control; it does not retire.

At a service boundary, rung honesty has a commercial consequence: a dimension may remain opaque only above a falsifiable quality floor with a named consequence. A claim that can change billing, trigger a remedy, or require refusal is a contract; without such a consequence it is marketing and establishes no rung. Delivery below the floor must refuse or discharge the remedy; a materially different delivery may be admitted only as its own named and priced product — a different contract subject, not a declared drop of the premium one; and a temporary loss of the ability to verify or enforce a floor is a §4b(3) declared drop on that same subject, which may force the commercial path to refuse and never authorizes silent below-floor delivery. Deviation is allowed; silence is refused.

Every newly discovered error class — incident, review finding, runtime exception, falsifier divergence — files or updates one row: invalid state, harm, distinguishing facts, rung found at, ceiling with reason, next trigger. Declared drops are rostered in full — previous rung, temporary rung, reason, population, restoration trigger — in [docs/design-ledgers.md](docs/design-ledgers.md), authority `gunbc.rung_drop`. A drop is retired BY ITS TRIGGER AND BY NOTHING ELSE, so the trigger is the whole check. The ones standing today:

- **Heal job for generated artifacts** — declared 2026-09-01
- **Six of the seven effect gates (one restored)** — declared 2026-09-01
- **Rust fmt as a merge-path gate** — declared 2026-09-01
- **Merge-admission stamping: the receipt has no producer** — declared 2026-09-01
- **Falsifier cadence: exposure open, disposition NOT decided** — declared 2026-09-01
- **Stable per-claim cost qualification under a shared execution envelope** — declared 2026-09-01
- **Emitted-bytes fixture witnesses in a required lane** — declared 2026-09-01
- **Direct-call argument TYPE-COMPAT judgment inside v2.* modules (one of two arms; inhabitance still runs)** — declared 2026-09-01
- **CI required-run composition** — declared 2026-08-15
- **Self-host emission board measurement** — declared 2026-08-24
- **Blocking emit-stage diagnostics on main** — declared 2026-08-25
- **Corpus-wide lens enforcement censuses** — declared 2026-08-11
- **Required gate reduced to the compiler floor** — declared 2026-08-29
- **Non-literal kernel-String refusal at the structural text boundary** — declared 2026-08-30
- **Fabric CI evidence lane as a required merge block** — declared 2026-08-31

**The floor first, the differentiator above it.** gunbc must first hold the ordinary compiler floor — names resolve, applications bind in exact bijection, values inhabit declared types, fields exist, closed variants eliminate exhaustively — and a failure there is a below-baseline safety regression, never compensated by higher-order capability. The differentiating claim begins above that floor: because the substrate carries causal, cardinality, algebraic, effect, ownership, cost, and realization facts structurally, the same ladder applies to classes ordinary compilers leave to tests, review, profiling, or production postmortems — a possibly-empty collection flowing into a nonempty consumer, recursion without a descent proof, a non-idempotent effect under automatic retry, a computation exceeding its declared complexity bound, a realization that does not preserve modeled behavior.

The promise is **not** that every property becomes impossible; it is that every modeled class climbs to the highest honest rung its facts and decidability permit, with the ceiling and the residual risk explicit. So, stated beside the ladder: the compiler does not invent unstated intent; does not prove unmodeled external reality; does not lift arbitrary predicates to proof; and richer type names are not safety — a brand, wrapper, or `Validated<T>` is cosmetic until construction and acceptance enforce the distinction. Runtime mechanisms — typed refusal, totality, rollback, budgets — remain real and necessary at honest boundaries; they must only never be mislabeled as construction. The rung census and climb plan: → [compiler-guarantee recovery gap analysis](docs/plans/compiler-guarantee-recovery-gap-analysis.md).

## 4c. Source annotations (prose the substrate can see)

**Source annotations are captured authored-source data, not semantic program data and not discarded trivia.** A language realization that admits annotation syntax routes it through an annotation-specific lexical channel; it may not produce an ordinary semantic token or namespace binding. Annotation capture is disjoint from semantic occurrence allocation: adding, deleting, or moving an annotation cannot alter any semantic occurrence identity, semantic graph, resolution result, semantic hash, or target-program bytes. Semantic compiler passes receive only the annotation-erased projection. The initial `.dag` realization admits only standalone leading `//` blocks attached to module-scope declarations; trailing, body, unattached, and block-comment forms refuse until separately modeled.

**Prose is not forbidden; unclassified prose is.** Any invariant, receipt, event, ruling, citation, status, count, or dissolution condition belongs in a typed carrier; an ordinary `String` declaration whose sole purpose is commentary is misplaced or dead data. The rule exists because the opposite was tried and measured: removing `//` as a parse error made comment *syntax* unwritable without making commentary unwritable, so the corpus hoisted comments into `data …: String` rows — where intent is mechanically indistinguishable from program data — and the first cleanup swept 215 dead prose rows across ~130 files. `//` is therefore the explicit quarantine boundary.

Two consequences bind authors: an annotation may preserve irreducible human rationale about *why* a construction has its shape, and it must not restate what the declaration structurally says; and an annotation is never evidence that a machine claim holds, because no `Accepted` program can read one.

## 5. Fail-closed (§1's safety axis)

Minimizing cost and complexity is worthless if a wrong thing passes silently. This code is digital: a wrong answer is a **loud error, never a warning** — a bridge collapses, it does not warn. Every path succeeds fully or fails with a typed, located diagnostic; no fabricated plausible output. Relax toward application-layer leniency only under protest, and lean to infra so others can build on your work.

Stronger than *catching* a wrong state is making it **unwritable** — **correctness by construction, not validation.** A check that re-states a constraint the model already carries is a second representation of it (§2/§3), so prefer a single authority from which the realization is derived over a check that flags the bad state after the fact. The tell that a check was validation standing where construction was available: it can be satisfied by editing the *declaration* while the realization still lies.

Construction makes a class unwritable only when membership is **decidable**, so every class is one of three: a *wall now* (decidable and grounded); a *wall after grounding* (decidable but waiting on its single authority); or a *ratchet forever* (undecidable — optimality, by Rice, never reaches "never"). The word **"never" is the trap**: it lets a ratchet masquerade as a wall, so check decidability before claiming one.

The deepest trap is **specification-without-execution**: a typecheck and a `.contains()` grep are not consumers. "Done" means a real consumer **green by execution** plus a discriminating input that goes red when the behavior is wrong. For the LLM agent especially: fluent, type-checking, grep-passing output is precisely the artifact that looks finished without running.

The same demand for an independent referent governs a test's oracle. **A merge-blocking test may compare a live repository population to a numeric literal only when that literal is grounded in a controlled fixture, external or versioned authority, explicit policy budget, or a monotone debt contract over a closed subject universe. A measurement copied from the same current tree is not an oracle. Completeness is an identity join, not a count equality.** A monotone debt contract is legitimate only when the subject universe is independently discovered and closed, membership is checked at identity grain rather than by count, and every removal carries a typed disposition. Review tell: if automating the literal's update collapses the assertion to `measure() == measure()`, the manual update was the test's entire content — a change detector, not a check.

A third named trap, the subtlest because it wears this section's own name: **the absorbing fallback — degradation is disguised fail-open.** When a mechanism cannot compute its precise answer, the tempting arm substitutes the *superset*: can't compute the affected set → rerun everything; cache key uncertain → scan all keys. Nothing is missed, so the arm gets labeled fail-closed — but it is **⊤-as-answer conflated with ⊤-as-ignorance**, and it fails open twice. On safety: absorption destroys the only signal that the precise mechanism has a deficit, so the deficit never ranks for fixing and the anemia compounds. On cost: the fallback is denominated in the *corpus*, not the *change*, so it grows until the budget breaks instead of the build. The confidence threshold that selects such an arm is a smuggled heuristic, so its existence *locates* the anemic modeling it papers over. **The rule, and the review tell: a failure arm must refuse, never widen** — every degradation a typed, located, countable diagnostic. Two neighbors are not this pattern: a structural over-approximation computed *as* the answer, and a deliberate interim fallback that is loud, budget-bounded, and lands with its dissolution trigger.

When §2's deferred cost is moved from the actor that accepted or caused it onto another principal, it becomes **externalization**. A risk intermediary or cost-causing actor fails open across an accountability boundary when it quietly re-exports an accepted risk or leaves a caused cost unpriced while preserving the apparent name, price, or contract — onto customers, employees, sellers, liquidation buyers, neighbors, or future maintainers. There are only two honest arms: absorb the risk and reserve for it, or expose the transfer as a separately named and priced contract; a materially degraded service is therefore its own product. Keeping the old name, price, or contract while another principal bears the changed burden is **externalized degradation**.

A corollary on the refusal itself: **no escape hatches** — a toggle whose only effect is "proceed as if the refusal had not fired" re-opens the arm §5 just closed. The operative discipline is the **factory model**, a merge requirement rather than a preference: a deficit stops the line; the stopped line is analyzed before it restarts; the only sanctioned second mode is a stopped-line audit that replays the run to ledger every deficit for that analysis — it reports, it does not green. **Review bar:** a diff that lands a non-fail-closed failure arm — a silent widen, a fabricated default, an uncounted degradation, an escape hatch — is a **hard reject**, regardless of what else it delivers. The reviewer's three questions: does the line stop? is the stop typed and located? does analysis precede restart?

The same arm exists at authoring time: **the workaround — an absorbing fallback executed by the author.** When the obstacle is the substrate itself — a parse error you do not understand, a check that will not green — the tempting move is to route around it: a different spelling, a dodged codepath, a "for now". The concealment is identical, except the concealed deficit is usually in the *language layer*, precisely what §6 says to root-cause first. So: **noticing you are implementing a workaround IS the line-stop signal.** Back up, reassess, root-cause it or flag for help.

**A dissolution condition describes how admitted debt ends. It does not authorize creating the debt.** The default and expected landing state is the final construction; a scaffold is an operator-approved exception, not an ordinary alternative. A throwaway artifact costs authoring plus review plus maintenance plus deletion plus review of the deletion, against a benefit bounded by its lifetime — so if the modeled path already exists the benefit is zero and the trigger was ceremony. Approval is external to the diff, because an author who can write a scaffold can equally write a row claiming it was approved. The verdict vocabulary and the out-of-band-actuation case: → [scaffold admission doctrine](docs/plans/scaffold-admission-doctrine.md).

## 6. How to work (given §1–§5 — these coexist)

- **Model:** DFS the concept DAG before inventing vocabulary; invent or reuse on proven coincidence, never bare-alias; a finished stage is one fold (any non-fold residue is either a named irreducible kernel or un-migrated modeling); model just-in-time and let the mark on the carrier be the authority, not a parallel-ledger doc; no scaffold lands by author declaration alone.
- **Intellectual sustainability:** do not spend future author, reviewer, operator or maintainer time to buy present convenience. Work expected to be thrown away is *presumed redundant*, so back up and complete the construction that will survive. The reviewer's independent test, applied whether or not the author labelled anything: **will this artifact survive the terminal architecture substantially unchanged, and be consumed by it?** If no, presume scaffold and stop the merge. A *missing* dissolution condition makes the finding **more** severe, not less; adding a trigger after review does not resolve the objection, it only makes the proposal eligible for a decision.
- Tells, each carrying a presumption: a hand-authored workflow, deployment or migration script (out-of-band actuation); a second path beside an existing modeled route (parallel authority); "bridge", "shim", "for now", "temporary", "until", "later" (deferred refactor); a model to be deleted whole when the real one lands (throwaway model); a hand-authored projection the model should generate (manual application committed as source); raw shell implementing semantics already expressible in `.dag` (unmodeled realization); a broad wrapper around a type or modeling deficit (workaround hiding substrate work); a condition whose terminal is "rewrite this properly"; a new artifact with no final consumer (experimental residue).
- **Prioritize holistically, not by the bottleneck:** balance the quantitative and the qualitative — do not anchor on one KPI or on pure taste. A 5ms step does not get a pass for not being the 80s one; it might be a 5ns step. One consequence is a standing rule, **bare minimum cost**: a proven cost-shape defect — a copied accumulator, a quadratic fold — is *always fixed*, regardless of the realized n. "n is small here" is not a time-stable fact (reuse changes n), and pricing per-site exceptions is itself redundant work.
- **Denominate the benefit:** the deliverable is a *displaced cost* — a pain someone pays to remove; the lens or substrate is the *mechanism*, not the product. Priced in elegance instead, the work is self-referential and unbounded (the purity trap — the economic twin of "never" in §5).
- **Enforce with lenses,** not grep — but **construction first**: a lens is validation, so make the class unwritable by single authority where you can and reserve the lens for the unstructurable residue. As a residue mechanism it earns its keep: a pure reader over the same `Node` tree, storing nothing, so a new analysis costs zero substrate edits. Beware the tier where the machinery exists but nothing gates on it — an inert lens is itself a lie.
- **Root-cause to the language layer** and fix related systems together — a local subsystem patch is the forked-logic trap. *e.g.* one catamorphism `fold_node` is reused by all 7 v2 stages; #4699 dissolved `06_translate` 4,912→3,973 lines. The symptom recurs wherever the root is unfixed: v2 still hand-rolls `ParseTable` because the Realization carrier is staged, not inhabited.
- **Name the instrument, never transcribe its output.** A measurement is cited by naming the producer that re-derives it — the run, the flag, the entry point — never by copying its numbers into prose, exactly as §3 requires a citation to name a symbol rather than a line, and for the same reason: a transcribed number is unreachable from the thing that owns it, so it rots without anyone touching either end. If a measurement is worth re-deriving it is worth an entry point; if it is not worth an entry point it is not an instrument but a one-off.

## 7. Self-hosting (the principles applied to the compiler itself)

The compiler is a pure transform and an ordinary substrate fact, analyzable by its own lenses. It is written in itself: the `.dag` graph is the truth and Rust is one realization — a seed that shrinks toward zero. v2 emits its own sources as the cleanest principled realization, proven **by execution**: the emitted module compiles and is behaviorally equivalent to the seed on a discriminating corpus. A byte-identical fixed point is explicitly *not* the goal — it would force v2 to reproduce the seed's warts to match bytes, cementing poor decisions.

The seed shrinks across a **typed self-host frontier**: each module is either *self-emitted* (green by execution) or *seed-retained*, and a seed-retained module is a declared row with a reason and a migration trigger — countable, prioritizable — never a silent escape hatch. Its tests are data. Its ontology dissolves into `std/`. This is the recursion: every principle above governs the system that implements them, and this document.

The payoff is that **language design itself opens up.** It is normally locked by cost: a new check means owning a *compiler fork*, and a new language means an *adoption* problem. Both dissolve here at once: a wall is a **row** (no fork), and because the substrate is medium-agnostic — one grammar read in both directions over many media — that row applies *on top of an existing language* (no adoption). So language design collapses from (compiler-fork × language) to (row + medium): a domain's bug-class can be made extinct in Rust or TypeScript without forking their compilers. It is sound exactly where ingest is lossless and fail-closed where it is not — "any language, with a typed honesty boundary about where the wall holds." This is §1's reduce-convention-to-necessity at the meta-level, and the place the purity trap bites hardest: bounded by §5 (decidability) and §6 (displaced cost, not elegance).

## Recurring failure modes (instances of §3–§5, kept for pattern-matching)

One row per class, each carrying its recognition rule and its receipts, in [docs/design-ledgers.md](docs/design-ledgers.md) — authority `gunbc.recurring_failure_mode`. They are rostered there rather than here because they are a LEDGER and not a consequence: every lane that finds a new class appends one, so the section grows without bound while this document's sections are fixed by §1. The index below is the classes; the ledger is the content.

- `censored_estimator_drops_its_own_tail`
- `selection_view_read_as_population`
- `unlanded_citation_indistinguishable_at_the_citing_end`
- `hollow_alias`
- `state_space_conflation`
- `absorbing_fallback`
- `empty_observation_narrow`
- `cache_impurity`
- `reflection_evidence_structural_proof`
- `coercion_proven_by_normalized_round_trip`
- `parallel_representation_debt`
- `internal_review_finds_missing_tests_external`
- `unmarked_workaround`
- `self_authorized_dissolution`
- `positional_citation`
- `bound_shaped_closure`
- `authority_substitution`
- `reachability_read_as_occupancy`
- `total_at_the_level_examined_blind`
- `execution_provenance_loss`
- `remediation_mutated_view`
- `diagnostic_name_mechanism_silent`
- `identity_absent_graph_traversal`
- `generated_binding_shadows_bare_render`
- `resolved_reference_outside_execution_closure`
- `surface_shorthand_preempts_resolved_identity`
- `meaning_fork`
- `externalized_degradation`
- `mistyped_body_radiates_nonlocal_diagnostics`
- `instrument_output_read_as_subject_content`
- `executed_conjunct_discriminates_nothing`
- `unbacked_execution_claim`
- `admitted_module_without_judged_standing`
- `mitigation_injected_where_judgment_declined`
- `transport_close_read_as_completion`
- `merge_region_excludes_shared_tail`
- `sealing_property_erases_structure`
- `disagreement_census_blind_to_agreed_wrong`
- `undecided_fraction_read_as_denominator`
- `restoration_promise_names_a_route_that_does_not_exist`
- `restored_bytes_reviewed_as_authorship`
- `review_summary_inverts_roles_and_affirms_the_join`
- `fabricated_debt`
- `accepted_source_emits_uncompilable_target`
- `incidental_denominator_as_wall`
- `compensating_errors_cancel_in_the_aggregate`
- `one_refusal_two_destinations`

## Building & checks

- Three local checks, each named with the CI step that executes it — a check named here with no executing step is a decoration (§4b): `cargo clippy --all-targets -- -D warnings` (`gunbc.repo_self_build` `repo_self_clippy_command`, the only command that compiles the integration-test and example targets, so a red there is invisible to every other step) and `cargo test --release -p v1-compiler --lib` (`repo_self_test_command`) run in the `rust-unit-tests` job of `gunbc.witness_floor_workflow`, which runs on every push and pull request but is not yet a `needs` of the required aggregate — promoting it is the one-row edit that job's authority names, gated on its measured wall clock; `cargo fmt --all --check` runs in the generated pre-commit hook (`gunbc.githooks_pre_commit_emit`). `cargo test --workspace` is local diligence only: no CI step executes the test targets outside `--lib`, they are compiled by the clippy step and run by nobody.
- one-time per clone: `git config core.hooksPath .githooks` — the only documented manual seed; generated pre-commit/pre-push hooks then idempotently converge `merge.generated-artifact.driver` and re-assert `core.hooksPath` via argv derived from `gunbc.repo_local_git_config` (clones that skip hooksPath degrade to vanilla text-merge for generated-artifact paths; drift gate still guards at CI). The driver REFUSES rather than answering `true`: git reaches a low-level merge driver only when both sides changed the path since the merge base — measured on a four-case matrix, one-sided and identical changes never reach it — and taking the ours side there dropped the other side's authority-derived bytes with no conflict, twice on #7836 against the stage0 seed. It now leaves the ours side in the worktree with no conflict markers, marks the path unmerged, and prints the regeneration recipe; the class is mechanically preventable, not structural, and its next-rung trigger is the commit-writer binding rows in `gunbc.commit_workflow`
- explicit actuator (CI / tooling): `gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/repo/repo_local_git_config.dag --function converge`
- **CI** is one emission, `gunbc.witness_floor_workflow` → `.github/workflows/witnesses.yml`, invoking our own binary once per LANE: `claim_executor --required-ci --source-root dag --source-root src/v2 --required-lane build` and the same with `--required-lane witnesses`, in two parallel jobs, plus a third aggregating job that carries the required context. Which phases a lane owns is decided in the binary, never in the YAML — the partition is an exhaustive match, so a phase belonging to no job fails to compile. **Read the roster from the run's own announcement, not from here:** every required run prints one `phase <name>` line per phase it owns and one `ROUTED to lane <other>` line per phase it does not. Several capabilities that used to gate are currently declared rung drops — see §4b.
