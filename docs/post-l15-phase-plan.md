> Part of: [THESIS.md](../THESIS.md) > [src/v3/ROADMAP.md](../src/v3/ROADMAP.md)

# Post-A/B Lane Plan — Working backward from the thesis

**Status:** Active plan. Half A merged; Half B pending. Starts immediately after Half B lands.
**Total time:** ~12 weeks, three major lanes (Lane 1 and Lane 2 overlap).
**Discipline:** Every open thesis obligation is placed in a lane. **Nothing is backlog.**

---

## Thesis → lane derivation

Three load-bearing claims in THESIS.md. Working backward, each claim has a gap between what's declared and what the compiler actually enforces. One lane closes each gap.

| Thesis claim | Current gap | Lane that closes it |
|---|---|---|
| *"Emission is mechanical translation. Adding a new target = one spec file."* | Three hand-written per-target Rust emitters (emit_rust.rs 3600+ lines, emit_go.rs, emit_python.rs). Substrate exposes `Dag.ports`/`Dag.nodes` as linear lists, forcing every lens to reinvent `find_port`/`find_behavior`. Targets beyond Rust/Go/Python (Verilog, SPICE, English) named in architecture.md but zero code. | **Lane 1: Emission unification** |
| *"Correctness is many orthogonal dimensions… inescapable like conservation laws."* | Termination ✓ and structural cost ✓ proven at compile time. Idempotency **declared** in `dsl/std/effects.dag` with 16 v2 tests but **compiler consumption not wired** (THESIS.md:1291). Symbolic bounds, parallelism-as-diagnostic both declared as "NOT YET IMPLEMENTED". | **Lane 2: Compile-time proofs** |
| *"Causal engine. The compiler describes itself in .dag and is its own first consumer."* | Compiler is ~97% hand-written Rust sketch. `compiler.dag` exists (PR #418) but emit → compile → emit fixed-point not wired. Diagnostics explain in compiler-internal vocabulary, not user-pasteable corrections. Mutual recursion, `data` semantics, `where` refinement still block self-describing compiler. | **Lane 3: Self-hosting cycle** |

The lanes exhaust the thesis. When all three complete, gunbc cashes out its claim: one source, provably correct across orthogonal dimensions, self-describing. There is no post-plan backlog.

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

## Sequencing

```
Week:    1   2   3   4   5   6   7   8   9  10  11  12
Lane 1: [1a][1b][1c][1d][  1e (2wk)  ][1f]
Lane 2:         [2a][ 2b   ][2c ][2d][2e][2f]
Lane 3:                             [  3a  ][3b][  3c  ]
```

**Hard dependencies:**
- Lane 1 Stage 1b must complete before Lane 2 starts — Lane 2 lenses need keyed substrate accessors
- Lane 3 Stage 3c needs Lane 1 Stage 1e complete — self-hosting through fragmented emitters is worthless
- Lane 3 Stage 3a can start before Lane 1 finishes if implementer slots allow; 3b and 3c are sequential

**Soft parallelism:**
- Lane 1 and Lane 2 run in parallel from week 3; Lane 2 implementer can block on 1b completion signal
- Lane 2 Stage 2d can start any time after 2a (independent of idempotency work)

---

## Escalation protocol

Each lane has its own design doc with per-stage escalation criteria. Universal rules:

1. **Scope expansion** — if a stage needs more than +25% of its time budget, stop and escalate. Don't absorb silently.
2. **Substrate gaps** — if a stage reveals a substrate gap not in the design doc, stop and surface. Do not invent a local workaround.
3. **Cross-lane coupling surprise** — if Lane 2 work starts depending on Lane 3 work (or vice versa) in a way the plan doesn't predict, stop and reassess the sequencing.
4. **"This was supposed to be simpler"** — if a stage feels significantly harder than its design doc predicted, that's usually a sign the design missed something real. Escalate before pushing through.

---

## What will be TRUE when the plan completes

A one-line test for "did we finish":

- [ ] One `.dag` source compiles to Rust, Go, Python, Verilog, SPICE, and English with one `cargo run` invocation
- [ ] Zero `#[allow(warnings)]` attributes in the codebase
- [ ] Zero per-language emit files (`emit_X.rs` does not exist)
- [ ] Zero lens-local lookup helpers (every lens reads from substrate accessors)
- [ ] Every `#[ignore]`d test in v3 is either unignored or removed with explicit justification
- [ ] A cloud workflow declared non-idempotent fails compile with a specific diagnostic
- [ ] Running the v3 compiler binary on `compiler.dag` produces bit-identical Rust on back-to-back runs
- [ ] Every thesis-doc diagnostic example (`docs/error-examples.md`) has a live implementation with literal fix snippet
- [ ] v2's 16 idempotency tests all have v3 equivalents that pass

**If any of these bullets fails, the plan is not done.** There is no backlog to push them to.

---

## What this plan explicitly deletes from the backlog

These items previously sat as "deferred M3/M4" or "what NOT to build yet." The plan absorbs them:

- **M1(4) multi-target emission** — was "add go.dag, python.dag as parallel fixtures." Absorbed into Lane 1e + 1f. Framing inverted: no more "one file per target."
- **M2 feature parity** (mutual recursion, `data`, `where`, generics) — Lane 3 Stage 3a.
- **M3 self-hosting** — Lane 3 Stage 3c. The .dag rewrite of the compiler IS the self-hosting cycle running on `compiler.dag`.
- **M4 thesis completion / all lenses** — Lane 2 delivers cost/parallelism/idempotency/user-dimensions; Lane 1 extracts ownership/copy as lenses during consolidation. "All lenses" is no longer a vague milestone — it's a concrete list, all placed.
- **"Generic dimension mechanism"** (was "what NOT to build yet") — Lane 2 Stage 2f. The Lane 2 infrastructure naturally supports user-declared dimensions; delivering them is a 0.5-week stage at the end.
- **"Advanced diagnostics (Level 3 auto-fix)"** (was "what NOT to build yet") — Lane 3 Stage 3b.
- **"Async/concurrent emission strategies"** (was "what NOT to build yet") — deliberately NOT in the plan. This is a different axis (execution concurrency), not thesis-closure. Legitimate backlog for a later planning round; does not block thesis cash-out.

---

## Coordination

- **Per-lane kickoff**: implementer reads the lane's master doc + relevant stage docs before any code. Confirms scope and escalation criteria.
- **Mid-stage**: stage escalates or continues; never silently expands.
- **Per-lane wrap**: all stage acceptance gates green before lane closes.
- **Cross-lane handoff**: when Lane 1 Stage 1b completes, explicit signal to Lane 2 implementer; when Lane 1 Stage 1e completes, explicit signal to Lane 3 implementer.

The point of this structure is to turn "what's left?" from a conversation into a glance at the acceptance bullets above.
