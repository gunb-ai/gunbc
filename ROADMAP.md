# gunbc — Roadmap

`DESIGN.md` is the authority for *why*. This is the **shape of the work** — a scannable, dependency-ordered checklist. **Checkboxes are authoritative for progress**; detail lives in the linked plan docs — don't restate it here (no dual representations). A task's real state is its branch/PR.

Legend: `[x]` done · `[ ]` todo · **indentation = depends on the item it sits under**. Each section opens with a **◆ Milestones** spine — the *verifiable* checkpoints in dependency order (`✓` reached · `▸` now · `○` ahead); the checklist below is the work toward them. Read L→R = the path.

**Priority order, top = now.** Bands: **stability / correctness** (§0–§4) → **expansion** (§5–§7) → **shelved** (§8).

## 0. Fail-closed lock-down LANE — BLOCKS expansion into products

Cache flakes, un-wired lenses, complexity violations = one problem: modeled, not made *impossible to write*. Fix = correctness by construction, not validation ([DESIGN §5](DESIGN.md)).

→ [audit + checklist](docs/plans/fail-closed-lockdown.md)

This window = a few days of STABILITY — shrink the fail-open surface, don't "lock" it. The deepest root (`Value::Null` overload, ~131 sites) stays OPEN until its own runway ([fork plan](docs/plans/model-realization-fork.md) §3).

