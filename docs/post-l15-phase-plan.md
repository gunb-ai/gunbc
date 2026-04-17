> Part of: [THESIS.md](../THESIS.md) > [src/v3/ROADMAP.md](../src/v3/ROADMAP.md)

# Post-A/B Lane Plan — Working backward from the thesis

**Status:** Active plan. Half A merged; Half B pending. Lane 1 can start today.
**Scope:** Four major lanes, sixteen stages total. Substantial parallelism (see sequencing section).
**Sizing:** Per-stage t-shirt sizes (S/M/L/XL). Lane totals are aggregate sizes, not calendar weeks.
**Discipline:** Every open thesis obligation is placed in a lane. **Nothing is backlog.**

---

## Thesis → lane derivation

Three load-bearing claims in THESIS.md. Working backward, each claim has a gap between what's declared and what the compiler actually enforces. One lane closes each gap.

| Thesis claim | Current gap | Lane that closes it |
|---|---|---|
| *"Emission is mechanical translation. Adding a new Shape A target = one spec file."* | Three hand-written per-target Rust emitters (emit_rust.rs 3600+ lines, emit_go.rs, emit_python.rs). Consumers reach for linear-list walks over `Dag.ports`/`Dag.nodes`. Adding a fourth Shape A programming-language target today costs a fourth per-language `.rs` file. | **Lane 1: Emission unification** |
| *"Correctness is many orthogonal dimensions… inescapable like conservation laws."* | Termination ✓ and structural cost ✓ proven at compile time. Idempotency **declared** in `dsl/std/effects.dag` with 16 v2 tests but **compiler consumption not wired** (THESIS.md:1291). Symbolic bounds, parallelism-as-diagnostic both declared as "NOT YET IMPLEMENTED". | **Lane 2: Compile-time proofs** |
| *"Causal engine. The compiler describes itself in .dag and is its own first consumer."* | Compiler is ~97% hand-written Rust sketch. `compiler.dag` exists (PR #418) but emit → compile → emit fixed-point not wired. Diagnostics explain in compiler-internal vocabulary, not user-pasteable corrections. Mutual recursion, `data` semantics, `where` refinement still block self-describing compiler. | **Lane 3: Self-hosting cycle** |
| *(tail obligations not cleanly derived from a single claim)* | Transport declarations, `dag run` interpreter, side effects as a compile-time dimension, space bounds as a compile-time dimension, async emission modeling. None fit Lanes 1–3's themes but all are thesis obligations. | **Lane 4: Completion layer** |

The four lanes exhaust the thesis. When all complete, gunbc cashes out its claim: one source, provably correct across every declared dimension, self-describing, executable through declared transports. There is no post-plan backlog.

**Shape B artifacts are explicitly out of scope** (SPICE netlists, Verilog, English docs, YAML, Terraform, K8s manifests, SQL schemas, etc.). Per THESIS.md §"Two shapes of omni-emission," these are outputs produced by `.dag` PROGRAMS using ordinary `concat`/`fold`/`match` operations — NOT compiler emission targets. A user writing a SPICE emitter writes a `.dag` library; the compiler's job is to compile that library to Rust/Go/Python so its users can invoke it. Enlarging the compiler core to know about Shape B formats is a category error.

---

## Lane summaries

### Lane 1 — Emission unification (XL, six stages)

**Closes:** "adding a new target = one spec file, zero new Rust"

Six internal stages. Each builds on the previous:

| Stage | Size | Scope | Design doc |
|---|---|---|---|
| 1a | M | L1.5 tail: Consumed rendering, Go unignores, receipts audit, m1_3 perf | [phase1-lane1-l15-tail.md](./phase1-lane1-l15-tail.md) |
| 1b | M | **Substrate keyed-lookup** — API shape is locked in [DB-5](./design-substrate-keyed-lookup-api.md): query FUNCTIONS (`port(d, id)`, `node(d, id)`, `resolve_producer(d, id)`) over the existing `List<DagPort>` / `List<Behavior>` authorities, not new parallel fields. Migrate existing 3 lenses; add INVARIANTS.md L-7 ("lenses don't reconstruct lookup locally"). | [lane1-stage-b-substrate-keyed-lookup.md](./lane1-stage-b-substrate-keyed-lookup.md) |
| 1c | M | Clean-emission invariant E-5: `CleanEmissionContract` per-target spec, pilot with unused pattern bindings | [phase1-lane2-clean-emission-invariant.md](./phase1-lane2-clean-emission-invariant.md) |
| 1d | M | Consolidation build plan: function inventory, spec gaps, bridge inventory, pilot target choice | [phase1-lane3-consolidation-build-plan.md](./phase1-lane3-consolidation-build-plan.md) |
| 1e | L | Consolidation execution: dissolve emit_rust.rs/go.rs/python.rs into one generic walker + per-target specs | (written at start of 1d, after build-plan locks design) |
| 1f | M | Consolidation proof: re-emission of Rust/Go/Python through the walker produces bit-identical output to current per-language emitters. Optional: add one additional Shape A language (another programming language, e.g. Swift/Kotlin) to prove "one new target = one spec file" | (written at end of 1e, sized by what consolidation actually shipped) |

**Acceptance:** `grep -r "fn render_" src/v3/compiler/src/` returns zero target-specific matches; Rust, Go, Python all roundtrip through a single generic walker with bit-identical output to pre-consolidation; zero `#[allow(warnings)]` attributes anywhere.

**Not in scope for Lane 1** (per THESIS.md §"Two shapes of omni-emission"): SPICE netlists, Verilog hardware descriptions, English documentation, YAML, Terraform, etc. These are **Shape B artifacts** — outputs of `.dag` PROGRAMS, not compiler emission targets. Writing a SPICE-netlist emitter is writing a `.dag` library, which any user program can invoke. Compiler core stays focused on Shape A (programming languages).

### Lane 2 — Compile-time proofs (XL, six stages, overlaps Lane 1 after 1b)

**Closes:** "every structural property gunbc claims is compile-time-enforced, not a runtime flag"

Six stages covering the three undeclared-compiler-consumption properties:

| Stage | Size | Scope |
|---|---|---|
| 2a | S | Port `dsl/std/effects.dag` → `src/v3/std/effects.dag`. Minimum carry-over; no new design. |
| 2b | L | Workflow idempotency lens: walks a pipeline, composes `EffectShape` per op via `compose_effects`, emits diagnostic pointing at the non-lattice op when chain breaks |
| 2c | M | Test obligation materialization: `generate_idempotency_obligations` spec → actually-emitted `f(f(x)) == f(x)` tests. End-to-end fixture: GCP bringup workflow compiler-proved idempotent, runnable test asserts it against mock API |
| 2d | M | L2 M1 symbolic cost bounds. Unignores `kf_1_lambda_body_cost_contributes_to_fold`. Structural cost composes through fold/map/loop with symbolic arity. O(n) vs O(n²) diagnostic |
| 2e | S | Parallelism-as-lens. Unignores `parallel_fold_on_commutative_monoid_is_reducible`. "Promotable to map" diagnostic becomes a lens output, not just a structural test |
| 2f | S | User-declared dimensions: infrastructure from 2b–2e generalizes so users can add custom compile-time proofs via `.dag` declaration (M4 thesis completion) |

**Acceptance:** the four previously `#[ignore]`d property tests are all green. A fixture that declares a non-idempotent cloud workflow (e.g., `POST /logs` in a retry loop) fails to compile with a diagnostic naming the breaking op. Symbolic cost reports O(n²) on nested fold. All v2 idempotency tests (`src/v2/tests/src/effects.rs`) have v3 equivalents passing.

Full design doc: [lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md)

### Lane 3 — Self-hosting cycle (XL, three stages; 3c gates on Lane 1e)

**Closes:** "the compiler is describable in .dag and is its own first consumer"

Three stages. This is M2 feature parity + diagnostics-as-corrections + the self-hosting cycle proper:

| Stage | Size | Scope |
|---|---|---|
| 3a | XL (5 sub-stages) | M2 feature parity for `compiler.dag`. Budget covers **design + implementation** for five substrate/surface extensions. Split: 3a.1 mutual recursion (L, design + impl via DB-9), 3a.2 `data` value semantics (S), 3a.3 `where` refinement (M), 3a.4 surface generics (S), 3a.5 Disj dotted-path parser extension (S, unblocks Half B B13). See [lane3 design](./lane3-self-hosting-cycle.md) |
| 3b | M | Diagnostics-as-corrections: every diagnostic carries `fix: List<Correction>` with literal code. Per-target fix syntax declared in spec (same `CleanEmissionContract` surface as Lane 1c — Rust fix syntax, Python fix syntax, etc.) |
| 3c | L | Self-hosting cycle: `compiler.dag` → Lane 1e emitter → Rust → `rustc` → v3_compiler binary. Fixed-point ratchet: re-emit is bit-identical. `cargo run --bin self-host-fixed-point` is a CI gate |

**Acceptance:** running the compiler binary on `compiler.dag` produces Rust identical to the previous run. Every diagnostic in `thesis_validation_test.rs`'s T-series emits a literal fix snippet. The compiler has dogfooded itself end-to-end.

Full design doc: [lane3-self-hosting-cycle.md](./lane3-self-hosting-cycle.md)

---

## Lane 4 — Completion layer (L, four stages)

**Closes:** everything else — transport declarations, `dag run` interpreter, side effects as a compile-time dimension, space bounds, async emission modeling.

Four stages:

| Stage | Size | Scope |
|---|---|---|
| 4a | L | Transport declarations — shape locked in [DB-6](./design-transport-taxonomy.md): substrate carries `TransportDeclaration { spec_ref, fields }`; transports are spec files under `extdeps/transports/`, not a closed compiler-side coproduct. Plus `dag run` interpreter (executes `.dag` by dispatching on `spec_ref`, not on a compiler enum). |
| 4b | M | Side effects as Dimension instance (extends Lane 2 2f framework). Workflow lens rejects hermetic/non-hermetic mixing |
| 4c | M | Space bounds as Dimension instance. Structural space cost composes through list operations. `where memory_bounded(…)` declarations enforce |
| 4d | S | Async emission modeling. Target spec declares async strategy; walker emits `async fn` / `.await` based on spec — zero new walker code |

**Acceptance:** `dag run` executes a cloud workflow through declared transports; side-effects and space-bounds dimensions have live workflow lenses; an async Rust variant of any program emits through the same walker as sync Rust.

Full design doc: [lane4-completion.md](./lane4-completion.md)

---

## Sequencing

Stage sizes and per-lane order (dependencies block below captures cross-lane gates):

```
Lane 1:  1a[M] → 1b[M] → 1c[M] → 1d[M] → 1e[L] → 1f[M]
Lane 2:                  2a[S] → 2b[L] → 2c[M] → 2d[M] → 2e[S] → 2f[S]
Lane 3:  3a[XL, 5 sub-stages] ─────────────────→ 3b[M] ─── (3c waits on 1e) ─── 3c[L]
Lane 4:                                                                        4a[L] → 4b[M] → 4c[M] → 4d[S]
```

Critical path: `1a → 1b → 1c → 1d → 1e → 3c` (six stages, one L, rest M). Stage 3a runs in parallel with Lane 1 throughout. Lane 2 starts once 1b lands. Lane 4 starts once 3a + 2f + 1e converge.

### Hard dependencies

```
1a → 1b, 1c, 1d (Lane 1 internal chain)
1b → 2a, 2b, 2c, 2d, 2e (Lane 2 reads substrate accessors)
1c → 3b (corrections share CleanEmissionContract surface)
1d → 1e (build plan gates execution)
1e → 1f, 3c, 4d (consolidated walker needed downstream)
3a → 3b, 3c, 4a (M2 parity blocks diagnostic-as-correction, self-hosting, transport/dag run)
2f → 4b, 4c (Dimension framework extends for side effects, space bounds)
2b–2e → 2f (user-dim abstraction generalizes over the concrete lenses)
```

**Note on 1a ↔ 3a**: Stage 3a (M2 feature parity) is *soft-coupled* to 1a (L1.5 tail), not hard-dependent. 3a CAN start in parallel with 1a if implementer slots allow; the sequencing diagram reflects this. Hard dep would be if 3a needed the post-1a repo state; it doesn't — 3a touches parser/lowering/substrate, 1a touches test cleanup and a renderer path.

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
| DB-10 | Lens Rust-boundary contract (L-8) | Lane 1 Stage 1a (cost lens migration) | [design-lens-rust-boundary.md](./design-lens-rust-boundary.md) |
| DB-10..DB-13 (m2) | M2 feature parity (data value semantics, `where` refinement, surface generics, Disj dotted-path) | Lane 3 Stage 3a.2 / 3a.3 / 3a.4 / 3a.5 | [design-m2-feature-parity.md](./design-m2-feature-parity.md) |
| DB-14 | Substrate external primitives (unblocks 1b accessors) | Lane 1 Stage 1b | [design-substrate-external-primitives.md](./design-substrate-external-primitives.md) |

**Numbering note.** PR #494 introduced `design-m2-feature-parity.md` with its four sub-blockers numbered `DB-10`..`DB-13`, colliding with the existing `DB-10` (Lens Rust-boundary) already listed above. The collision is visible here rather than papered over. Suggested cleanup in a separate PR: renumber `design-m2-feature-parity.md`'s sub-blockers to `DB-11`..`DB-14` and renumber DB-14 (substrate external primitives, this PR) to `DB-15`. Not blocking any implementation; readers can distinguish by design-doc link.

Dependencies between design docs (DB-4 references DB-1, DB-2 references DB-4 and DB-5, DB-10 added post-Half-A review, DB-14 added post-Stage-1b-escalation): each design doc calls out its dependencies in the header.

**Single authority (Invariant D-2).** When a DB doc locks a shape, the DB doc is the authority; lane and master docs reference it by link and prose summary, **not** by restating the fields. Reviewers have caught this drift multiple times — DB fixes a rejected shape, but a lane doc still paraphrases the old version. The ratchet below exists because "internally consistent" was a claim made twice before it was actually true; the forbidden-string list turns it into a mechanical check.

### Banked dissolutions — rejected shapes (ratchet)

Each design blocker records, in its rejected-alternatives section, a shape the doc set formally rejected. Those rejected names become a **forbidden-string ratchet**: lane docs and the master plan MUST NOT restate rejected shapes, even in prose. Invariant D-2 (DB is the single authority for its locked shape) covers the positive direction; this list covers the negative — the shapes that keep resurfacing and need a mechanical gate.

When a reviewer points at a rejected shape in a lane/master doc, the fix is to delete the restatement and reference the DB doc instead — not to argue the shape is compatible.

| Rejected shape (forbidden string) | Rejected in | Use instead |
|---|---|---|
| `port_by_id`, `node_by_id` (as substrate fields) | [DB-5](./design-substrate-keyed-lookup-api.md) | `port(d, id)`, `node(d, id)` query functions |
| `RestTransport`, `ShellTransport`, `GrpcTransport`, `TransportKind` (as closed compiler coproduct) | [DB-6](./design-transport-taxonomy.md) | `TransportDeclaration { spec_ref, fields }` |
| `target_language: TargetLanguageId?` on `Correction` | [DB-1](./design-correction-shape.md) | Future `TargetCorrection` carrier (out of scope for DB-1) |
| `struct_fields: StructFieldRule` / `AllowAttributeOnStructDecl` on `CleanEmissionContract` | [DB-4](./design-clean-emission-contract.md) | Visibility/publicity concern, outside E-5 |
| `MutualLoop` 6th `Behavior` variant | [DB-9](./design-mutual-recursion-lowering.md) | SCC fact at lens level; substrate stays at 5 behaviors |
| `#[deprecated]` shims for `emit_rust`/`emit_go`/`emit_python` | [DB-2](./design-generic-walker-api.md) | Atomic caller flip; delete old entrypoints in the same PR |
| New `Map<K, V>` declaration in v3 std (parallel to `dsl/std/types.dag`) | [DB-5](./design-substrate-keyed-lookup-api.md) | Port existing `Map<K, V> = PartialFunction<K, V>` binding |

**Mechanical gate (runs in Lane 1 Stage 1a):** a CI grep check that fails the build if any lane doc (`docs/lane*.md`, `docs/phase*.md`) contains a forbidden string. Two files are exempt from the scan because they legitimately enumerate rejected names: the DB docs themselves (`docs/design-*.md`) — where rejection is recorded — and this master plan (`docs/post-l15-phase-plan.md`) — where the ratchet itself lives.

The `FORBIDDEN=(...)` block below is the **single authority** for the ratchet. `scripts/check-banked-dissolutions.sh` parses this block directly — adding a rejected shape means extending this list in the master plan, not mirroring it in the script. The human-readable table above must stay in sync (enforced at review).

```bash
FORBIDDEN=(
  # Row 1 — DB-5: substrate fields
  "port_by_id" "node_by_id"
  # Row 2 — DB-6: closed transport coproduct
  "RestTransport" "ShellTransport" "GrpcTransport" "TransportKind"
  # Row 3 — DB-1: target_language on Correction
  "target_language: TargetLanguageId"
  # Row 4 — DB-4: clean-emission contract fields
  "struct_fields: StructFieldRule"
  "StructFieldRule" "AllowAttributeOnStructDecl"
  # Row 5 — DB-9: 6th Behavior variant
  "MutualLoop"
  # Row 6 — DB-2: emitter shim deprecations
  "#[deprecated]"
  # Row 7 — DB-5: new Map declaration in v3 std (parallel to dsl/std/types.dag)
  "Map<K, V>"
)
# Scan: docs/lane*.md and docs/phase*.md, fail on any match.
```

**Coverage invariant.** The array above carries at least one forbidden
substring for every row in the rejected-shape table. When a new row is
added to the table, the corresponding strings land in the array in the
same edit. Conversely, the array must not carry entries whose
rejections aren't documented in a table row — every entry is
answerable with a DB doc reference. Reviewers enforce both directions
by hand when the table/array change; the per-row comments above
cross-reference DB docs to make the mapping explicit.

When a future DB doc rejects a shape, add the rejected name to this list as part of the DB's acceptance. The list only grows; a banked dissolution is permanent.

## Half B → lane dissolution map

Half B (PR #490) landed 8 of 9 original blockers + 3 new issues. Classifying each against the post-merge lanes helps reviewers understand what's durable vs what has a short half-life in Lane 1e consolidation:

**Survives consolidation (substrate / semantic invariants):**
- B1 typed `CallableParameter` — slot-keyed shape; foundational for DB-6 transport taxonomy
- B7 `Bind`-not-consume — permanent semantic invariant (reinforced in DB-9 mutual recursion cluster analysis)
- B11 typed `TargetLanguage` dispatch — the pattern persists; field names may reshape in Lane 1e consolidation but name-prefix dispatch is gone
- B-NEW-1 dissolution receipts on all new sum types — modeling-discipline pattern that keeps

**Dissolves in Lane 1e (emit_*.rs → single generic walker + specs):**
- B3 last-use tracking → moves to dedicated ownership lens
- B4 `decl_is_copy` cycle guard → becomes a copy-type lens
- B6 fail-closed in `analyze_user_defined_callable` → folded into walker realization dispatch
- B14 operand algebra walk in `emit_python` → spec-driven dispatch per DB-2
- B-NEW-2 Go consumes `parameter_dispositions` → walker reads from CallableRealization spec
- B-NEW-3 Python `TargetExecutionModel` typed → walker reads from target spec

**Deferred to an explicit future lane:**
- B13 (.dag parser dotted-path Disj variant access) → Lane 3 Stage 3a (M2 feature parity)

**Pessimistic fallbacks to revisit in Lane 1e:** `decl_is_copy` structural walk (over-eager Copy classification) and `OwnedConstructLastUse` optimization (unsound under template reorder) were reverted during Half B merge reconciliation. Clone ratchet bumped 1 → 5 (vs main's 6). These are known-pessimistic areas catalogued in Lane 1 Stage 1d's [consolidation build plan](./phase1-lane3-consolidation-build-plan.md) §5.

### Critical path

```
1a → 1b → 1c → 1d → 1e → 3c (self-hosting)
```

Six stages: five M, one L. Everything else runs in parallel against this spine.

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
- Lane 2 symbolic cost ↔ Lane 3 mutual recursion: symbolic cost (2d) reasons about recursion shape; mutual recursion (3a.1) adds SCC structure. Sequenced: 3a.1 lands before 2d starts (3a runs in parallel with Lane 1 from the start; 2d gates on Lane 1 1b + prior Lane 2 stages).

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

- [ ] One `.dag` source compiles to Rust, Go, Python through one generic walker + per-target specs; adding a fourth Shape A programming language requires one spec file, zero new Rust
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
- [ ] No `.dag` lens's public Rust surface erases its typed failure carrier via panic or opaque primitive — L-8 invariant enforced by CI grep gate (see [design-lens-rust-boundary.md](./design-lens-rust-boundary.md))

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
