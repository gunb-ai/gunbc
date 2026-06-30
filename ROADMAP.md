# gunbc — Roadmap

`DESIGN.md` is the authority for *why*. This is the **shape of the work** — a scannable, dependency-ordered checklist. **Checkboxes are authoritative for progress**; detail lives in the linked plan docs — don't restate it here (no dual representations). A task's real state is its branch/PR.

Legend: `[x]` done · `[ ]` todo · **indentation = depends on the item it sits under**. Each section opens with a **◆ Milestones** spine — the *verifiable* checkpoints in dependency order (`✓` reached · `▸` now · `○` ahead); the checklist below is the work toward them. Read L→R = the path.

**Priority order, top = now.** Bands: **stability / correctness** (§0–§4) → **expansion** (§5–§7) → **shelved** (§8).

## ✦ Ergonomics LANE — make the fold the path of least resistance *(lead lane of the stability band, upstream of §0 — placed bright-stag-194)*

→ [charter: spine + ranked focus](docs/plans/fold-ergonomics.md)

Why a tier: **"it compiles but nothing works" traces to non-fold residue** — a hand-rolled `match` has a `_ =>` fail-open escape; a fold over a closed coproduct is total by construction and has none. So the chain is **ergonomics → adoption → fail-closed**: when the fold is awkward to reach, people hand-roll, and every hand-roll reintroduces a fail-open arm ([model↔realization fork](docs/plans/model-realization-fork.md) is the systemic instance — per-site bridges that should be one coercion fold). This lane **stops new residue by making folds ergonomic**; §0's fail-open-shape walls **retire the old**. The two together drain the fork. Guardrail (§6 — ergonomics is the #1 purity-trap magnet): every item **names the fail-open class or measured friction it retires** (displaced cost), never "cleaner."