**◆ Milestones:** fail-open audited ✓ · numeric tower grounded ✓ (#5428) → **▸ NOW: cache warm==cold oracle · every lens wired-or-deleted · stage0 census under budget** → `Value::Null` split *(deep root)*

**Audits (done):**

- [x] lens/gate wiring — most analytical lenses inert (authored, no discovered gate)
- [x] fail-open code — cache lossy-digest · under-keyed memos · `unwrap_or_default` infer
- [x] **model↔realization fork — ROOT CONFIRMED** — one seam (~13 bridges); sub-roots = numeric tower + `Value::Null` [plan](docs/plans/model-realization-fork.md)
- [ ] remaining: coercion/equality straddles · inference fail-open · cache-purity · CI-coverage-completeness [detail](docs/plans/fail-closed-lockdown.md)

**In-scope this window:**

- [x] **numeric-tower grounding** (#5428) — `Int=GroupCompletion<Nat>`; `==` straddle guard now dead-in-corpus [plan](docs/plans/model-realization-fork.md)
- [ ] **cache trustworthy** — authoritative home is §2 F2/F3/P1; ship the warm==cold oracle as a detective now
- [ ] **rust-gate coverage** (shared §1) — opt-level=3 restores Pop-A to per-PR (#5456); run-all-unless-`#[ignore]`d (#5427); `.dag`→rust coverage wall = edge-(b), SCOPED / pending operator greenlight *(quick-ant-298)* [cause table](docs/plans/ci-selection-vs-scheduling.md) [edge-(b) brief](docs/plans/edge-b-rust-dag-provenance-brief.md)
- [ ] **promote-or-delete inert lenses · de-vacuum gates** — EmitHostGate de-vacuumed ✓ (#5477); 4 advisory lenses widened+bounded, whole-corpus deferred to `.dag` structural-reflection (also unlocks coverage/testgen) *(silent-wren-739)*
- [x] **realization-vocabulary containment guard** (#5445/#5453) — target-AST importable only at the realization edge (fail-closed, shrinking-roster); dissolve-on: bash-sidecar arc empties the roster → pure wall → `program.dag` deletable [plan](docs/plans/emission-ingestion-inverse.md)
- [ ] **stage0 clone-census inert + seed regressed** to 21540 (~1138 over) — resolve by clone-reduction / substrate-migration, NEVER a cap-bump; #5427 `#[ignore]` is the interim *(fierce-hawk-540 via quick-ant-298)*

**Fenced OUT (after stability):**

- [ ] **split `Value::Null`** (None/Absent/miss/Violates → own carriers) — ~131-site substrate change, the deeper root; own runway
- [ ] **self-host purity gate** — a §5 deliverable, not §0 (avoids the §0↔§5 cycle)
- [x] **cross-tree import activation** (§5) (#5473) — LANDED; the §0↔§5 escalate item is now closed
- [ ] **`Disposition` carrier** — a new concept; parked [plan](docs/plans/disposition-carrier.md)
- [ ] complexity-budget whole-codebase (§3) · cache-redundancy completeness (§2 P3) — residue, after construction
- [ ] **cardinality refinement** — illegal cardinalities (wrong length · empty · overflow) unwritable by construction; the *decidable* refinement axis (linear arithmetic over counts), fold-propagated. MVP-1 (`Byte` via `Length<8>`) + P4 (fold homomorphism · uint8 overflow → typed `Rejected`) proven (#5512); P1 (`where` lowering) / P2 (construction-enforced) behind this lane; P5 (phantom-width reflection) substrate-blocked. [plan](docs/plans/cardinality-refinement.md) [P1](docs/plans/p1-where-clause-lowering.md)

**Meta — lock down the reasoning (§7 recursion):**

- [x] **inert-lens hygiene backstop** (#5433) — every `lens/*.dag` wired or deleted; runs over the corpus
- [ ] **reachability-completeness lens** — every declared node (code carrier · doc · lens) reachable from a run-root, rostered, or deleted; generalizes #5433 to carriers + docs [plan](docs/plans/inert-layer-lens.md)
- [ ] **gate-hygiene: a floor-enrolled gate must be green-on-main at merge** — roster-completeness assertion promoted to should-land ([plan](docs/plans/emission-ingestion-inverse.md) §2; [merge-freshness decision record](docs/plans/ci-merge-freshness.md)) *(quick-ant-298)*
- [x] **construction-justification rule** (#5476) (authoring-time) — justify why a class can't be construction before adding a lens *(silent-wren-739)* [plan](docs/plans/construction-justification-rule.md) [DESIGN §6](DESIGN.md)
- [ ] **expressibility frontier** — partition each modeling discipline into wall / lens-residue / undecidable-review *before* gating [plan](docs/plans/expressibility-frontier.md)
- [ ] **confront the skipped modeling decisions** — the `🟡` comment backlog [Disposition plan](docs/plans/disposition-carrier.md)
- [ ] **axiom + syllogism lens** (DESIGN open thread #1) — every claim chains back to an axiom, no orphan/cycle; stays `[ ]` until it runs executably over this doc [scope](docs/plans/axiom-syllogism-lens.md)

## 1. CI as the substrate integration dogfood (the correctness floor)

A flaky or green-but-broken floor means no gate protects anything — so CI is upstream of every §0 claim. CI is also the one workload that flexes *every* substrate layer at once (execution · scheduling · caching · secrets/effects · emission), so it is the forcing function that turns each modeled-but-inert abstraction load-bearing. **Deliverable = shared abstractions proven by CI consuming them** (one Materialization kernel · one Placement authority · one secrets model); faster CI falls *out* of that, it is not the goal (§6 — price the lane in displaced cost, "move with confidence", not elegance).

→ [charter: causal-chain gap analysis](docs/plans/ci-process-end-to-end.md) — what's on `.dag` today vs not, push→execute.

**◆ Milestones:** execution-as-DAG ✓ (the floor *is* a bounded forward graph walk) · width on `.dag` ✓ (#5444) → **▸ NOW — host-operation on `.dag` (placement · runner deployment · caps are hand-managed, off-fabric)** → resolve-cache enabled → one Materialization kernel (collapse the 5 caches) → one Placement authority (jobs · threads · sessions = 3 forks) → shared secrets · gunbhub closes the GitHub engine (G6, parked)

**What's on `.dag` today (the gap map — detail in the charter §2/§4):**

- [x] **execution = a dependency-graph walk** — `claim_executor` interprets `ci_floor_plan.dag`; one fold, batches from dependency edges (the realest layer)
- [x] **scheduling: width axis** (#5444) — `memory_aware_spawn_width` consumes the measured envelope; single-host
- [x] **Section 1 spawn-width foundation — std.measure expressibility** (#5470/#5478) — the FLOOR family (measure_scale_fraction_floor + measure_fit_count_floor) and the demand-CEIL family completing the ceil-not-equal-floor money-pair landed, dissolving the measure unwrap→raw-arith→rewrap §3 fork. Authority: the expressibility-frontier spec #5467 [plan](docs/plans/expressibility-frontier.md)
- [ ] **scheduling: placement + materialization inert** — `Placement`/`Materialization` modeled + witness-passing, **no live consumer** (same band as the host-ops gap, substrate side)
- [ ] **caching forked** — sccache live · resolve-cache **dormant** (pure-proven 616/616, env var unset — biggest dormant lever) · ParseTable memo live · RecordedFixture · BuildBuddy opt-in → converge on `realize(subject)` (§2 P2)

**Host-operation band — off-fabric, unmodeled, unenforced (G1–G3, NOW):**

- [ ] **G1 placement** — which host a job lands on is GitHub-native, demand-blind, first-idle → heavy runs co-reside, other host idles (the underutilization root) [plan](docs/plans/compute-envelope-model.md)
- [ ] **G2 runner deployment** — runners/host + registration are hand-run shell, **no repo artifact**; derive from `operator_fleet`+envelope, generate + drift-gate (the `ci.yml` pattern, for the host)
- [ ] **G3 cgroup caps** — `TasksMax`/`MemoryMax` host-set by hand, only *read* live; derive + reconcile-gate so a hand-edit reds
- [ ] **CI on compute fabric** — derive every host knob from one measured `ResourceEnvelope`; ends the crash-or-idle swing [plan](docs/plans/compute-envelope-model.md)

**Adjacent gaps (smaller, outside the host band):**

- [ ] **G4 dispatch dup** — `workflow_dispatch`+PR fire two same-SHA runs; `run_id` concurrency fallback won't collapse them → OOM [decision record](docs/plans/ci-merge-freshness.md)
- [ ] **G5 rust-gate selection** — rust fmt/clippy/run-all is all-or-nothing on `.rs` PRs; no affected-set (the `.dag` floor already has one) [plan](docs/plans/ci-selection-vs-scheduling.md)
- [ ] **floor runs the right things** — SELECTION (what changed) vs SCHEDULING (by cost); cost never drives selection [plan](docs/plans/ci-selection-vs-scheduling.md)
  - [x] opt-level=3 restores Pop-A to per-PR (#5456) — merged
  - [ ] per-PR = #5427 run-all sound baseline, shrunk to the affected set (#5427)
  - [ ] nightly = full-corpus selector-backstop + non-hermetic residue (#5447 stood down; ⚠ CI-gen load-bearing) *(quick-ant-298)*
- [ ] **tree-scoped builtin registry** (fail-closed) — global seed registry leaks intrinsics into the substrate compile; instance fix #5452, class fix (partition) open *(quick-ant-298)* [force-check plan](docs/plans/compile-clean-forcecheck.md)
- [ ] **kill sccache false-greens** — exit-0-no-binary; build-verify asserts artifact exists + fresh (partly landed in `ci.yml`)

**Shared abstractions (the lane's real deliverable — §6: pull in as CI flexes them, not by taxonomy):**

- [ ] **one Materialization kernel** — collapse sccache / resolve / ParseTable-memo / RecordedFixture / BuildBuddy onto `realize(subject)` (§2 P2)
- [ ] **one Placement authority** — jobs (GitHub) · threads (`spawn_width`) · sessions (ctrl `plans.capacity`) are 3 forks of "put work on a host"
- [ ] **shared secrets/effects** — BMC · tokens · sccache-auth modeled once (when the fork is the pain)

**Downstream / parked:**

- [x] privacy (compute fabric)
- [ ] repo model (internal repo) on compute fabric
- [ ] **gunbhub** — own the Git/CI engine (closes G6, the irreducible GitHub boundary); not pressing
- [ ] *(downstream)* compute fabric as a sellable infra piece

## 2. Minimal work — caching by realization (fail-closed)

Gate: uncached non-redundant work is an ERROR, not "slow". The cache-key-from-inputs construction is the §0 "cache trustworthy" item. → [plan](docs/plans/realization-measurement-loop.md)

**◆ Milestones:** key-by-construction ✓ (#5425) → **▸ warm==cold purity proven (616/616, #5429)** → resolve-cache ON — ~18% floor cut *(core ask)* → native `content(T)` *(gated B2)*

- [x] F1 scheduler gives heavy nodes budgeted width (#5421)
- [x] F2/F3 `resolved_graph` key derived from declared `inputs_considered` (#5425) — construction, not a lens
  - [ ] P1 honest keys by construction — warm==cold purity oracle (#5429)
    - [ ] P2 one door: `realize(subject)` sole API — kernel inhabits `cache_interface.dag` (#5446); ParseTable dissolution is downstream of the dsl→v2 de-fork (§5)
      - hermetic fixtures feed P2: [x] M4.1 universal hermetic corpus governance (#5236, [plan](docs/plans/m4-universal-hermetic-corpus.md)); [ ] M5 fixture-store onto one Realization kernel ([plan](docs/plans/m5-fixture-store-consolidation.md))
      - [ ] P3 **resolve-cache enable** — cuts ~18% of floor wall; purity proven (616/616); gated on #5429 ← **core ask**
- [ ] P4 economic tier (measured cost → `Materialization`) — instrument done (#5431); remaining = the consumer feedback + width-fold
- [ ] P5 native `content(T) = content_hash(subgraph)` — gated on B2

- blockers: [ ] B1 #5295 generic-instantiation (gates cross-shard `Share`) · [ ] B2 cross-tree content-hash (gates P5)

## 3. Complexity budget gate (stability — validation)

Operator decision (2026-06-21): budget-gate validation is the in-window tool; the algorithmic-cost *rewrite construction* is expansion, relocated to §5. Detection is total by construction (cost.dag U2); the gate's reach is a subject-production limit (fn-body reflection), not a detection one.

**◆ Milestones:** budget-gate non-toothless ✓ (#5437) → complexity gates the whole codebase *(gated on §5 fn-body reflection)*

- [x] complexity lens total over the kernel (cost.dag U2); the gate runs a curated subject roster
- [x] cost-lens zero-absorption fix — budgets non-toothless (#5437)
  - [ ] a subject-producer for every fn (#5437 helper; whole-corpus needs fn-body reflection)
    - [ ] complexity budget gates the whole codebase (gated on fn-body reflection)
- [ ] synthesis stays advisory (by Rice, optimality is a ratchet not a wall — DESIGN §5)

→ rewrite-catalog construction design preserved + relocated to §5 ([plan](docs/plans/algebraic-rewrite-optimization.md))

## 4. Testgen as the bug-class oracle (coverage by construction)

Prevent the next class, not the last instance: generate witnesses from declared structure. → [audit + method](docs/plans/testgen-oracle.md)

**◆ Milestones:** output gated ✓ (#5434) · coproduct-exhaustiveness structural ✓ (#5441) · cross-rep-equality ✓ (#5449) · oracle-method mapped ✓ (#5471) → anemia lens? *(parked, likely advisory)*

- [x] gate the generated output (#5434) — floor-discover `generated/` (or regen==committed drift gate)
- [x] CoproductExhaustiveness made structural (#5441) — over every declared coproduct, not a hand-roster
- [x] cross-representation-equality category (#5449) — straddle witness per coproduct × native realization
- [x] **the oracle method (retro)** (#5471) — bug-class→mechanism map (generator/lens/wall); testgen owns A + B-routing only, rest are lenses/walls ([map](docs/plans/testgen-oracle.md) §2)
- [x] affected-set = the completeness half (#5430) — model the full repo-process universe
  - [ ] *anemia lens?* (parked, DESIGN §2 leaf-side) — likely advisory, not a hard gate; decide whether to elevate

## 5. Self-host v2 → delete `src/v1` (expansion)

Anchor (do not flip-flop): `.dag` = truth; purely self-hosting (v2 emits its own seed, no stage0 hand-edits); emit Rust + TypeScript; then shrink the seed to zero. → [plan](docs/plans/v2-self-hosting.md) · [de-fork audit](docs/plans/dsl-v2-defork-audit.md)

Adjacent lane — algorithmic-cost rewrite engine (the §3 construction design; post-stability, natural once `.dag` is the self-hosted truth). → [plan](docs/plans/algebraic-rewrite-optimization.md)

**◆ Milestones (critical path):** front-end ✓ · emit-rust well-typed ✓ · de-fork Step 1 ✓ (#5473) · class-3 corpus-coherence + cargo-green seed ✓ (#5481) → **▸ NOW: emitted crate cargo-builds green** → std forks collapsed → real fixed point → **KEYSTONE: `regen --verify` in CI** → seed-honesty → **TERMINAL: `src/v1` deleted** *(§7 regen-fixpoint deferred, #5514; src/v1 NOT yet deletable)*

- [ ] rewrite suboptimal patterns to the cheaper equivalent (`O(n²)→O(n)` …) — published finite catalog, bulletproof where it fires
- [ ] `Unknown` dissolved over time as an anemic atom (reuse the `Disposition` carrier)
- [ ] `O(n^x)→O(n log n)` substitution as per-idiom rules (open: cleaner shared framing?)
- [x] front-end (parse / resolve / infer) over the whole tree
- [x] emit whole tree `--target rust` (well-typed under CI gate)
- [ ] de-fork dsl ↔ v2 (one std authority, no historical forks)
  - [x] turn on cross-tree import — Step 1 **LANDED** (#5473) — PR-B (collapse forks) next *(nimble-koi-625)*
    - [ ] collapse clear duplicates (algebra, logic, nat, reducible, measure)
    - [ ] resolve same-name/different-job pairs (integer, effects, float, coercion, node, verification)
- [ ] emitted crate `cargo build`s green (Route-A last mile)
  - [ ] real fixed point: `content_hash` stage1==stage2 (dissolve placeholder hashes)
    - [ ] wire `regen_stage0 --verify` lockstep gate into CI — enforces no stage0 hand-edits ← **keystone**
      - [ ] dissolve seed hand-patches (`patch_*` / `HAND_MAINTAINED_STAGE0_FILES`)
  - [ ] TypeScript to first-class (beyond the `add` slice)
  - [ ] seed-honesty discharge (Diverse Double-Compiling)
  - [ ] collapse `src/v1` → pinned v2-emitted seed; delete the 154k hand-written lines (terminal, not a big-bang `rm`)

## 6. idea → idea compiler (expansion — stop anchoring on code)

A program is a canonical `Node` (the *idea*); ingest / emit / eval across many media via one grammar read both directions (§2 N+M). → [plan](docs/plans/idea-machine.md)

**◆ Milestones:** medium axis ✓ → language axis: English ingest round-trip → cross-media first-class (JSON/react/diagram) → invert hand-maintained: ROADMAP emitted + drift-gated

- [x] **medium axis** — `Medium<R>` + `DecodeFidelity`; `LanguageModel` unified (13 forks dissolved); `compile(Eval) → EvalResult{value: Medium<Node>}`
- [x] **round-trip law (ingest∘emit = id, DecodeFidelity-bounded)** (#5525/#5527) — established across two structurally-different media: markdown (block-document) and GHA-expr (recursive-expression). v2-TargetModel convergence is the deferred single-authority destination; per-medium round-trips are v1-seed interim. Authority for the law: the round-trip oracle #5513 §5.2 [plan](docs/plans/emission-ingestion-inverse.md)
- [ ] **language axis** — 15+ targets wave-1; English emit proven
  - [ ] English vocabulary closure → fail-closed English ingest (today's catch-all is fail-open; also §0)
  - [ ] English ingest round-trip (only emit proven today)
- [ ] **cross-media targets beyond syntax** — JSON / react / diagram as first-class media (not stringified)
  - [ ] `Medium<A> ↔ Medium<B>` homomorphisms
- [ ] **emission = ingestion⁻¹ extended past syntax** — diagnostic + orchestration intent emit per-target by rows [plan](docs/plans/emission-ingestion-inverse.md)
- [ ] `FidelityDisposition` compose-up → medium-level `DecodeFidelity`
- [ ] eval runtime generalization (wave-1 literal pins → `wave1_model_core` primitives)
- [ ] **invert hand-maintained artifacts** — emit each (ROADMAP flagship · doc indexes) from its `.dag` authority + drift-gate it (the ci.yml pattern) [plan](docs/plans/invert-hand-maintained.md)
  - [x] **PR→checkbox status + section-emit + projection layer** (#5491/#5508/#5520) — box derived from an explicit *completes-iff-PR* binding (not mere mention), drift-gated; slice-1 status derivation + slice-2 section-emit + step-3a whole-document projection landed; #5513 medium-as-Node operative as the acceptance test [plan](docs/plans/invert-hand-maintained.md)

## 7. HTML / React rendering (expansion — the "website" sellable piece)

Depends on §6 — react/html is a first-class medium (idea-machine.md §3/§4), downstream of §6's cross-media item.

**◆ Milestones:** react/html page stands up (real page) → in the demo beside the TS emit

- [ ] react/html rendering stands up (real page, not fixture)
- [ ] add to the demo alongside the TypeScript emit (website + language, dogfoodable)

## 8. Session dashboard on `.dag` (SHELVED)

Product/infra tooling — shelved during the stability window (no `.dag`-correctness leverage right now).

- [ ] idea → PR pipeline *(deferred)*
