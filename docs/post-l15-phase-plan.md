> Part of: [THESIS.md](../THESIS.md) > [src/v3/ROADMAP.md](../src/v3/ROADMAP.md)

# Post-A/B Lane Plan — Working backward from the thesis

**Status:** Active plan. Half A merged; Half B pending. Lane 1 can start today.
**Total time:** ~14 weeks, four major lanes (substantial parallelism; see Gantt below).
**Discipline:** Every open thesis obligation is placed in a lane. **Nothing is backlog.**

---

## Thesis → lane derivation

Three load-bearing claims in THESIS.md. Working backward, each claim has a gap between what's declared and what the compiler actually enforces. One lane closes each gap.

| Thesis claim | Current gap | Lane that closes it |
|---|---|---|
| *"Emission is mechanical translation. Adding a new target = one spec file."* | Three hand-written per-target Rust emitters (emit_rust.rs 3600+ lines, emit_go.rs, emit_python.rs). Substrate exposes `Dag.ports`/`Dag.nodes` as linear lists, forcing every lens to reinvent `find_port`/`find_behavior`. Targets beyond Rust/Go/Python (Verilog, SPICE, English) named in architecture.md but zero code. | **Lane 1: Emission unification** |
| *"Correctness is many orthogonal dimensions… inescapable like conservation laws."* | Termination ✓ and structural cost ✓ proven at compile time. Idempotency **declared** in `dsl/std/effects.dag` with 16 v2 tests but **compiler consumption not wired** (THESIS.md:1291). Symbolic bounds, parallelism-as-diagnostic both declared as "NOT YET IMPLEMENTED". | **Lane 2: Compile-time proofs** |
| *"Causal engine. The compiler describes itself in .dag and is its own first consumer."* | Compiler is ~97% hand-written Rust sketch. `compiler.dag` exists (PR #418) but emit → compile → emit fixed-point not wired. Diagnostics explain in compiler-internal vocabulary, not user-pasteable corrections. Mutual recursion, `data` semantics, `where` refinement still block self-describing compiler. | **Lane 3: Self-hosting cycle** |
| *(tail obligations not cleanly derived from a single claim)* | Transport declarations, `dag run` interpreter, side effects as a compile-time dimension, space bounds as a compile-time dimension, async emission modeling. None fit Lanes 1–3's themes but all are thesis obligations. | **Lane 4: Completion layer** |

The four lanes exhaust the thesis. When all complete, gunbc cashes out its claim: one source, provably correct across every declared dimension, self-describing, executable through declared transports. There is no post-plan backlog.

---

## Lane summaries

### Lane 1 — Emission unification (~5 weeks)

**Closes:** "adding a new target = one spec file, zero new Rust"

Six internal stages. Each builds on the previous:

| Stage | Time | Scope | Design doc |
|---|---|---|---|
| 1a | 1 week | L1.5 tail: Consumed rendering, Go unignores, receipts audit, m1_3 perf | [phase1-lane1-l15-tail.md](./phase1-lane1-l15-tail.md) |
| 1b | 1 week | **Substrate keyed-lookup** (meta-review root cause): `port_by_id`, `node_by_id`, `resolve_producer` Bind-pass-through; migrate existing 3 lenses; INVARIANTS.md L-7 ("lenses don't reconstruct lookup locally") | [lane1-stage-b-substrate-keyed-lookup.md](./lane1-stage-b-substrate-keyed-lookup.md) |
| 1c | 1 week | Clean-emission invariant E-5: `CleanEmissionContract` per-target spec, pilot with unused pattern bindings | [phase1-lane2-clean-emission-invariant.md](./phase1-lane2-clean-emission-invariant.md) |
| 1d | 1 week | Consolidation build plan: function inventory, spec gaps, bridge inventory, pilot target choice | [phase1-lane3-consolidation-build-plan.md](./phase1-lane3-consolidation-build-plan.md) |
| 1e | 2 weeks | Consolidation execution: dissolve emit_rust.rs/go.rs/python.rs into one generic walker + per-target specs | (written at start of 1d, after build-plan locks design) |
| 1f | 1 week | New targets as smoking gun: add Verilog + SPICE + English — each is ONE spec file, zero new Rust | (written at end of 1e, sized by what consolidation actually shipped) |

**Acceptance:** `grep -r "fn render_" src/v3/compiler/src/` returns zero target-specific matches; every current target + three new ones (Verilog, SPICE, English) roundtrip through a single generic walker; zero `#[allow(warnings)]` attributes anywhere; adding a fourth new target requires no Rust changes.

### Lane 2 — Compile-time proofs (~4 weeks, overlaps Lane 1 from week 3)

**Closes:** "every structural property gunbc claims is compile-time-enforced, not a runtime flag"

Six stages covering the three undeclared-compiler-consumption properties:

| Stage | Time | Scope |
|---|---|---|
| 2a | 0.5 week | Port `dsl/std/effects.dag` → `src/v3/std/effects.dag`. Minimum carry-over; no new design. |
| 2b | 1.5 weeks | Workflow idempotency lens: walks a pipeline, composes `EffectShape` per op via `compose_effects`, emits diagnostic pointing at the non-lattice op when chain breaks |
| 2c | 1 week | Test obligation materialization: `generate_idempotency_obligations` spec → actually-emitted `f(f(x)) == f(x)` tests. End-to-end fixture: GCP bringup workflow compiler-proved idempotent, runnable test asserts it against mock API |
| 2d | 1 week | L2 M1 symbolic cost bounds. Unignores `kf_1_lambda_body_cost_contributes_to_fold`. Structural cost composes through fold/map/loop with symbolic arity. O(n) vs O(n²) diagnostic |
| 2e | 0.5 week | Parallelism-as-lens. Unignores `parallel_fold_on_commutative_monoid_is_reducible`. "Promotable to map" diagnostic becomes a lens output, not just a structural test |
| 2f | 0.5 week | User-declared dimensions: infrastructure from 2b–2e generalizes so users can add custom compile-time proofs via `.dag` declaration (M4 thesis completion) |

**Acceptance:** the four previously `#[ignore]`d property tests are all green. A fixture that declares a non-idempotent cloud workflow (e.g., `POST /logs` in a retry loop) fails to compile with a diagnostic naming the breaking op. Symbolic cost reports O(n²) on nested fold. All v2 idempotency tests (`src/v2/tests/src/effects.rs`) have v3 equivalents passing.

Full design doc: [lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md)

### Lane 3 — Self-hosting cycle (~4 weeks, starts after Lane 1e)

**Closes:** "the compiler is describable in .dag and is its own first consumer"

Three stages. This is M2 feature parity + diagnostics-as-corrections + the self-hosting cycle proper:

| Stage | Time | Scope |
|---|---|---|
| 3a | 1.5 weeks | M2 feature parity blockers for `compiler.dag`: mutual recursion → Loop (SELF_HOSTING §2.4), `data` value semantics, `where` refinement predicates, full surface generics |
| 3b | 1 week | Diagnostics-as-corrections: every diagnostic carries `fix: List<Correction>` with literal code. Per-target fix syntax declared in spec (same `CleanEmissionContract` surface as Lane 1c — Rust fix syntax, Python fix syntax, etc.) |
| 3c | 1.5 weeks | Self-hosting cycle: `compiler.dag` → Lane 1e emitter → Rust → `rustc` → v3_compiler binary. Fixed-point ratchet: re-emit is bit-identical. `cargo run --bin self-host-fixed-point` is a CI gate |

**Acceptance:** running the compiler binary on `compiler.dag` produces Rust identical to the previous run. Every diagnostic in `thesis_validation_test.rs`'s T-series emits a literal fix snippet. The compiler has dogfooded itself end-to-end.

Full design doc: [lane3-self-hosting-cycle.md](./lane3-self-hosting-cycle.md)

---

## Lane 4 — Completion layer (~4 weeks)

**Closes:** everything else — transport declarations, `dag run` interpreter, side effects as a compile-time dimension, space bounds, async emission modeling.

Four stages:

| Stage | Time | Scope |
|---|---|---|
| 4a | 1.5 weeks | Transport declarations (typed `RestTransport`/`ShellTransport`/etc., not string-tagged) + `dag run` interpreter (execute .dag without emission) |
| 4b | 1 week | Side effects as Dimension instance (extends Lane 2 2f framework). Workflow lens rejects hermetic/non-hermetic mixing |
| 4c | 1 week | Space bounds as Dimension instance. Structural space cost composes through list operations. `where memory_bounded(…)` declarations enforce |
| 4d | 0.5 week | Async emission modeling. Target spec declares async strategy; walker emits `async fn` / `.await` based on spec — zero new walker code |

**Acceptance:** `dag run` executes a cloud workflow through declared transports; side-effects and space-bounds dimensions have live workflow lenses; an async Rust variant of any program emits through the same walker as sync Rust.

Full design doc: [lane4-completion.md](./lane4-completion.md)

---

## Sequencing (Gantt)

```
Week:      1    2    3    4    5    6    7    8    9   10   11   12   13   14
Lane 1:  [1a] [1b] [1c] [1d] [       1e (2w)      ] [1f]
Lane 2:             [2a] [     2b    ] [2c] [2d] [2e] [2f]
Lane 3:  [    3a     ] [3b] [               wait             ] [      3c     ]
Lane 4:                                                         [   4a   ] [4b] [4c] [4d]
```

Week-by-week view:

| Wk | Lane 1 | Lane 2 | Lane 3 | Lane 4 |
|----|--------|--------|--------|--------|
| 1  | 1a (L1.5 tail) | — | 3a (M2 parity) | — |
| 2  | 1b (keyed-lookup) | — | 3a | — |
| 3  | 1c (E-5 invariant) | 2a (effects port) | 3b (corrections) | — |
| 4  | 1d (build plan) | 2b (idempotency lens) | (3c blocked on 1e) | — |
| 5  | 1e (consolidation) | 2b | — | — |
| 6  | 1e | 2c (test materialize) | — | — |
| 7  | 1e | 2d (symbolic bounds) | — | — |
| 8  | 1f (Verilog/SPICE/English) | 2e (parallelism lens) | — | — |
| 9  | — | 2f (user dims) | 3c start | — |
| 10 | — | — | 3c | 4a (transports + dag run) |
| 11 | — | — | 3c | 4a |
| 12 | — | — | — | 4b (side effects) |
| 13 | — | — | — | 4c (space bounds) |
| 14 | — | — | — | 4d (async emit) |

### Hard dependencies

```
1a → 1b, 1c, 1d, 3a (clean repo state)
1b → 2a, 2b, 2c, 2d, 2e (Lane 2 reads keyed accessors)
1c → 3b (corrections share CleanEmissionContract surface)
1d → 1e (build plan gates execution)
1e → 1f, 3c, 4d (consolidated walker needed downstream)
3a → 3b, 3c, 4a (M2 parity blocks diagnostic-as-correction, self-hosting, transport/dag run)
2f → 4b, 4c (Dimension framework extends for side effects, space bounds)
2b–2e → 2f (user-dim abstraction generalizes over the concrete lenses)
```

### Design blockers (all resolved — design docs ready for implementer review)

Nine cross-cutting design decisions needed to be locked before implementation. Each has a dedicated design doc with scope, rationale, rejected alternatives, implementation notes, acceptance gates, and open questions.

| DB | Resolves | Owning lane/stage | Design doc |
|---|---|---|---|
| DB-1 | `Diagnostic.fix: List<Correction>` shape | Lane 3 Stage 3b (implements); Lane 2 all (consumes) | [design-correction-shape.md](./design-correction-shape.md) |
| DB-2 | Generic walker API | Lane 1 Stage 1e | [design-generic-walker-api.md](./design-generic-walker-api.md) |
| DB-3 | `Dimension` abstraction for compile-time proofs | Lane 2 Stage 2f | [design-dimension-abstraction.md](./design-dimension-abstraction.md) |
| DB-4 | `CleanEmissionContract` concrete fields | Lane 1 Stage 1c | [design-clean-emission-contract.md](./design-clean-emission-contract.md) |
| DB-5 | Substrate keyed-lookup API | Lane 1 Stage 1b | [design-substrate-keyed-lookup-api.md](./design-substrate-keyed-lookup-api.md) |
| DB-6 | Transport type taxonomy | Lane 4 Stage 4a | [design-transport-taxonomy.md](./design-transport-taxonomy.md) |
| DB-7 | Symbolic cost algebra | Lane 2 Stage 2d | [design-symbolic-cost-algebra.md](./design-symbolic-cost-algebra.md) |
| DB-8 | Fixed-point ratchet mechanics | Lane 3 Stage 3c | [design-fixed-point-ratchet.md](./design-fixed-point-ratchet.md) |
| DB-9 | Mutual recursion → Loop lowering | Lane 3 Stage 3a | [design-mutual-recursion-lowering.md](./design-mutual-recursion-lowering.md) |

Dependencies between design docs (DB-4 references DB-1, DB-2 references DB-4 and DB-5): each design doc calls out its dependencies in the header. The doc set is internally consistent.

### Critical path

```
1a → 1b → 1c → 1d → 1e → 3c (self-hosting) = ~10 weeks
```

Everything else runs in parallel against this spine.

### What can start TODAY (before Half B merges)

- **Lane 3 Stage 3a** (M2 feature parity): zero dependencies on Half B's work. Mutual recursion, `data` semantics, `where` refinement, surface generics can all be designed and prototyped now.
- **Lane 4 Stage 4a design**: transport types can be sketched now; `dag run` interpreter scaffolding can start.

### What needs Half B + Lane 1 Stage 1b before starting

- Lane 2 (all stages) — the keyed-lookup accessors must exist
- Lane 3 Stage 3b — needs Lane 1c's CleanEmissionContract
- Lane 3 Stage 3c — needs Lane 1e's consolidated walker

---

## Coordination summary

**Hard handoffs (producer completes → consumer notified):**
1. Lane 1 Stage 1b → Lane 2 implementer: "substrate accessors ready"
2. Lane 1 Stage 1c → Lane 3 Stage 3b: "CleanEmissionContract shape locked"
3. Lane 1 Stage 1e → Lane 3 Stage 3c + Lane 4 Stage 4d: "generic walker ready"
4. Lane 2 Stage 2f → Lane 4 Stages 4b/4c: "Dimension framework ready to extend"
5. Lane 3 Stage 3a → Lane 4 Stage 4a: "M2 parity complete — transports can compile"

**Soft coordination (same surface, concurrent work):**
- Lane 2 diagnostics ↔ Lane 3b corrections: Lane 2's idempotency lens emits diagnostics; those diagnostics must carry the `Correction` shape Lane 3b defines. Mitigation: Lane 2 designs diagnostics against Lane 3b's target shape from day one.
- Lane 2 symbolic cost ↔ Lane 3 mutual recursion: symbolic cost (2d) reasons about recursion shape; mutual recursion (3a) adds SCC structure. Sequenced: 3a starts week 1, done by ~week 2.5, well before 2d starts ~week 7.

---

## Escalation protocol

Each lane has its own design doc with per-stage escalation criteria. Universal rules:

1. **Scope expansion** — if a stage needs more than +25% of its time budget, stop and escalate. Don't absorb silently.
2. **Substrate gaps** — if a stage reveals a substrate gap not in the design doc, stop and surface. Do not invent a local workaround.
3. **Cross-lane coupling surprise** — if Lane 2 work starts depending on Lane 3 work (or vice versa) in a way the plan doesn't predict, stop and reassess the sequencing.
4. **"This was supposed to be simpler"** — if a stage feels significantly harder than its design doc predicted, that's usually a sign the design missed something real. Escalate before pushing through.

---

## What will be TRUE when the plan completes

A concrete test for "did we finish":

- [ ] One `.dag` source compiles to Rust, Go, Python, Verilog, SPICE, and English with one `cargo run` invocation
- [ ] Zero `#[allow(warnings)]` attributes in the codebase
- [ ] Zero per-language emit files (`emit_X.rs` does not exist)
- [ ] Zero lens-local lookup helpers (every lens reads from substrate accessors)
- [ ] Every `#[ignore]`d test in v3 is either unignored or removed with explicit justification
- [ ] A cloud workflow declared non-idempotent fails compile with a specific diagnostic
- [ ] Running the v3 compiler binary on `compiler.dag` produces bit-identical Rust on back-to-back runs
- [ ] Every thesis-doc diagnostic example (`docs/error-examples.md`) has a live implementation with literal fix snippet
- [ ] v2's 16 idempotency tests all have v3 equivalents that pass
- [ ] `dag run fixture.dag` executes a cloud workflow through declared transports
- [ ] Side-effect and space-bound dimensions are live `Dimension` instances; violating declarations fail compile
- [ ] An async Rust variant of a program emits through the same walker as sync Rust with one spec-field change

**If any of these bullets fails, the plan is not done.** There is no backlog to push them to.

---

## What this plan explicitly deletes from the backlog

Everything previously marked "deferred M3/M4" or "what NOT to build yet" is now in a lane with acceptance gates:

- **M1(4) multi-target emission** → Lane 1e + 1f. Framing inverted: no more "one file per target."
- **M2 feature parity** (mutual recursion, `data`, `where`, generics) → Lane 3 Stage 3a.
- **M2 remaining** (transport declarations, `dag run` interpreter) → Lane 4 Stage 4a.
- **M3 self-hosting** → Lane 3 Stage 3c.
- **M4 thesis completion / all lenses** → Lane 2 (cost/parallelism/idempotency/user-dims) + Lane 1 (ownership/copy extracted as lenses) + Lane 4 (side effects, space bounds as Dimension instances). All lenses enumerated.
- **"Generic dimension mechanism"** → Lane 2 Stage 2f.
- **"Advanced diagnostics (Level 3 auto-fix)"** → Lane 3 Stage 3b.
- **"Async/concurrent emission strategies"** → Lane 4 Stage 4d. No longer deferred.

**True backlog = empty.** Everything either has a lane or is closed pre-plan.

---

## Coordination

- **Per-lane kickoff**: implementer reads the lane's master doc + relevant stage docs before any code. Confirms scope and escalation criteria.
- **Mid-stage**: stage escalates or continues; never silently expands.
- **Per-lane wrap**: all stage acceptance gates green before lane closes.
- **Cross-lane handoff**: when Lane 1 Stage 1b completes, explicit signal to Lane 2 implementer; when Lane 1 Stage 1e completes, explicit signal to Lane 3 implementer.

The point of this structure is to turn "what's left?" from a conversation into a glance at the acceptance bullets above.