**◆ Milestones:** staging/`then_outcome` combinator seeded ✓ (#5512 — compiler front-end de-pyramided to a stage fold) → **generic inference fixed ✓ (#5552, in queue)** + measure the residue (inert-abstraction lens) → fold reachable *by default* → new non-fold residue can't merge (pairs with §0 wall)

**Audit half — measure the friction + the residue (decidable; wall-able):**

- [ ] **inert-abstraction lens** *(keystone)* — flag any item *defined + self-tested + zero non-test consumers*; generalizes the inert-lens backstop (#5433) from lenses to all carriers. **First RED witness = `Placement` / `Materialization` / `RealizationObjective`** (charter §4: "modeled + witness-passing, no non-test consumer") — a genuinely-inert carrier the lens *fires* on day one, so the lens isn't itself inert (its own §6 guardrail). (`cached_stage` is the *resolved* case — now wired, see Fix half — so it's the worked example, not the witness.)
- [ ] **non-fold-residue audit** — `_ =>` catch-alls over *closed* coproducts · `unwrap_or_default` in inference · hand-rolled recursion where a fold exists. These are the decidable fail-open shapes → §0 wall candidates, not just lenses
- [ ] **fold-friction audit** — what makes the fold awkward to reach (the #5512 pre-state: generic fn-params mis-inferred as kernel `Witness`/`Optional`, forcing typed-param workarounds) — the friction predicts where new residue appears

**Fix half — make the fold the path of least resistance:**

- [ ] **generic-inference fix** *(fix keystone — #1)* — **▸ #5552 (keystone green-by-execution + red-on-revert witness, MERGED):** `= FreeMonoid<Symbol>` now resolves. The hand-roll collapse (`qualified_name` trio · `ParseTable`) + §0's realization-grounding (~120 `Value` bridges) follow in de-fork phase 2. Dissolve-on `feature:free-monoid-entry-generic-inference` ([charter](docs/plans/fold-ergonomics.md) §3, [de-fork brief](docs/plans/dsl-v2-defork-audit.md))
- [ ] **generalize the staging combinator** — `then_outcome` (Kleisli for the `Outcome` monad) seeded the pattern (#5512); lift it to the standard way to compose fail-closed stages, so a pipeline is a fold of typed stages, not a `bind_outcome` pyramid
- [ ] **wire the seams, don't strand them** — an abstraction lands *consumed* or scaffold-marked with a dissolution trigger ([construction-justification rule](docs/plans/construction-justification-rule.md), #5476). Worked example (the full arc): `cached_stage` seeded inert (#5512) → caught by the keystone lens → wired with a `Miss`-stub wrapping `stage_resolve`. **Boundary:** this lane owns only that the seam *lands consumed-or-marked*; **§1/§2 own *enabling* the resolve-cache** (the realization work) — one home each.

**Own runway — down-ranked (the lane's §6 guardrail applied to its own scope):**

- [ ] **ban source comments** (`.dag` + `.rs`) — *separate runway, sequenced BELOW the fold items.* Orthogonal to folds (it's documentation-hygiene), load-bearing (grammar `02_parse`/`syntax`), and the item most likely to swallow the lane. Displaced cost: reviewer reads multiples of the real diff (a comment-heavy `.dag` fn is ~10% code) + LOC inflation; that's the pain, not "cleaner." Construction arc: model the live-state survivors → migrate → **delete the rest aggressively** (git is the backup) → **parser refuses free `//`** (§5, lands LAST). *(calm-seal-13: pilot CI-gate files → fan out → wall; deletion has no modeling-blocker — no gate text-scans comment bodies)*

**Pairs with:** §0 (walls retire old residue; this lane stops new) · [model↔realization fork](docs/plans/model-realization-fork.md) (the residue's deepest instance) · §4 testgen (anemia/structure lenses are the same "measure the modeling" move).

## 0. Fail-closed lock-down LANE — BLOCKS expansion into products

Cache flakes, un-wired lenses, complexity violations = one problem: modeled, not made *impossible to write*. Fix = correctness by construction, not validation ([DESIGN §5](DESIGN.md)).

→ [audit + checklist](docs/plans/fail-closed-lockdown.md)

This window = a few days of STABILITY — shrink the fail-open surface, don't "lock" it. The deepest root (`Value::Null` overload, ~131 sites) stays OPEN until its own runway ([fork plan](docs/plans/model-realization-fork.md) §3).

**◆ Milestones:** fail-open audited ✓ · numeric tower grounded ✓ (#5428) → **▸ NOW: cache warm==cold oracle · every lens wired-or-deleted · stage0 census under budget** → `Value::Null` split *(deep root)*

**Audits (done):**

- [ ] lens/gate wiring — most analytical lenses inert (authored, no discovered gate) — ⏳ awaiting sign-off
- [ ] fail-open code — cache lossy-digest · under-keyed memos · `unwrap_or_default` infer — ⏳ awaiting sign-off
- [ ] **model↔realization fork — ROOT CONFIRMED** — one seam (~13 bridges); sub-roots = numeric tower + `Value::Null` [plan](docs/plans/model-realization-fork.md) — ⏳ awaiting sign-off
- [ ] remaining: coercion/equality straddles · inference fail-open · cache-purity · CI-coverage-completeness [detail](docs/plans/fail-closed-lockdown.md)

**In-scope this window:**

- [ ] **numeric-tower grounding** (#5428) — `Int=GroupCompletion<Nat>`; `==` straddle guard now dead-in-corpus [plan](docs/plans/model-realization-fork.md) — ⏳ awaiting sign-off
- [ ] **cache trustworthy** — authoritative home is §2 F2/F3/P1; ship the warm==cold oracle as a detective now
- [ ] **rust-gate coverage** (shared §1) — **▸ run-all at nextest speed ✓ CI-green-proven (#5427, in queue): full v1 coverage every `.rs` PR ~6m (was ~42m), no coverage-vs-speed tradeoff**; `.dag`→rust coverage wall = edge-(b), SCOPED / pending operator greenlight *(quick-ant-298)* [cause table](docs/plans/ci-selection-vs-scheduling.md) [edge-(b) brief](docs/plans/edge-b-rust-dag-provenance-brief.md)
- [ ] **promote-or-delete inert lenses · de-vacuum gates** — EmitHostGate de-vacuumed ✓ (#5477); 4 advisory lenses widened+bounded, whole-corpus deferred to `.dag` structural-reflection (also unlocks coverage/testgen) *(silent-wren-739)*
- [ ] **realization-vocabulary containment guard** (#5445/#5453) — target-AST importable only at the realization edge (fail-closed, shrinking-roster); dissolve-on: bash-sidecar arc empties the roster → pure wall → `program.dag` deletable [plan](docs/plans/emission-ingestion-inverse.md) — ⏳ awaiting sign-off
- [ ] **stage0 clone-census inert + seed regressed** to 21540 (~1138 over) — resolve by clone-reduction / substrate-migration, NEVER a cap-bump; #5427 `#[ignore]` is the interim *(fierce-hawk-540 via quick-ant-298)*
- [ ] **`Disposition` carrier** *(un-parked, operator GO)* — slice-1 prove-by-use active *(adhoc-0a633bef-bb9 / fierce-crane-13 under neat-dove-397)*: model `Disposition`+`ConstructionMechanism` in std → migrate one region (the post-#5579 `data:String` marker fleet) → fail-closed redundancy lens. Single-authority convergence home for scaffold/rationale/dissolution marks; substrate-mandatory #1 is the end-state, not this slice. [plan](docs/plans/disposition-carrier.md)

**Fenced OUT (after stability):**

- [ ] **split `Value::Null`** (None/Absent/miss/Violates → own carriers) — ~131-site substrate change, the deeper root; own runway
- [ ] **self-host purity gate** — a §5 deliverable, not §0 (avoids the §0↔§5 cycle)
- [ ] **cross-tree import activation** (§5) (#5473) — LANDED; the §0↔§5 escalate item is now closed — ⏳ awaiting sign-off
- [ ] complexity-budget whole-codebase (§3) · cache-redundancy completeness (§2 P3) — residue, after construction
- [ ] **cardinality refinement** — illegal cardinalities (wrong length · empty · overflow) unwritable by construction; the *decidable* refinement axis (linear arithmetic over counts), fold-propagated. MVP-1 (`Byte` via `Length<8>`) + P4 (fold homomorphism · uint8 overflow → typed `Rejected`) proven (#5512); P1 (`where` lowering) / P2 (construction-enforced) behind this lane; P5 (phantom-width reflection) substrate-blocked. [plan](docs/plans/cardinality-refinement.md) [P1](docs/plans/p1-where-clause-lowering.md)

**Meta — lock down the reasoning (§7 recursion):**

- [ ] **inert-lens hygiene backstop** (#5433) — every `lens/*.dag` wired or deleted; runs over the corpus — ⏳ awaiting sign-off
- [ ] **reachability-completeness lens** — every declared node (code carrier · doc · lens) reachable from a run-root, rostered, or deleted; generalizes #5433 to carriers + docs [plan](docs/plans/inert-layer-lens.md)
- [ ] **gate-hygiene: a floor-enrolled gate must be green-on-main at merge** — roster-completeness assertion promoted to should-land ([plan](docs/plans/emission-ingestion-inverse.md) §2; [merge-freshness decision record](docs/plans/ci-merge-freshness.md)) *(quick-ant-298)*
- [ ] **construction-justification rule** (#5476) (authoring-time) — justify why a class can't be construction before adding a lens *(silent-wren-739)* [plan](docs/plans/construction-justification-rule.md) [DESIGN §6](DESIGN.md) — ⏳ awaiting sign-off
- [ ] **expressibility frontier** — partition each modeling discipline into wall / lens-residue / undecidable-review *before* gating [plan](docs/plans/expressibility-frontier.md)
- [ ] **confront the skipped modeling decisions** — the `🟡` comment backlog *+ the post-#5579 `data:String` marker fleet* (the comment-wall moved marks comments→rows, growing the §3-migrate surface); converges on the `Disposition` carrier (single authority, `0-disposition`) [Disposition plan](docs/plans/disposition-carrier.md)
- [ ] **reference grounding** — migrate stringly-typed *references* (a symbol/module/fn/url written as free text where a typed edge belongs) to typed carriers (`DeclarationRef`/`QualifiedName`/`Uri`); the carrier half of the groundedness effort (detector half = `v2.lens.grounding` EXTRACT). Splits the coincidence into REFERENCE (independent entity → `DeclarationRef`) vs ROLE (genitive/ownership of the enclosing concept → an EdgeLabel/FieldRef tag, EARNED from an adjudication ledger, not guessed). Follow-on to `0-construction-rule`'s prose deletion *(tidy-badger-45)* [plan](docs/plans/reference-grounding-migration.md) [construction rule](docs/plans/construction-justification-rule.md)
- [ ] **axiom + syllogism lens** (DESIGN open thread #1) — every claim chains back to an axiom, no orphan/cycle; stays `[ ]` until it runs executably over this doc [scope](docs/plans/axiom-syllogism-lens.md)
- [ ] **self-applying lenses (detect → generalize → emit → write)** — a lens *produces the correct pattern and applies it via the write API*, not just flags; the §7-recursion upgrade to the whole lens family, unified by **redundant intent** (specification complexity above the essential min). Anti-unification is the shared engine (term-layer fold + `structural_similarity` type-generic = one kernel, two binders); seeded in `v2.lens.simulated_relationship` (#5584). Decidable-wall classes only — ratchet residue stays detect-only. Depends on §6 emit + a write effect + resolve facts [plan](docs/plans/self-applying-lenses.md)

**Dispatch discipline (anti-stale-ledger, §6):** a lever earns a lane only after it is re-measured against CURRENT main — a displaced cost already paid by another merge is the purity trap (e.g. rust-test sharding, ruled out post-#5427's nextest cut). And the lane/parked list is DERIVED from this authority, never hand-typed (a hand-list drifts exactly like a second representation). The authority tracks ALL planned work; work-items dispatch only the active subset.

## 1. CI as the substrate integration dogfood (the correctness floor)

A flaky or green-but-broken floor means no gate protects anything — so CI is upstream of every §0 claim. CI is also the one workload that flexes *every* substrate layer at once (execution · scheduling · caching · secrets/effects · emission), so it is the forcing function that turns each modeled-but-inert abstraction load-bearing. **Deliverable = shared abstractions proven by CI consuming them** (one Materialization kernel · one Placement authority · one secrets model); faster CI falls *out* of that, it is not the goal (§6 — price the lane in displaced cost, "move with confidence", not elegance).

→ [charter: causal-chain gap analysis](docs/plans/ci-process-end-to-end.md) — what's on `.dag` today vs not, push→execute.

→ [CI humming](docs/plans/ci-humming.md) — the throughput/wall-clock/reliability operations plan for the **▸ NOW** milestone: un-throttle runner slots from the modeled budget (fix the build-pool double-count), verified-effective caps, the carrier, SessionSliceEnforcement, oomd demoted to backstop.

**◆ Milestones:** execution-as-DAG ✓ (the floor *is* a bounded forward graph walk) · width on `.dag` ✓ (#5444) → **▸ NOW — host-operation on `.dag` (placement · runner deployment · caps are hand-managed, off-fabric)** · **floor wall-clock 26m → <1min (resolve-phase incrementality)** → resolve-cache enabled → one Materialization kernel (collapse the 5 caches) → one Placement authority (jobs · threads · sessions = 3 forks) → shared secrets · gunbhub closes the GitHub engine (G6, parked)

**Floor wall-clock — the measured gap (26m → <1min, profiled 2026-06-24):**

- [ ] **the floor is a full recompute every run** — profiled: the one `gunbc ci` step = **26m**, of which **~518s resolves all 870 witnesses** cold, a **second** `run_discovery_corpus` pass ~275s, effect-bound shell gates ~360s, seed compile ~134s. On a one-file PR ~99% of that resolve recomputes witnesses whose inputs did **not** change — §2 redundant work, un-inhabited. **Target: <1min on a typical PR.**
- [ ] **resolve is not affected-set-pruned** — discovery is deliberately tree-wide (fail-closed enrol), but EXECUTION still **resolves all 870** every run; #5427 selection shrinks compile/test, not the 518s resolve. The lever: extend the affected-set across discovery→resolve so unchanged witnesses skip (the floor-selection items below, applied to the resolve phase). [plan](docs/plans/ci-selection-vs-scheduling.md)
- [ ] **affected-set de-fork: `v2.lens.affected_set` as single authority** — N=2 parallel reverse-reachability impls exist: `.dag` authority (`v2.lens.affected_set` + `affected_set_floor_runner`, INERT, `WallAfterGrounding`) and Rust parallel (`NodeFrontierSeeds` + `entry_touches_frontier_seeds`, LIVE). The `.dag` authority must become the single live implementation; the Rust path deleted. BOTH witness-selection (per-row skip) AND precompute-skip (`precompute_whole_tree_published_mock_keys` at `cli_run.rs:3509`, unconditional today) wire to the same one `.dag` query. Acceptance: (a) `.dag`-vs-Rust equivalence on real corpus diff, (b) Rust rep deleted (N→1), (c) real floor wall-clock+peak-RSS drop on scoped diff. [plan](docs/plans/affected-set-precompute-pruning.md) — ⏳ awaiting sign-off
- [ ] **kill the within-run double-resolve** — the corpus resolves once for discovery and again for execution (~275s second pass); execution should piggyback the discovery resolve. Pure within-run win, no cross-run cache needed.
- [ ] **profile the 518s resolve** — ~0.6s/witness average, but a known machine-independent resolver bug resolves `budget_roster` ~450x its structural twin; a few pathological witnesses likely dominate the 518s. Profile which, fix the resolver.
- [ ] **fractal Gantt profiling receipt (#5757 baseline)** — per-claim timing tap (`GUNBC_FLOOR_GANTT=1`) + Batch 1/2/3 breakdown; hot spots: rust gate ~530s (host-effect, architectural sep), discovery eval ~229s (interpreter, `width=3`), cold resolve ~50s (resolve-cache lever). [plan](docs/plans/ci-floor-fractal-gantt.md) — ⏳ awaiting sign-off
- [ ] **interpreter memory chronicle (post-#5910)** — execution-grounded floor RSS receipt (width=1 @ 8 GiB: ~5.12 GiB self-RSS, ~5.31 GiB cgroup) + per-shard/per-resolve probes vs the 14.2→5.5→5.1 GiB arc; phase-0 measurement scaffold toward `PerformanceReceipt`. [plan](docs/plans/interpreter-memory-chronicle.md) — ⏳ awaiting sign-off

**The <1min lever is resolve-phase incrementality, NOT cross-run resolve-cache** — CI is cold-dominated (the exe-hash re-colds on every code change), so §2 P3 resolve-cache is the *warm* ~18% lever and was measured net-negative as the cold-CI path. Incrementality (don't resolve what didn't change) is upstream of caching (memoize what repeats); the <1min target rides the former. **This re-frames the §2 "core ask" — operator review.**

→ [resolved-graph representation minimization](docs/plans/representation-minimization.md) — operator-funded root-first leanness after the #5867 InternTable fix (whole-tree resolve 14.2 to 5.5 GiB): emitter-determinism gate, stream/evict (Lever C — converges with resolve-incrementality above), variant-Node minimal representation, func_env.sigs single-authority. Gated on the root-vs-doomed-seed discriminator (v1 is going away).

→ [§5 determinism mechanism](docs/plans/determinism-mechanism-design.md) — signature-derived DeterminismClass axis orthogonal to EffectShape; P1 primitive roster + compose algebra (#5941); bundles into #3468 InferredFacts at P4.

**What's on `.dag` today (the gap map — detail in the charter §2/§4):**

- [ ] **execution = a dependency-graph walk** — `claim_executor` interprets `ci_floor_plan.dag`; one fold, batches from dependency edges (the realest layer) — ⏳ awaiting sign-off
- [ ] **scheduling: width axis** (#5444) — `memory_aware_spawn_width` consumes the measured envelope; single-host — ⏳ awaiting sign-off
- [ ] **Section 1 spawn-width foundation — std.measure expressibility** (#5470/#5478) — the FLOOR family (measure_scale_fraction_floor + measure_fit_count_floor) and the demand-CEIL family completing the ceil-not-equal-floor money-pair landed, dissolving the measure unwrap→raw-arith→rewrap §3 fork. Authority: the expressibility-frontier spec #5467 [plan](docs/plans/expressibility-frontier.md) — ⏳ awaiting sign-off
- [ ] **scheduling: placement + materialization inert** — `Placement`/`Materialization` modeled + witness-passing, **no live consumer** (same band as the host-ops gap, substrate side)
- [ ] **scheduling: resource-aware (larger allocation → larger throughput)** — `spawn_width` is a side-channel derived from a static data row (`gunbc_ci_floor_measured_peak`, coarse whole_run÷concurrency estimate); goal: width derived from per-`Runnable` cost in the plan itself, self-calibrating from measured per-shard RSS. Four nodes: (A) measurement plumbing — claim_batch emits `[measurement] per-shard-peak-rss`, claim_executor collects max *(dispatched adhoc-97532cd3-dfe)*; (B) `CostEstimate.space` on `Runnable` in std; (C) scheduler reads cost → derives width, side-channel deleted; (D) calibration loop retires static data rows. Construction invariant: host memory increase → higher width → more throughput, no `.dag` edits required. [plan](docs/plans/resource-aware-scheduler.md)
- [ ] **caching forked** — sccache live · resolve-cache **opt-in (`GUNBC_RESOLVED_GRAPH_CACHE_DIR`; #5789 always-on default reverted — its whole-file JSON read/write OOMs the floor, must stream before it can default on)** · ParseTable memo live · RecordedFixture · BuildBuddy opt-in → converge on `realize(subject)` (§2 P2)

**Host-operation band — off-fabric, unmodeled, unenforced (G1–G3, NOW):**

→ [host-effect orchestration plan](docs/plans/host-effect-orchestration.md) — the unifying design: one `apply(target, effect, policy) → Receipt` interface (install · manage · decom on the same fleet authority), the shell→`.dag` migration phasing, and the ctrl-exodus / host-self-converge end-state. G1–G3 below are its consumers.

→ [srv3 virtual-media install (design for sign-off)](docs/plans/srv3-webui-kvm-virtual-media.md) — the first *install*-band consumer instance: a BMC `apply` transport `NbdProxyServe` (a direct wss+NBD seed client over OpenBMC's nbd-proxy, no browser/JS), capability-solver-dispatched (`os_install_mechanism`). srv3's cited row stays honestly grounded (PXE today) until an operator-gated ws-upgrade dry-run confirms the nbd-proxy surface; seed client gated on that.

- [ ] **G1 placement** — which host a job lands on is GitHub-native, demand-blind, first-idle → heavy runs co-reside, other host idles (the underutilization root) [plan](docs/plans/compute-envelope-model.md)
- [ ] **G2 runner deployment + cross-host placement** *(dispatched proud-tern-439: model + drift-gate)* — derive runners/host + registration from `fleet_intent`+envelope; **host-apply RESOLVED (operator-set, CI-humming plan authority): the gunbc-EMITTED converge shell** (`.github/fleet-converge.sh` = a §2 host-effect handler inhabiting the §6 carrier *by emission* — the emitted shell IS the executing consumer, escaping the §5 spec-without-execution trap; ctrl thin-runs it + parses a fail-closed receipt; §7 seed-shrink). In progress: emit (wise-eagle-664) + thin consume/run (fierce-carp)
- [ ] **G3 cgroup caps** — `TasksMax`/`MemoryMax` host-set by hand, only *read* live; derive + reconcile via the **emitted converge shell + fail-closed receipt** (same carrier-inhabitation-by-emission as G2) so a hand-edit reds
- [ ] **CI on compute fabric** — derive every host knob from one measured `ResourceEnvelope`; ends the crash-or-idle swing [plan](docs/plans/compute-envelope-model.md)

**Adjacent gaps (smaller, outside the host band):**

- [ ] **G4 dispatch dup** — `workflow_dispatch`+PR fire two same-SHA runs; `run_id` concurrency fallback won't collapse them → OOM [decision record](docs/plans/ci-merge-freshness.md)
- [ ] **CI inline-shell de-fork** — `RunStep.run` is raw concat'd bash across ~26 sites; #5427 modeled `cargo.Build.Nextest` but bypasses it for the release build (`ci_release_build_script` hand-writes bash) = a model↔realization fork in one file (+ hardcoded `CARGO_BUILD_JOBS`, pinned nextest version, a bash `uname` arch-case we already model as `TargetArchitecture`). Root: `RunStep` carries modeled effects, not a `String`; revive the inline-shell reducibility lens — §3 transport-fusion de-fork (the N×M adapter trap → one shape + N bound handlers). Sequence after #5427/#5546 (same file). [plan](docs/plans/emission-ingestion-inverse.md)
- [ ] **interpreter terminal-output de-fork** — the seed interpreter's CLI rendering (`v1_interpreter.rs` sgr/color_enabled/cli_verbosity/render_shell_trace, `claim_executor.rs` result lines) is a hand-Rust realization of three `.dag` authorities it does not yet consume: `extdeps.render.ansi.ansi_mappings` (SGR codes), `extdeps.render.terminal_capability.detect_capability` (NO_COLOR/CI/TTY to color), and `gunbc.output_policy` (resolve_verbosity precedence, channel_decision, env_var_for dispatch, shell_trace_summary_max_columns cap). §3 single-authority: the parallel representation can drift until the interpreter reads these tables. Dissolve-on: interpreter consumes the .dag render/output-policy tables at runtime (same class as CI inline-shell de-fork — seed forks the modeled authority; one home each). [plan](dsl/gunbc/output_policy.dag)
- [ ] **G5 rust-gate selection** — rust fmt/clippy/run-all is all-or-nothing on `.rs` PRs; no affected-set (the `.dag` floor already has one) [plan](docs/plans/ci-selection-vs-scheduling.md)
- [ ] **floor runs the right things** — SELECTION (what changed) vs SCHEDULING (by cost); cost never drives selection [plan](docs/plans/ci-selection-vs-scheduling.md)
  - [ ] opt-level=3 restores Pop-A to per-PR (#5456) — merged — ⏳ awaiting sign-off
  - [ ] per-PR = #5427 run-all sound baseline, shrunk to the affected set (#5427)
  - [ ] nightly = full-corpus selector-backstop + non-hermetic residue (#5447 stood down; ⚠ CI-gen load-bearing) *(quick-ant-298)*
- [ ] **tree-scoped builtin registry** (fail-closed) — global seed registry leaks intrinsics into the substrate compile; instance fix #5452, class fix (partition) open *(quick-ant-298)* [force-check plan](docs/plans/compile-clean-forcecheck.md)
- [ ] **kill sccache false-greens** — exit-0-no-binary; build-verify asserts artifact exists + fresh (partly landed in `ci.yml`)

**Shared abstractions (the lane's real deliverable — §6: pull in as CI flexes them, not by taxonomy):**

- [ ] **one Materialization kernel** — collapse sccache / resolve / ParseTable-memo / RecordedFixture / BuildBuddy onto `realize(subject)` (§2 P2)
- [ ] **one Placement authority** — jobs (GitHub) · threads (`spawn_width`) · sessions (ctrl `plans.capacity`) are 3 forks of "put work on a host"
- [ ] **resource budget tree** (§2 one-concept-every-scale, money→memory→infra) — grounded in real accounting (`extdeps.accounting.budget`, anchored, zero-based) generic over `Measure<Q,S>` (money = instantiation #1, memory = #2); a recursive tree where a child's appropriation is a line item charged at its parent (divide-once, structural). Three §5-distinct verdicts, never conflated: **admission** (`admit_all`, zero-based justified-&-approved) is the **construction** path — the committed set is provably within appropriation, over-commit unwritable on it; `node_conserves` is the honest **residue lens** for raw literals (the unstructurable residue, *not* a wall — the §5 distinction); runtime intent-vs-actual **reconcile** is the fail-closed **handler** (evict by QoS / loud-error on unsatisfiable Guaranteed). Subsumes spawn-width (#5444) · placement (#5559) · compile-jobs (#5546) as consumer leaves; a survey found `realization_width` + complexity `EffortBudget` are the SAME capped-resource→claims concept (§3 convergence candidates onto the one authority). **Protective only paired with an enforcement actuator** (admission cap or cgroup `memory.max`, operator-fenced): the model decides budgets, it does not itself prevent the kernel OOM, so interim the L1 claim-count must be the conservative-HIGH ceiling (carrier #5582). [carrier](dsl/product/budget_tree.dag) [grounding](docs/plans/budget-tree.md)
- [ ] **shared secrets/effects** — BMC · tokens · sccache-auth modeled once (when the fork is the pain)

**Downstream / parked:**

- [ ] privacy (compute fabric) — ⏳ awaiting sign-off
- [ ] repo model (internal repo) on compute fabric
- [ ] **gunbhub** — own the Git/CI engine (closes G6, the irreducible GitHub boundary); not pressing
- [ ] *(downstream)* compute fabric as a sellable infra piece

## 2. Minimal work — caching by realization (fail-closed)

Gate: uncached non-redundant work is an ERROR, not "slow". The cache-key-from-inputs construction is the §0 "cache trustworthy" item. → [plan](docs/plans/realization-measurement-loop.md)

**◆ Milestones:** key-by-construction ✓ (#5425) → **▸ warm==cold purity proven (616/616, #5429)** → resolve-cache ON — ~18% floor cut *(core ask)* → native `content(T)` *(gated B2)*

- [ ] F1 scheduler gives heavy nodes budgeted width (#5421) — ⏳ awaiting sign-off
- [ ] F2/F3 `resolved_graph` key derived from declared `inputs_considered` (#5425) — construction, not a lens — ⏳ awaiting sign-off
  - [ ] P1 honest keys by construction — warm==cold purity oracle **LANDED (#5429)** — ⏳ awaiting sign-off
    - [ ] P2 one door: `realize(subject)` sole API — kernel inhabits `cache_interface.dag` (#5446); ParseTable dissolution is downstream of the dsl→v2 de-fork (§5)
      - hermetic fixtures feed P2: [x] M4.1 universal hermetic corpus governance (#5236, [plan](docs/plans/m4-universal-hermetic-corpus.md)); [ ] M5 fixture-store onto one Realization kernel ([plan](docs/plans/m5-fixture-store-consolidation.md))
      - [ ] P3 **resolve-cache enable** — purity proven (616/616); cache is correct but **opt-in** (`GUNBC_RESOLVED_GRAPH_CACHE_DIR`): #5789's always-on `$TMPDIR` default was reverted because the whole-file JSON read (hit) and write (miss) buffer ~11x the packed graph in memory and OOM the concurrent-shard floor. Re-enabling by default is gated on a streaming IO realization (binary + `deserialize_from`/`serialize_into`, no whole-file `Vec<u8>`)
- [ ] P4 economic tier (measured cost → `Materialization`) — instrument done (#5431); remaining = (a) space arm: per-shard RSS plumbing → `CostEstimate.space` on `Runnable` (nodes A+B, [plan](docs/plans/resource-aware-scheduler.md)); (b) time arm: consumer feedback → `CostAccount.time = Measured` feeds width-fold (node C+D); dissolves `cost_account_predicted_zero()` on the floor path
- [ ] P5 native `content(T) = content_hash(subgraph)` — gated on B2

- blockers: [ ] B1 #5295 generic-instantiation (gates cross-shard `Share`) · [ ] B2 cross-tree content-hash (gates P5)

## 3. Complexity budget gate (stability — validation)

Operator decision (2026-06-21): budget-gate validation is the in-window tool; the algorithmic-cost *rewrite construction* is expansion, relocated to §5. Detection is total by construction (cost.dag U2); the gate's reach is a subject-production limit (fn-body reflection), not a detection one.

**◆ Milestones:** budget-gate non-toothless ✓ (#5437) → complexity gates the whole codebase *(gated on §5 fn-body reflection)*

- [ ] complexity lens total over the kernel (cost.dag U2); the gate runs a curated subject roster — ⏳ awaiting sign-off
- [ ] cost-lens zero-absorption fix — budgets non-toothless (#5437) — ⏳ awaiting sign-off
  - [ ] a subject-producer for every fn (#5437 helper; whole-corpus needs fn-body reflection)
    - [ ] complexity budget gates the whole codebase (gated on fn-body reflection)
- [ ] synthesis stays advisory (by Rice, optimality is a ratchet not a wall — DESIGN §5)

→ rewrite-catalog construction design preserved + relocated to §5 ([plan](docs/plans/algebraic-rewrite-optimization.md))

## 4. Testgen as the bug-class oracle (coverage by construction)

Prevent the next class, not the last instance: generate witnesses from declared structure. → [audit + method](docs/plans/testgen-oracle.md)

**◆ Milestones:** output gated ✓ (#5434) · coproduct-exhaustiveness structural ✓ (#5441) · cross-rep-equality ✓ (#5449) · oracle-method mapped ✓ (#5471) → **▸ NOW: wiring-liveness oracle (compile-time lens · fail compilation)** → anemia lens? *(parked, likely advisory)*

Motivating instance: a prod rotation 401'd `CREDENTIALS_MISSING` because `auth_input: access_token` was modeled (§3-correct) but the interpreter REST realization (`resolve_auth`) only realizes the env-var path — a declared input wired into nothing, typechecking but dead, and the only "auth tests" were `.contains("Bearer")` greps on emitted source (§5 spec-without-execution). The general class is **wiring-liveness**: a declared input must influence the output it feeds — the cache-purity oracle read backwards (purity: *same* in ⇒ *same* out; liveness: *different* in ⇒ *different* out — one perturbation kernel, two readings). → [audit + design](docs/plans/wiring-liveness-preflight.md)

- [ ] gate the generated output (#5434) — floor-discover `generated/` (or regen==committed drift gate) — ⏳ awaiting sign-off
- [ ] CoproductExhaustiveness made structural (#5441) — over every declared coproduct, not a hand-roster — ⏳ awaiting sign-off
- [ ] cross-representation-equality category (#5449) — straddle witness per coproduct × native realization — ⏳ awaiting sign-off
- [ ] **the oracle method (retro)** (#5471) — bug-class→mechanism map (generator/lens/wall); testgen owns A + B-routing only, rest are lenses/walls ([map](docs/plans/testgen-oracle.md) §2) — ⏳ awaiting sign-off
- [ ] affected-set = the completeness half (#5430) — model the full repo-process universe — frontier now has a paying consumer: the wiring residue-preflight (below) — ⏳ awaiting sign-off
  - [ ] *anemia lens?* (parked, DESIGN §2 leaf-side) — likely advisory, not a hard gate; decide whether to elevate
- [ ] **wiring-liveness oracle** — a declared input must influence its output; the cache-purity oracle read backwards (perturbation-response, one kernel two readings). Decidability split (§5): static dataflow reachability over the Node DAG is the **compile-time wall** (declared input with no path to an output = unwired → fail compilation, no runtime gate); perturbation/fuzz is the runtime **confirmation** for the opaque-seed residue (`resolve_auth` is Rust, invisible to `.dag` reflection today). [plan](docs/plans/wiring-liveness-preflight.md)
  - [ ] **pre-send fail-closed guard** (narrowest seam) — declared-auth-but-no-token ⇒ typed `AuthDeclaredButUnwired` refusal before send. **LANDED (#5683)** — fail-closed typed error before the request leaves `dispatch_rest`, never a remote 401; perturb-confirmed (drop the token → typed error) — ⏳ awaiting sign-off
  - [ ] **wiring lens — static reachability, compile-time** — over `.dag`-modeled input→output relations; fails compilation on a dead wire. **Wave-1 LANDED (#5679): reachability kernel + wired floor witness.** Primary mechanism; grows to subsume the seed realizations as they self-host (§5/§7)
  - [ ] **wiring witness — opaque-realization generator** — perturbation witness over declared service input→request relations; RED when an input is dropped (the execution-grounded generalization of the auth case, for realizations the compile-time lens can't see)
  - [ ] **runtime preflight — residue only** — run the wiring check over the `affected_set` `ReExecFrontier` (the about-to-run slice) before execution; `FailClosed` on a dead wire. Needed only for opaque realizations; dissolves into the compile-time lens as they self-host

## 5. Self-host v2 → delete `src/v1` (expansion)

Anchor (do not flip-flop): `.dag` = truth; purely self-hosting (v2 emits its own seed, no stage0 hand-edits); emit Rust + TypeScript; then shrink the seed to zero. → [plan](docs/plans/v2-self-hosting.md) · [de-fork audit](docs/plans/dsl-v2-defork-audit.md)

Adjacent lane — algorithmic-cost rewrite engine (the §3 construction design; post-stability, natural once `.dag` is the self-hosted truth). → [plan](docs/plans/algebraic-rewrite-optimization.md)

**◆ Milestones (critical path):** front-end ✓ · emit-rust well-typed ✓ · de-fork Step 1 ✓ (#5473) · class-3 corpus-coherence + cargo-green seed ✓ (#5481) → emitted crate cargo-builds green ✓ (#5777/#5873) → **▸ NOW: real fixed point** → `regen --verify` in CI ✓ (#5873) → seed-honesty → **TERMINAL: `src/v1` deleted** *(§7 regen-fixpoint deferred, #5514; src/v1 NOT yet deletable)*

- [ ] rewrite suboptimal patterns to the cheaper equivalent (`O(n²)→O(n)` …) — published finite catalog, bulletproof where it fires
- [ ] `Unknown` dissolved over time as an anemic atom (reuse the `Disposition` carrier)
- [ ] `O(n^x)→O(n log n)` substitution as per-idiom rules (candidate shared framing: **redundant intent** — the anti-unification generalization measures `spec − minimal`; see §0 self-applying lenses)
- [ ] front-end (parse / resolve / infer) over the whole tree — ⏳ awaiting sign-off
- [ ] emit whole tree `--target rust` (well-typed under CI gate) — ⏳ awaiting sign-off
- [ ] de-fork dsl ↔ v2 (one std authority) — **grounding cluster UNPARKED: operator ruled FreeMonoid/algebra single authority** (coproduct = structural authority; dsl record-surface derived-from-inhabitance; grounded-realization wins) → de-fork + self-host fused into one grounding lane ([brief](docs/plans/dsl-v2-defork-audit.md) §3b)
  - [ ] turn on cross-tree import — Step 1 **LANDED** (#5473) — ⏳ awaiting sign-off
  - [ ] **Root A — emit-seam grounding ✓ LANDED (#5734)**: String/`FreeMonoid`→host `Vec` via the #5428 `RustCorpusRepr` seam; drove HostNative cargo-green *(jolly-cat)* — ⏳ awaiting sign-off
  - [ ] **Root B — generic-inference keystone ✓ MERGED (#5552)** → def-unification (coproduct authority + aliases) → repoints (algebra/nat/integer/float/logic/effects/verification) → 🟡-marker dissolution *(bright-deer-111)*
  - [ ] v1-coupled `coercion`/`node` renames — deferred to v1-delete
- [ ] **emitted crate `cargo build`s green (Route-A last mile) ✓ LANDED (#5777/#5873)** — execution-grounded **re-census CONFIRMED 0 TOTAL errors** (cool-hawk-908: `regen_stage0 --emit-fresh` → `cargo build` debug+release, 2026-06-28). **REFUTED** the stale 'residual 2 = E0091×2' hypothesis: #5777 cleared E0599/E0624 **and** E0091 (`alias_unused_param_names` → `PhantomData` for unused type-alias params; `type_alias_phantom_param_test` witnesses). Trajectory: 647 E0308 → 0 (#5716/#5718/#5717) → ~32 other-family → 5 TOTAL → **0 TOTAL** (#5777). #5873: two-generation regen cutover + `regen --verify` CI gate. Compose emits `PhantomData<(Algebra, MachineConstraint)>`. Model hole stays flagged: pipe-first `OptionalOf` unwrap follow-up owed upstream (`Value::Null` runway). *(jolly-cat #5705/#5716/#5718 + cool-hawk-908 re-census)* — ⏳ awaiting sign-off
  - [ ] real fixed point: `content_hash` stage1==stage2 (dissolve placeholder hashes)
    - [ ] wire `regen_stage0 --verify` lockstep gate into CI — enforces no hand-edits to **generated** stage0 files (`GENERATED_STAGE0_FILES`); `HAND_MAINTAINED_STAGE0_FILES` are copied through, excluded by design ← **keystone** — ⏳ awaiting sign-off
      - [ ] dissolve seed hand-patches (`patch_*` / `HAND_MAINTAINED_STAGE0_FILES`) — the `emit_rust` hand-sync caveat the gate reproduces (the required-facts plan is dissolved into the executable `RegenVerifyGate`)
    - [ ] **TypeScript self-host (own lane)** — emit the compiler ITSELF as TypeScript and reproduce a per-realization merkle fixed point (the Rust `regen --verify` gate, mirrored for the TS realization). This is what makes "language design collapses to a row" real at full scale — §7 medium-agnostic proof, emit Rust *and* TypeScript, fixed point *per realization*. `5-ts-first-class` is the emit seed; this is the self-host. Gated on the Rust fixed point.
  - [ ] TypeScript to first-class (beyond the `add` slice) — the emit SEED. **In progress: Char atom-realization + green typed-fn emit (#5695), FieldAccess + RecordConstruct + `tsc --noEmit` oracle (#5701), operators by inhabitance (#5704)**
  - [ ] seed-honesty discharge (Diverse Double-Compiling)
  - [ ] collapse `src/v1` → pinned v2-emitted seed; delete the 154k hand-written lines (terminal, not a big-bang `rm`)

## 6. idea → idea compiler (expansion — stop anchoring on code)

A program is a canonical `Node` (the *idea*); ingest / emit / eval across many media via one grammar read both directions (§2 N+M). → [plan](docs/plans/idea-machine.md)

**◆ Milestones:** medium axis ✓ → language axis: English ingest round-trip → cross-media first-class (JSON/react/diagram) → invert hand-maintained: ROADMAP emitted + drift-gated

- [ ] **medium axis** — `Medium<R>` + `DecodeFidelity`; `LanguageModel` unified (13 forks dissolved); `compile(Eval) → EvalResult{value: Medium<Node>}` — ⏳ awaiting sign-off
- [ ] **round-trip law (ingest∘emit = id, DecodeFidelity-bounded)** (#5525/#5527) — established across two structurally-different media: markdown (block-document) and GHA-expr (recursive-expression). v2-TargetModel convergence is the deferred single-authority destination; per-medium round-trips are v1-seed interim. Authority for the law: the round-trip oracle #5513 §5.2 [plan](docs/plans/emission-ingestion-inverse.md) — ⏳ awaiting sign-off
- [ ] **ingestion as a first-class direction** — fold foreign surface INTO the node tree (the inverse of emit): `Lossless` where decidable, fail-closed `DecodeFidelity` where not, and **emit = ingest⁻¹ over ONE `GrammarRelation`** (§4: one grammar read both directions; the §7 "any language with a typed honesty boundary" payoff). Emission is well-covered (round-trip law, language axis); ingest *at large* — beyond the per-medium round-trips — has no lane. (the comment parser-wall #5579 is one corner of the construction direction; ingest-at-large needs an owner.) [plan](docs/plans/emission-ingestion-inverse.md)
- [ ] **language axis** — 15+ targets wave-1; English emit proven
  - [ ] English vocabulary closure → fail-closed English ingest (today's catch-all is fail-open; also §0)
  - [ ] English ingest round-trip (only emit proven today)
- [ ] **cross-media targets beyond syntax** — JSON / react / diagram as first-class media (not stringified)
  - [ ] `Medium<A> ↔ Medium<B>` homomorphisms
- [ ] **emission = ingestion⁻¹ extended past syntax** — diagnostic + orchestration intent emit per-target by rows [plan](docs/plans/emission-ingestion-inverse.md)
- [ ] `FidelityDisposition` compose-up → medium-level `DecodeFidelity`
- [ ] eval runtime generalization (wave-1 literal pins → `wave1_model_core` primitives)
- [ ] **invert hand-maintained artifacts** — emit each (ROADMAP flagship · doc indexes) from its `.dag` authority + drift-gate it (the ci.yml pattern) [plan](docs/plans/invert-hand-maintained.md)
  - [ ] **PR→checkbox status + section-emit + projection layer** (#5491/#5508/#5520) — box derived from an explicit *completes-iff-PR* binding (not mere mention), drift-gated; slice-1 status derivation + slice-2 section-emit + step-3a whole-document projection landed; #5513 medium-as-Node operative as the acceptance test [plan](docs/plans/invert-hand-maintained.md) — ⏳ awaiting sign-off

## 7. HTML / React rendering (expansion — the "website" sellable piece)

Depends on §6 — react/html is a first-class medium (idea-machine.md §3/§4), downstream of §6's cross-media item.

**◆ Milestones:** react/html page stands up (real page) → in the demo beside the TS emit

- [ ] react/html rendering stands up (real page, not fixture) — **SERVE page green-by-execution via gunbc-run landed (#5662); real socket the remaining gap (Lane C)**
- [ ] add to the demo alongside the TypeScript emit (website + language, dogfoodable)

## 8. Session dashboard on `.dag` (SHELVED)

Product/infra tooling — shelved during the stability window (no `.dag`-correctness leverage right now). **One slice un-shelved (operator-directed): the roadmap-as-spawner MVP below** — it earns `.dag`-correctness leverage by making the roadmap the single authority for tracked work (§3) and is the first inhabitant of migrating ctrl onto the substrate (§7).

- [ ] idea → PR pipeline *(deferred)*
- [ ] **roadmap-as-spawner MVP** *(active — operator-directed, un-shelves this slice)* — the gunbc roadmap `.dag` becomes the work-tracking DAG that drives ctrl session spawns. **Structure** (what work exists, deps, sizing, acceptance) lives only in the `.dag` (single authority §3, edited by committing to main); **runtime state** (done/live-session) flows ctrl→gunbc read-only as acceptance evidence — two kinds of fact, one home each, no dual authority. gunbc emits `roadmap-spawn-request/v1` via `roadmap_next_spawnable` (graph readiness = not-done ∧ deps-done); a thin ctrl bridge consumes it, dedups vs live sessions, and reuses the existing auto-spawn poller (zero new spawn code, #1812 merged). **Fail-closed pause kill switch** (§5, default paused — a missing/garbled control file spawns nothing). Stage 2 migrates spawn/session-management onto the `.dag` host-effect `apply()` seam (host-effect-orchestration Phase D/E), shrinking the ctrl realization toward zero (§7). [plan](docs/plans/roadmap-spawner.md)
