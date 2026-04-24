# gunbc Roadmap

Single source of truth for project status, active work, and deferred items. Long-form receipts and historical narratives now live under `docs/history/` and `docs/db-history/` so this file can stay operational.

> Design spec: [docs/v3-spec.md](docs/v3-spec.md)
> Validation: [docs/v3-validation-experiments.md](docs/v3-validation-experiments.md)
> Lineage: [docs/design-lineage.md](docs/design-lineage.md)
> **Lens capability register: [docs/v3-lens-capability-register.md](docs/v3-lens-capability-register.md) — read before dispatching any brief that assumes "v3 subsumes v2 X".**
> **Compiler–`std/` consolidation end state: [docs/thesis/compiler-std-consolidation.md](docs/thesis/compiler-std-consolidation.md) — new types in `src/v3/compiler/*.dag`, `src/v3/lenses/*.dag`, or `src/v3/std/*.dag` require a home-check against the positive definition (pipeline types / regen types / lens-specific return carriers / substrate accessors). Lens-local 2-variant `Missing | Found(T)` Lookup duplicates count against the ratchet; carriers with distinct semantic variants stay lens-API. Everything else schedules migration to `std/`.**

## How this doc is organized

Read this file for the live plan, milestone state, and current DB status lines. Read [docs/history/roadmap-post-ab-lane-plan.md](docs/history/roadmap-post-ab-lane-plan.md), [docs/history/roadmap-active-deferrals.md](docs/history/roadmap-active-deferrals.md), and [docs/history/roadmap-scheduled-deletions.md](docs/history/roadmap-scheduled-deletions.md) for full receipts and narrative detail.

## Release R1 Program

**Goal.** First external discussion of gunbc, to a compiler-literate audience. Not a toy compiler — a working demonstration of the thesis on real programs, shown to principal engineers and compiler nerds first.

**Not the goal.** Strict v2 feature parity across every lens; consumer-facing polish; web/agentic marketing push. Those are R2+.

**Meta-acceptance (two stages).** R1 ships when every lane's acceptance `TestClaim`:

- **(a) compiles as a `.dag` declaration** — predicates already in today's DB-15 schema compile from Day 1; predicates scheduled for T-TestGen schema extensions compile once T-TestGen lands those additions. Not every gate is Day-1 compilable — see `### Lane acceptance` below for the per-predicate split.
- **(b) evaluates true at release** — requires T-TestGen's runner closure.

T-TestGen is therefore the gate-enabling lane on two axes: it extends the predicate vocabulary where gates need new predicates, and it lands the runner that evaluates every predicate structurally. The release gate IS a `.dag` program. This is the thesis eating its own dogfood.

**Debt paydown continues in parallel.** R1 does not freeze the tracked-debt ledger. The lane structure names forward deliverables; the ledger below keeps dispatching — CI ratchet audit, stale-brief sweep, INVARIANTS cross-ref cleanup, scheduled-deletion work. Treat debt work as a continuous **T-Receipts** track (bundle 2-4 items per PR per the standing preference).

### Goals (the six non-negotiables)

1. **Complexity lens at v2 parity or beyond — no debt.** Structured `CostExpr(work, span, asymptotic_class)` from structure. Lane T-LaneE.
2. **Test generation + integration + service simulation — first-class.** Tests are `.dag` data. Lane T-TestGen.
3. **Multi-target emission.** Rust production-grade; Python/Go demonstrably working. Lane T-Emit.
4. **Impossible-bugs demo suite.** Enumerated bug classes with compile-time proofs (see THESIS `Enumerable impossible-bug classes`). Lane T-Demo.
5. **Arbitrary lens composition.** User-authored lenses. Lane T-LensAPI.
6. **Self-hosting (Pure Bootstrap).** Hand-authored compiler files at the irreducible-shim floor (≤5 per `docs/design-pure-bootstrap.md`); generated escape hatch acceptable for additional files. Lanes T-PB-A + T-PB-B.

Enablers (prerequisites for the goals): T-P0 (bug sweep), T-Sub (surface syntax completion).

### Nine lanes

Each lane owns one concrete `.dag` gate. Lane owners do the comprehensive decomposition; this section holds intent and acceptance only.

| Lane | Size | Covers | Cross-ref into debt ledger |
|------|------|--------|----------------------------|
| T-P0 | S | P0 sweep (repeat_string, REST_OPS, no_profile_sentinel) | §P0 — real bugs |
| T-Sub | S | `match` over user sums (landed PR #702), `CharClass` in std.unicode, type-alias `where` (landed PR #703) | §P4 (bit.dag refinements), Character-level under-consumption |
| T-Emit | M | Rust harden, #650 generic-bound fidelity, Python/Go reconcile | SurfaceLiteral→LiteralBits, variant-constructor template |
| T-LaneE | XL | Complexity lens v2 parity via substrate-carrier-port | Existing Lane E-T/C/I/P/M program |
| T-TestGen | L | Testgen runner, service simulation, first-class TestClaim | DB-15 follow-up |
| T-LensAPI | M-L | User-authored lenses + composition | Lens capability honesty pass |
| T-PB-A | XL | Compiler self-emits (fixed-point); **non-test** hand-Rust surface reaches the ≤5 irreducible-shim floor (per `docs/design-pure-bootstrap.md`; generated escape hatch OK). Live baseline is the non-test subset of the SG-0 census (`EXPECTED_HAND_AUTHORED_NON_TEST` file-level + `EXPECTED_HAND_AUTHORED_FRAGMENTS` crate-root scaffolds) in `src/v3/compiler/tests/integration/sg0_census_test.rs` — this doc does not freeze the count. **Lens-producer files are the priority slice within this census** (concrete examples: `per_call_descent_evidence` at `src/v3/compiler/src/dag.rs`; the cost / complexity / descent-evidence producer family around the E-family port). Retiring a lens-producer `.rs` dissolves "lens purity by reviewer-convention" into "lens purity by construction" via the closed-world `.dag` kernel — the bounded kernel admits no hidden computation in a lens body, so `.dag`-authored lens producers are pure-total-projections by construction rather than by audit. That purity is what makes lens sustainability structural rather than carried by reviewer memory. Tracked as the `lens_producer_files_remaining` gate below. The **test subset** (`EXPECTED_HAND_AUTHORED_TEST`) of the same census is T-PB-B's responsibility, not T-PB-A's. | Compiler–std consolidation program, SG-0 census |
| T-PB-B | M | Tests-as-data — pipeline/contract tests port to `.dag`. The two `TESTING.md §Post-R2 shape` residual categories (compiler-internal unit tests for Rust-only helpers; external-toolchain boundary tests invoking rustc/go/python) remain Rust-authored. | DB-15 + T-TestGen |
| T-Demo | M | Two canonical fixtures + impossible-bugs suite + narrative | — (new) |

### Lane acceptance — `.dag` gates

This section lists gate names + schema-compilability tags; full `TestClaim` declarations will land as deliverables of the lane-brief drafting step (lane owners author them as `.dag` after being named). Each predicate is tagged `[Day 1]` (compiles against today's DB-15 schema — `Compiles`, `FailsWithDiagnostic`, `OutputEquals`, `CostBounded`, `PortHasState`) or `[ext]` (requires a T-TestGen schema extension before compiling). Day-1 predicates are a minority — the majority block on T-TestGen's runner + schema work, which is why T-TestGen is the gate-enabling lane.

- **T-P0.** `p0_repeat_string_correct` [Day 1] · `p0_no_fabrication_sentinel` [ext] · `p0_rest_ops_aligned` [ext]
- **T-Sub.** `sub_match_over_user_sum` [Day 1, landed PR #702] · `sub_type_alias_where_lowers` [ext, landed PR #703] · `sub_charclass_in_std_unicode` [ext, open: phase-2 reproduction/triage]
- **T-Emit.** `emit_rust_fixtures_rustc_green` [ext: `ExecuteCommand`] · `emit_generic_bounds_survive` [ext] · `emit_omni_demo_fixtures_green` [ext: `ForAllTargets` + `ExecuteCommand`]
- **T-LaneE.** `complexity_merge_sort_is_nlogn` [ext: `LensOutputEquals`] · `complexity_v3_matches_v2_oracle` [ext: `DifferentialEquals`]
- **T-TestGen.** `testgen_structural_coverage` [ext] · `testgen_mock_backed_integration_safe` [ext: `MockBackedInvariant` wiring] · `testgen_manual_claim_is_first_class` [ext] — T-TestGen also owns scoping the predicate shape for `[ext]` gates that other lanes consume; currently includes `lens_producer_files_remaining` for T-PB-A (enumeration declared in `sg0_census_test.rs` at scoping time).
- **T-LensAPI.** `user_authored_lens_compiles` [Day 1] · `lens_composition_associative` [ext: `AlgebraicLaw`] · `lens_output_is_queryable_data` [ext]
- **T-PB-A.** `pb_hand_rust_at_shim_floor` [ext] · `lens_producer_files_remaining` [ext — priority slice of the non-test census: hand-Rust files implementing lens producers, drops as each migrates to `.dag`; enumeration declared in `sg0_census_test.rs` when T-TestGen scopes the predicate] · `pb_self_compile_fixed_point` [ext] · `pb_compiler_std_ratchet_zero` [ext] — live baselines read from authorities; not frozen in this doc. Hand-Rust (**non-test subset**): SG-0 census (file-level + fragments ratchet) minus the test subset owned by T-PB-B → ≤5 irreducible-shim per `docs/design-pure-bootstrap.md`. Consolidation ratchet: compiler-local types not in positive-def set → 0.
- **T-PB-B.** `pb_test_file_generated_from_dag` [ext] · `pb_rust_tests_outside_residual_zero` [ext] — the first gates the pipeline-equivalent suite; the second gates the outcome: zero Rust-authored tests exist outside the `TESTING.md §"Post-R2 shape"` residual (compiler-internal unit tests + external-toolchain boundary tests). Single-file generation is insufficient proof of the lane's end-state.
- **T-Demo.**
  - `fixture_compiler_nerd_canonical` [Day 1 (Compiles) / ext (lens-output demos)] — demonstrates: complexity, ownership, parallelism
  - `fixture_integration_canonical` [Day 1 (Compiles) / ext (lens-output demos)] — demonstrates: effects, idempotency, testgen
  - `impossible_bug_class_suite_r1` [ext] — three classes demoed: idempotency-violation, suboptimal-complexity, transport/type-drift. Remaining three (nested-optional flatten, unhandled diagnostic paths, unenumerated effects) are tagged **[R2+]** in THESIS — thesis-committed but not scheduled to a specific release. THESIS §"Enumerable impossible-bug classes" is the authority on scheduling tags.
  - `demo_user_authored_lens_rejects_violating_program` [ext] — operationalizes THESIS §"User-defined dimensions". Demo shows a user-written lens (~20 lines of `.dag`; e.g., "max external HTTP calls per workflow") rejecting a program that violates it, alongside the built-in complexity lens. Proves the ceiling of what gunbc can prove is user-extensible, not compiler-baked. Consumes `user_authored_lens_compiles` from T-LensAPI.

**T-Demo scoping note.** All lanes ship features whole (no compromise per §Goals). T-Demo curates the R1 *narrative* — fixtures and impossible-bug demos selected for visceral audience impact, not exhaustive feature coverage. Audience curation is demo-scoping; feature shipment is lane-scoping.

### Scheduled cleanups: LensOutputEquals runner and R1 gate fixtures

Day-1 PR #717 landed explicit `LensOutputEquals` dispatch and fixtures with **no inline `TODO` in code**: the items below are the authoritative schedule (T-LensAPI / compiler lowering / schema), not drive-by debt comments.

1. **`eval_lens_output_equals` witness compile** (`src/v3/compiler/src/test_runner.rs`): remove the side `compile_to_dag(&claim.source, …)` once the runner performs real `LensOutputEquals` evaluation so compilation lives only on the apply/compare path. Until then the receipt is intentionally thin (trivial fixture `source`); a lowering failure can surface as `Fail` while the predicate is still runner-deferred.

2. **Split fixture** (`src/v3/compiler/tests/fixtures/r1_lens_output_equals_gate.dag`): delete the file and fold its claim into `src/v3/compiler/tests/fixtures/r1_gates.dag` after same-TU lowering fixes `DeclarationRef` / `Int` resolution when `LensOutputEquals(Int, …)` shares a module with the large embedded `user_authored_lens_compiles_gate` `source` string. Cross-ref: **T-LensAPI**, **T-LaneE** lane acceptance (`LensOutputEquals` [ext]). Check: merged fixture + `cargo test -p v3-compiler` green, then remove the split file.

3. **P2 parallel lens text** (`r1_gates.dag` + `m1_5_user_authored_lens_gate_test.rs` lockstep): remove the duplicated `TestClaim.source` mirror of `src/v3/lenses/named_function_count.dag` and delete the byte-identical ratchet test when a single authority exists (generated fixture splice, `TestClaim` / substrate path resolution for `source`, or runner-resolved lens `DeclarationRef` from the bootstrap DAG per INVARIANTS.md P2).

### Dependency DAG

```
┌─────────────────────────────────────────────────────────────────┐
│                       RELEASE R1 GATE                           │
│     all lane TestClaim declarations compile + eval true         │
└────────────────────────────▲────────────────────────────────────┘
                             │
                    ┌────────┴──────────┐
                    │  [M] T-Demo       │
                    │  fixtures +       │
                    │  impossible-bugs  │
                    └────────▲──────────┘
         ┌───────────┬───────┼───────┬───────────┐
         │           │       │       │           │
         │      ┌────┴───┐   │   ┌───┴──────┐    │
         │      │ [XL]   │   │   │  [M-L]   │    │
         │      │T-LaneE │   │   │T-LensAPI │    │
         │      └────────┘   │   └──────────┘    │
         │                   │                   │
   ┌─────┴─────┐      ┌──────┴──────┐     ┌─────┴─────┐
   │ [XL]      │      │ [M] T-PB-B  │     │ [M]       │
   │ T-PB-A    │      │             │     │ T-Emit    │
   └─────▲─────┘      └──────▲──────┘     └─────▲─────┘
         │                   │                   │
         │          ┌────────┴────────┐          │
         │          │  [L] T-TestGen  │          │
         │          └────────▲────────┘          │
         │                   │                   │
         └───────────────────┼───────────────────┘
                             │
                    ┌────────┴────────┐
                    │  [S] T-Sub      │
                    └────────▲────────┘
                             │
                    ┌────────┴────────┐
                    │  [S] T-P0       │
                    └─────────────────┘
```

Read bottom-up. Arrows point producer → consumer.

**Critical path.** `max(T-LaneE, T-PB-A, T-Sub → T-TestGen → T-PB-B) → T-Demo`.

- T-LaneE and T-PB-A do not gate each other; both XL. T-LaneE runs from W0 with no upstream dependencies. T-PB-A starts W0 in parallel. Its former match-emit-dependent clusters (regen-emits-match, variant-constructor templates) are no longer blocked on T-Sub's `sub_match_over_user_sum` gate because PR #702 landed that receipt. The DAG above is authoritative on the specific cluster-level edges.
- T-LensAPI is decoupled, starts W0.
- T-TestGen is the serial hinge for T-PB-B.
- T-Emit (M) feeds T-Demo but is off the critical path — the XL lanes dominate its duration, so T-Emit is slack relative to T-LaneE / T-PB-A.

**Two distinct baselines inside T-PB-A.** Both read from live authorities — this doc does not freeze counts.

- **Hand-Rust census (T-PB-A owns the non-test subset):** SG-0 census — `EXPECTED_HAND_AUTHORED_NON_TEST` + `EXPECTED_HAND_AUTHORED_FRAGMENTS` for the T-PB-A subset, and `EXPECTED_HAND_AUTHORED_TEST` for the T-PB-B subset — in `src/v3/compiler/tests/integration/sg0_census_test.rs`. T-PB-A's gate scopes to the **non-test entries** → ≤5 irreducible-shim (per `docs/design-pure-bootstrap.md`). The **test entries** of the same census are T-PB-B's gate: zero Rust-authored tests **outside the TESTING.md §"Post-R2 shape" residual** (compiler-internal unit tests + external-toolchain boundary tests, which stay Rust-authored permanently per TESTING.md as single authority). The census floor is therefore `(≤5 irreducible-shim per design doc) + (TESTING.md residual)`, not the shim floor alone; neither lane dissolves the TESTING.md residual, and each lane is a scoped half of the census, not blocked on the other for work.
- **Compiler–std consolidation ratchet:** compiler-local `type` declarations not in the positive-def set and not exempted → 0 (see ROADMAP §"Compiler–`std/` consolidation program"). What's allowed to exist as a compiler-local type name.

Both land inside T-PB-A because dissolving the compiler-local surface forces both gates down together, but the acceptance claims (`pb_hand_rust_at_shim_floor`, `pb_compiler_std_ratchet_zero`) are independent gates tracked against independent authorities.

**SG-0 ratchet split is structural.** The R1 split (T-PB-A owns non-test; T-PB-B owns test) is now mechanically checked by `sg0_census_test.rs`: `EXPECTED_HAND_AUTHORED_NON_TEST` plus `EXPECTED_HAND_AUTHORED_FRAGMENTS` form the T-PB-A sub-ratchet, while `EXPECTED_HAND_AUTHORED_TEST` forms the T-PB-B sub-ratchet. The total census still checks their union against the generated-file partition. `pb_hand_rust_at_shim_floor` / `pb_rust_tests_outside_residual_zero` predicates still need to name this partition once T-TestGen extensions support them.

### Relationship to existing milestone status

R1 absorbs what was "Post-A/B Lane Plan" and L1.5 forward work, framed by release deliverable rather than architectural stage. The status table below stays accurate for backward-looking context; R1 is the forward-looking companion.

## Post-R1 Program — Grounding Completeness

Named post-R1 program promoted from the target-grounding proposal ([PR #695](https://github.com/gunb-ai/gunbc/pull/695), landing at `docs/thesis/target-grounding-proposal.md` on merge; `PROPOSAL` mode on the proposal doc itself; this ROADMAP section is the committed scope counterpart). Architectural authority remains [`docs/single-emitter-design.md`](docs/single-emitter-design.md) (ROADMAP Track 13 / single-emitter dissolution). This section names concrete lanes; the proposal doc carries worked examples and work-estimate detail.

**The claim.** Target-side primitive types in Rust, Python, and Go are structurally modeled from their language references (Rust Reference §Types at <https://doc.rust-lang.org/reference/types.html>; Python data model; Go specification). Algebra inhabitance is declared structurally on target primitives, parallel to how user-side `.dag` types declare inhabitance (`Int64 = OrderedRing<Word64>`). Coercion is a structural algebra-homomorphism search, not a name-keyed table lookup. The current `TypeCheckpoint` / `InhabitantDecl` / `carrier: String` surface is bootstrap scaffolding that dissolves via Track 13 closure at the end of this program.

**Why post-R1.** This is a substantial program (sized ~T-LaneE equivalent), cleanly separable from R1's release gates, and blocked in full on post-R1 substrate capabilities (DB-11 alias-RHS `where` parsing; cardinality-substrate; optionally DB-18 parametric algebra attachment). A Day-1-dispatchable pilot lane exists that uses only live substrate.

### Post-R1 Grounding lanes

| Lane | Size | Covers | Blockers |
|------|------|--------|----------|
| T-Ground-Pilot | S | Rust integer family (i8–i64, u8–u64, bool, Unit) + toy inhabitance-search engine + routing-stability tests demonstrating parity with current table lookup | None — uses only live substrate; dispatchable post-R1 Day-1 |
| T-Ground-Rust | XL | Two-authority split: **(a) Rust Reference §Types** (<https://doc.rust-lang.org/reference/types.html>) — language-level structural types: boolean, numeric (integer + floating-point), textual (`char`, `str`), never, tuple, array, slice, struct, enum, union, function item, function pointer (`fn(...) -> ...`), closure, reference (`&T`, `&mut T`), raw pointer (`*const T`, `*mut T`), trait object (`dyn Trait`), `impl Trait`. **(b) std-library carriers** (std documentation is the authority, separate from the reference) — `String`, `Vec<T>`, `Box<T>`, `Rc<T>`, `Arc<T>`, `HashMap<K,V>`, `BTreeMap<K,V>`, `HashSet<T>`, `BTreeSet<T>`, `Option<T>`, `Result<T, E>`. Each category cites its own authority; mixing them into one "Rust Reference" claim is a faithfulness violation. | DB-11 (refinement-carrying qualifiers on primitives), cardinality-substrate (container cardinality bounds) |
| T-Ground-Python | L | Two-authority split: **(a) Python language reference** — built-in types (numeric int/float/complex, bool, None, sequence list/tuple/range, text str, binary bytes/bytearray, mapping dict, set, frozenset, callable, module, class). **(b) CPython stdlib** (`typing`, `collections`, etc. — when lanes grow beyond built-ins). | DB-11; cardinality-substrate |
| T-Ground-Go | L | Two-authority split: **(a) Go language specification** (<https://go.dev/ref/spec>) — boolean, numeric, string, array, slice, struct, pointer, function, interface, map, channel. **(b) Go standard library carriers** when modeling beyond the spec's primitives. | DB-11; cardinality-substrate |
| T-Ground-Engine | M | Inhabitance-search walker; minimum-satisfier selection; fail-closed tie-breaking with structured diagnostic (per the target-grounding proposal's tie-breaking discipline — [PR #695](https://github.com/gunb-ai/gunbc/pull/695)) | Layers 1–3 populated to a useful coverage threshold |
| T-Ground-Tests | S | Routing-stability TestClaim class; L4 witness-based certification for target-side algebra-inhabitance claims per `verifiability-invariant.md` ("consistent by construction + verified by L4") | T-TestGen runner green (R1 deliverable); layers 1–4 |
| T-Ground-Dissolve | S | Track 13 closure — single PR deleting the coercion scaffolding entirely: **`dsl/std/coercion.dag`** (schema file: `TypeCheckpoint` / `InhabitantDecl` / `CallableRepr` / `CastSyntax`), **`dsl/extdeps/languages/{rust,python,go,dag}/types.dag`** (per-target instantiation tables), **`TypeRealization.carrier: String`** field in `src/v3/std/emit_model.dag`, and every emit-pipeline call site reading the old surface. Routing-stability assertions are authored as direct `TestClaim` declarations (not graduations of `TypeCheckpoint` / `InhabitantDecl` data), so the scaffolding's testgen-assertion role dissolves alongside its routing-authority role. | All other lanes reaching parity with current table |

**Critical path:** `T-Ground-Pilot → T-Ground-Rust → T-Ground-Engine → T-Ground-Tests → T-Ground-Dissolve`. Python and Go lanes run parallel after Pilot validates the pattern.

**Grounding Manager** coordinates per [`docs/briefs/grounding-manager.md`](docs/briefs/grounding-manager.md). R1 Director Brief's escalation discipline continues to apply: scope changes to this program route to director; amendments to THESIS.md §"Grounding completeness" / this ROADMAP section require director-authored PRs.

**Acceptance gates** (to be authored as `TestClaim` declarations once T-TestGen runner is green):
- `ground_rust_int_family_structurally_declared` [Day 1 after Pilot]
- `ground_rust_reference_complete` [ext: after DB-11 + cardinality-substrate]
- `ground_routing_stability` [ext: after engine]
- `ground_l4_certified_rust_i64_inhabits_orderedring` [ext: after tests]
- `ground_track13_dissolution_complete` [ext: single Dissolve PR]

**Thesis claim tracked by this program:** "Grounding completeness" per THESIS.md §"Thesis claims — complete list" (Tier 1 — Structural correctness).

## Status at a glance

| Milestone | State | Notes |
|-----------|-------|-------|
| **M0** Skeleton | ✅ Complete | 40 acceptance tests green. PR #441 merged. |
| **M1(2.5)** Substrate rework | ✅ Landed | PR #445. Historical rationale remains in `src/v3/M1_DESIGN.md`. |
| **M1(2.6)** Facts flow + single authority | ✅ Landed | Folded into PR #445. |
| **M1(2.7)** Enumeration-driven substrate fix | ✅ Landed | Folded into PR #445. |
| **M1(3)** First downstream consumer | ✅ Landed | End-to-end emitter path proven on PR #445. |
| **L1** Reflection framework | ✅ Complete | PR #466. |
| **L1.5** Clean bootstrap | 🟡 In progress | Authority migration and multi-target cleanup remain. |
| **Post-A/B** Lane plan | ⏸ Absorbed into R1 Release Program | See §"Release R1 Program" above — the nine-lane R1 structure supersedes the four-lane Post-A/B framing for active planning. |
| **M2** Feature parity | ⏸ Absorbed into Lane 3 Stage 3a | The remaining tail is tracked through the lane docs. |
| **M3** Self-hosting | ⏸ Absorbed into Lane 3 Stage 3c | Same cycle, clearer owner. |
| **M4** Thesis completion | ⏸ Absorbed across Lanes 1–3 | No free-floating milestone debt remains. |

## Principles

- Keep it simple. If a file gets large, something is wrong.
- Behaviors compose from `std/`; hardcoded rules mean missing modeling.
- Every decision should trace to a validation experiment or a v2 lesson.
- v2 is the reference implementation and test oracle.
- Facts flow forward from declaration source to consumer.
- Single authority: one declaration per concept.
- `ROADMAP.md` is the tracker; internal follow-up state belongs here and in the docs it links to.

## Sketch vs Oracle framing (M0–M2)

The Rust at `src/v3/compiler/` is a sketch used to validate substrate design during M0–M2; the `.dag` rewrite is the real v3 authority.

That framing still governs style decisions: refactor hand-written Rust where the structure is wrong, not because the future `.dag` version will look different.

## Architecture

```
Source text → tokenize → parse → lower → Dag (declarations + behaviors)
                                          │
                                          ├── infer (writes port state)
                                          ├── lenses read the DAG (cost, ownership, effects, ...)
                                          └── emitter translates DAG + LanguageSpec → text
```

Five L1 behaviors and six type connectives remain terminal absent a stop-signal-class substrate argument.

## M0 — Skeleton (complete)

Historical detail lives in `M0_RETROSPECTIVE.md`. The operational summary is unchanged: five behaviors survived validation and adding a sixth still requires the C1 stop signal.

## M1(2.5) — Substrate rework (shipped in PR #445)

Historical design rationale moved to `src/v3/M1_DESIGN.md`; this file only tracks the live state.

## M1(2.6) — FACTS FLOW FORWARD + SINGLE AUTHORITY (active, PR #445)

This milestone is closed; the receipts remain in the roadmap history archive.

## M1(2.7) — Enumeration-driven substrate fix (landed on PR #445)

This milestone is closed; the detailed downstream-gap receipts remain in `src/v3/DOWNSTREAM_REQUIREMENTS.md` and the roadmap history archive.

## M1(3) — What PR-B validated

The first downstream consumer path is landed. Historical receipts remain in the roadmap history archive.

## M2 — Feature parity (absorbed into Lane 3 Stage 3a)

Feature-parity work is now tracked through lane ownership instead of as a free-standing milestone.

## M3 — Self-hosting (deferred)

Self-hosting is now Lane 3 Stage 3c and later SG work, not a detached milestone bucket.

## M4 — Thesis completion (deferred)

The thesis-completion surface is fully distributed across lanes and no longer managed as a separate backlog bucket.

## Post-A/B Lane Plan

**Superseded by §"Release R1 Program" above.** The four-lane Post-A/B framing is absorbed into the nine-lane R1 structure; R1 is the active authority for forward planning. The historical receipts remain useful for context.

See [docs/history/roadmap-post-ab-lane-plan.md](docs/history/roadmap-post-ab-lane-plan.md) for the full embedded plan and [docs/post-l15-phase-plan.md](docs/post-l15-phase-plan.md) for the master dependency graph.

## Active deferrals — follow-up work from merged PRs

The full deferral ledger moved to [docs/history/roadmap-active-deferrals.md](docs/history/roadmap-active-deferrals.md). The live DB-track status lines are kept here for quick review.

- `DB-1`: diagnostics-as-corrections shipped end to end; malformed-correction production carrier remains follow-up. See [docs/db-history/db-1.md](docs/db-history/db-1.md).
- `DB-3`: user-declared dimensions core shipped; generic `.dag` lowering and example-authoring follow-ups remain. See [docs/db-history/db-3.md](docs/db-history/db-3.md).
- `DB-7`: symbolic-cost algebra shipped; typed polynomial-degree and related carrier cleanups remain follow-up. See [docs/db-history/db-7.md](docs/db-history/db-7.md).
- `DB-8`: fixed-point ratchet infrastructure landed; full self-hosting cycle remains gated on Lane 1e. See [docs/db-history/db-8.md](docs/db-history/db-8.md).
- `DB-9`: mutual-recursion lowering shipped under the R2 substrate shape. See [docs/db-history/db-9.md](docs/db-history/db-9.md).
- `DB-10`: `data` value semantics shipped; the historical trade-off receipt moved out of line. See [docs/db-history/db-10.md](docs/db-history/db-10.md).
- `DB-11`: Parameter / generic **`where`** refinement lowered (see `test_3a3_*`); out-of-fragment rejection and narrowing receipts moved out of line. Type-alias RHS `where` parsing + lowering landed in PR #703 and is covered by `test_db11_type_alias_where_*` integration receipts. See [docs/db-history/db-11.md](docs/db-history/db-11.md).
- `DB-12`: surface generics shipped as a tests-first slice. See [docs/db-history/db-12.md](docs/db-history/db-12.md).
- `DB-13`: Disj dotted-path support shipped as a tests-first slice. See [docs/db-history/db-13.md](docs/db-history/db-13.md).
- `DB-14`: substrate accessor follow-on remains open through the E-9 bootstrap rewrite. See [docs/db-history/db-14.md](docs/db-history/db-14.md).
- `DB-15`: test-infrastructure schema landed; generated runner execution remains follow-up. See [docs/db-history/db-15.md](docs/db-history/db-15.md).
- `DB-16`: refined-generic substitution and `FnExternalBody` reconciliation receipts moved out of line; equality-authority cleanup remains follow-up. See [docs/db-history/db-16.md](docs/db-history/db-16.md).
- `DB-17`: reference-resolution provenance remains the named authority for the user-range fallback class. See [docs/db-history/db-17.md](docs/db-history/db-17.md).
- `DB-18`: workflow-effect carrier and Rust reflection shipped; Go accessor proof remains a later slice. See [docs/db-history/db-18.md](docs/db-history/db-18.md).
- `DB-19`: reserved; no in-tree design doc is allocated yet. See [docs/db-history/db-19.md](docs/db-history/db-19.md) if and when receipts exist.
- `DB-20`: workflow `ParallelEffect` parallel-composition safety shipped; thesis-facing graph parallelism remains separate open work. See [docs/db-history/db-20.md](docs/db-history/db-20.md).

## Scheduled deletions — scaffolds with named dissolution triggers

The full scheduled-deletions table, notes, and enforcement rationale moved to [docs/history/roadmap-scheduled-deletions.md](docs/history/roadmap-scheduled-deletions.md).

The operational rule is unchanged: every live scaffold needs an explicit dissolution trigger and enforcement path, and deleting the scaffold removes its row.

## Tracked debts — 2026-04 analyses

Findings from two reflective analyses (integration loop health, `main@b014746` and `main@11e66b4`) plus two exploratory analyses (scope: `dsl/std/*.dag`, `src/v2/tests/src/*.rs`, `dsl/extdeps/`, `THESIS.md`, `INVARIANTS.md`, `MODELING.md`, same commits). Items below are grouped by urgency. Each line: 1-sentence fact · dissolution trigger · owner-or-next-step.

### PR #726 (quiet-eagle-364) — E-I / E-C lane ship-with-debt receipt (2026-04-24)

OpenAI-pro meta-review: **SHIP_WITH_DEBT** — substrate staging and receipt-level tests are merge-ready; marginal review value is low without the next structural slice. Remaining work is **E-P / E-M behavioral wiring**, scaffold dissolution (`promote_to_strict` rename/delete), and a **unified numeric-refinement authority** across Peano literal bridges (see existing Peano ratchet row in P4 below). Canonical checklist (owners, triggers, what already landed): [docs/debt/pr-726-ei-carrier-debt.md](docs/debt/pr-726-ei-carrier-debt.md).

### Debt classification — framing

Items in this ledger fall into three categories. The count alone is a poor health signal; the **flow** (items arriving vs. items dissolved) is the real signal.

- **`[honest-debt]`** — genuine mistakes or bugs caught by review. Small set (P0s, a few emit bugs). These deserve unambiguous blame semantics and fast dispatch.
- **`[transitional]`** — bridges with named dissolution triggers (file-preference rank, `parse_parser_body.txt`, dual v2/v3 `std/` authority). Not debt in the blame sense; pre-paid scaffold that dissolves by construction when its trigger fires.
- **`[invariant-reveal]`** — patterns flagged because the thesis sharpened after they were authored (fail-closed discipline, no string-keyed lookups, partitioned `EffectShape`, structural authority). These are **evidence the language grew**, not evidence of sloppiness. An empty `[invariant-reveal]` bucket would mean the thesis stopped evolving.

Per-row tagging is a scheduled follow-up sweep — **trigger:** post-merge of this PR, before the next receipt-closure wave lands; **owner:** ROADMAP maintainer, bundled with the "Stale-receipt sweep in `docs/briefs/`" row earlier in this section. Dominant classification by section:

| Section | Dominant class | Notes |
|---------|----------------|-------|
| P0 | `[honest-debt]` | All three are real bugs. |
| P1 (fabrication / fail-open) | `[invariant-reveal]` | Fail-closed discipline retroactive. |
| P2 (structural compression) | Mixed | Hand-rolled lattices `[invariant-reveal]`; `languages.dag` dup and `effects.dag` dual authority `[transitional]`. |
| P3 (modeling gaps) | `[invariant-reveal]` | Twenty `declaration_by_name` sites = retroactive string-lookup smell. |
| P4 (type refinement) | `[invariant-reveal]` | Cardinality and alias-refinement gaps. |
| Post-merge 2026-04-20 | `[transitional]` | `parse_parser_body.txt` dissolves with SG-2b proper. |
| Post-merge 2026-04-21 | Mixed | Class 5 Gap 1 `[invariant-reveal]`; file-preference rank `[transitional]`; emit-gap items `[honest-debt]`. |
| Lane E substrate-carrier-port | `[invariant-reveal]` | v3 substrate shape revealed missing structural carriers. |
| Compiler–std consolidation | `[transitional]` | Every migration has a named dissolution trigger. |

### P0 — real bugs (silent wrong execution)

- **render.dag repeat_string ignores `n`**: `repeat_string` folds over singleton `[0]` so any `n > 0` returns one copy of `s`. Propagates to `indent_text`. Dissolution: fold over a length-`n` list or use a `repeat` combinator. Owner: immediate dispatch.
- **REST_OPS test table drift — `CreateComment` path wrong**: test authority at `src/v2/tests/src/effects.rs:149-218` lists `POST /repos/{...}/pulls/{pull_number}/comments`; extdep authority at `dsl/extdeps/github/pulls.dag:179-195` says `/repos/{...}/issues/{issue_number}/comments`. Dissolution: test table consumes extdep operation facts directly; delete the parallel table. Owner: immediate dispatch.
- **`__BUG_NO_PROFILE_…` fabrication sentinel in `dsl/std/types.dag:115-119`**: `container_param_name_required` returns `concat("__BUG_NO_PROFILE_", kind_name)` on miss; duplicated as fallback in `src/v2/tests/src/compiler-tests.rs:2350-2354`. Violates M5 and C-8 fail-closed. Dissolution: function returns `Option`, caller handles. Owner: immediate dispatch.

### P1 — fabrication / fail-open boundaries

- **http_path.dag `None => ""` fabrications**: `dsl/std/http_path.dag:37-69` silently normalizes malformed `{...}` segments to empty strings. Dissolution: parser returns `Option` / `Result`, caller handles.
- **effects.dag reconstructs `HttpMethod` and `PathTemplate` from strings**: `dsl/std/effects.dag:214-223, 270-287` reparses already-modeled structures. Dissolution: `derive_op_effect` parameterized by the typed transport declaration, not `(method_str, path_str)`.
- **`ResourceHandle` forgeable despite opacity claim**: `dsl/std/resources.dag:18-25` documents "only compiler's acquire nodes can mint these" but carries plain-record fields (user code can construct arbitrary handles). Dissolution: witness-carrier + private constructor, or typed opaque handle per Track 9 pattern.

### P2 — structural compression (biggest ROI)

- **Four hand-rolled `BoundedLattice<T>` / `Lattice<T>` instances**: `FermiDepth` (`dsl/std/fermi.dag`), `Encoding` (`dsl/std/encoding.dag`), `DescentEvidence` (`dsl/std/termination.dag:91-129`), `SubValueRelation` (`dsl/std/induction.dag:202-281`). Each names the intended algebra in comments. Dissolution: declare each type as inhabiting the generic algebra; delete the hand-rolled meet/join pairs. Partially tracked under Tracks 8 / 9.
- **Language-fact duplication — `languages.dag` vs `dsl/extdeps/languages/*/emit.dag`**: per-target reserved-words/syntax tables exist in both; emit.dag files explicitly say "mirrors std.languages." Dissolution: Lane 1e consumes `languages.dag`, per-target emit files dissolve.
- **Triple `MethodTranslation` schema** across `dsl/extdeps/languages/{rust,python,go}/runtime.dag`: same shape; only the per-target template field-name differs. Dissolution: single generic `MethodTranslation` with a `target: LanguageId` discriminator.
- **`effects.dag` dual authority** — `dsl/std/effects.dag` (395 lines) vs `src/v3/std/effects.dag` (818 lines, diverged 2×). Dissolution: decide authoritative location, collapse the other; name a convergence lane.
- **`container_template_algebra_rows` table duplicates type aliases** — `dsl/std/types.dag:133-146` is the string-keyed table (renamed from `container_to_algebra` in PR #651's narrow dissolution); `types.dag:211-213` declares `type List<T> = FreeMonoid<T>` etc. Dissolution: derive the table from the aliases when alias reflection lands.
- **Parallel string-keyed authorities in types.dag + coercion.dag**: `kernel_primitives` (65-74), `container_arity` (79-91), `ordered_collections` (125-130), `TypeCheckpoint.dag_name: String` (38-44), `InhabitantDecl.algebra: String` (59-65). Header labels as transitional. Dissolution: derive from structural declarations.

### P3 — modeling gaps

- **`declaration_by_name(...)` pattern in emit** — 20 sites on `origin/main`, including `emit.rs`, `emit/rust_target.rs`, `emit/python_target.rs` looking up `"OrderedRing"`, `"SubstrateAccessorBinding"`, `"Dag"`, `"fold"`, `"id"`. Violates Layer Opacity / Semantic Authority (post-lowering). Dissolution: typed substrate access via cached declaration-id (`algebra_field_for_operator` pattern) at every call site.
- **`pipeline_authority.rs` parses body-span text for stage order**: Semantic Authority leak. Dissolution: typed stage list from `pipeline.dag` declarations.
- **LLM service flattening**: `dsl/extdeps/llm/llm.dag` declares `Role`, `ContentBlock`, `LlmMessage`, `StopReason`; Anthropic/OpenAI service operations still take `model: String`, `messages: Json`, and extract outputs by string path. Dissolution: service operations consume the typed carriers; outputs returned as typed responses.
- **GitHub auth model bypass**: `dsl/extdeps/github/github.dag:42-46` declares `GitHubAuthToken { token, scopes, expires_at }` but `dsl/extdeps/github/auth.dag:13-23` returns only `{ token: Secret }` and hardcodes GCP secret-manager policy. Dissolution: `github_token()` returns the full typed token; remove the hardcoded provider policy.
- **`errors.dag` dead generic layer**: `HttpErrorShape`, `AuthError`, `RateLimitError`, `ConflictError`, `ProviderError` declared at `dsl/std/errors.dag:5-30`; no non-doc consumer; extdeps use only provider-specific shapes. Dissolution: wire the generic layer or delete it.

### P4 — type refinement / modeling faithfulness

- **Fixed-width types aren't structurally fixed**: `dsl/std/bit.dag` uses nominal records with `List<Bit>` / `List<Byte>` fields — cardinality not substrate-enforced; alias-form refinements unavailable until alias `where` parses/lowers ([db-11 gap](docs/db-history/db-11.md)). Dissolution: wire type-alias refinement, field refinement, or a `Cardinality(element, Exact(n))`-style carrier.
- **Surface int-literals are concept-layer host-narrowed, not reconciliation-narrowed**: `src/v3/std/tokenize.dag:30` declares `IntLit(Int)` where `Int = Int64 = OrderedRing<Word64>` (`dsl/std/integer.dag:43`), so the lexer's digit-run → token concept is already narrowed to host `i64` at the tokenizer boundary. Surfaced 2026-04-24 by substrate lane-owner analysis: `i64::MIN` as a single-token literal is unrepresentable (principled workaround `(0 - MAX - 1)` exists under `OrderedRing` additive-inverse — pure ring arithmetic over representable operands). The class generalizes to `UInt64` upper boundary `2^64 - 1` and to silent narrowing of typed numerics in non-`Int64` contexts. Dissolution: `IntLit` carries an unbounded magnitude at the concept layer (candidate carrier: a `std.natural` / magnitude-unbounded-natural type), narrows to the target int algebra at reconciliation; negation stays `OrderedRing` additive-inverse — no unary-minus parse primary is needed (`src/v3/compiler/parse_tables.dag:413-448` documents the closed-set prefix-opener dispatch with a full 4-pattern dissolution audit, and `-x = 0 - x` desugars principally under the ring). Adjacent to the fixed-width-types row above (shared DB-11 alias-`where` + cardinality-substrate blocker family) and to the Post-R1 Grounding Completeness Program (`ROADMAP.md:149` — surface-side sibling to that program's target-side primitive-grounding claim; natural fit as a Tier-1 thesis-claim vignette once the program opens). Owner: unassigned; M scope.
- **Peano witness carriers extend the recursive-type ratchet (visible, temporary)**: `PositiveDescentAmount` / `ProportionalDivisor` in `dsl/std/termination.dag` and mirrors add self-recursive substrate types beyond the historical `Node`-centric recursive set. This is intentional for E-I proof carriers and is **ratchet-only-up** in visibility: inline `.dag` comments and this row record the bridge; dissolution tracks **E-P** (shared `PositiveInt` / ranged-`Int` refinement authority for literal bridges, collapsing Peano materialization to one refinement story across `std.termination` / `std.computation` / `std.induction`). Not a behavioral blocker for landing E-T/E-C/E-I staging — PR body should name the ratchet when touching those carriers. Owner: E-P lane; modeling receipt.

### Post-merge debt (2026-04-20 cleanup brief)

- **`src/v3/compiler/parse_parser_body.txt` — 1350 LOC hand-authored recursive-descent parse algorithm**: PR #589 retired `parse.rs` from the `.rs` census, but the algorithm moved here as a `.txt` fragment `include_str!`'d into `regen_parse` output. SG-0 now counts `.txt` scaffolds via `EXPECTED_HAND_AUTHORED_FRAGMENTS` in `sg0_census_test.rs` — the sole census authority for crate-root scaffolds. (Not added to `compiler.dag::stage0.hand_maintained_src`: that list models `source_dir` companions; its freshness/copy consumers never walk the crate root, so an entry there would be dead — extending the model to cover one crate-root file would be premature per `feedback_enumerate_before_substrate`.) SG-3f surface reflection is now landed: `src/v3/std/parse_surface.dag` (`v3.std.parse_surface`) owns the full Surface* carrier schema, Rust/Go/Python realization rows exist, parser-output mirror tests cover recursive sums/patterns/literals, and SG-3f-d consumption proof compiles/emits/links/runs against reflected surface values. Dissolution trigger is now solely structural `parse.dag` ownership via SG-2b proper. Owner: unblocked by SG-3f; queue behind SG-2b proper dispatch.

### Post-merge debt (2026-04-21 receipt-closure wave)

- **Class 5 Gap 1 — `Bool` inhabits `BooleanAlgebra<Bool>` grounding not wired**: surfaced during #610's closing. Current `OperatorKind::Logical(_)` dispatch in the three emitters is hardcoded (`if let OperatorKind::Logical(_)` branches bypass the algebra-field resolution that arithmetic operators use). The substrate already models `BooleanAlgebra<T>` with `meet/join/complement` per `dsl/std/algebra.dag:230-236`, and comments there state *"Bool inhabits BooleanAlgebra"* — but the `inhabits` edge isn't structurally landed. Dissolution: wire `Bool inhabits BooleanAlgebra<Bool>` + extend `resolve_operator_arrow` for `OperatorKind::Logical(_)` + delete the hardcoded emitter branches. Owner: 1e-2b lane (Path A).
- **Class 5 Gap 3 — `data` body shape boundary (substrate capability gap)**: `ValueBody` in `src/v3/compiler/src/dag.rs:259-287` carries three variants — `Unparsed(SourceSpan)`, `Structural { fields: Vec<(String, FieldValue)> }`, `Scalar(LiteralBits)`. **Field-level** shapes inside `Structural` are FULL per `FieldValue` at `src/v3/compiler/src/dag.rs:328-353` (`Literal`, `Reference`, `Record`, `List`, `Variant { constructor, payload }`) — nested records, list literals, declaration references, `Var` references, sum-variant literals all lower today via `lower_structural_field_value` at `src/v3/compiler/src/lower.rs:2616+`. **Remaining gap is at the top-level `ValueBody` boundary**: `data foo: List<T> = [...]`, `data foo: SumType = SomeVariant`, and `data foo: T = other_decl_ref` parse to real `SurfaceExpr` shapes but cannot lower structurally today — only scalar literals (→ `ValueBody::Scalar`) and record-structural bodies (→ `ValueBody::Structural`) are accepted; anything else falls to `ValueBody::Unparsed` and `reject_user_unparsed_scaffolds` rejects it for user-range declarations. The comment at `dag.rs:269-287` is explicit: growing `Scalar` to swallow `Reference` / `List` / `Variant` is rejected; extension is via new `ValueBody` variants. **Authority caveat:** `src/v3/DOWNSTREAM_REQUIREMENTS.md:239` describes the *pre-PR-B-unwind* shape (where `FieldValue` was `LiteralBits`-only and "port-carried field values" were the gap); the code has since extended `FieldValue` with the four additional variants, so the DOWNSTREAM entry's "remaining gap" framing is itself stale against the live code. Read `dag.rs` / `lower.rs` for the current boundary. **Consumer impact confirmed 2026-04-24** by Surface Manager sub-child `quiet-gull-882`: `sub_charclass_in_std_unicode` phase-2 (`tokenize.dag` retype to `Char` / `List<Char>` / `CharClass`) is not blocked by parser list syntax; the minimal fixture `data xs: List<Int> = [1, 2]` reaches the same R14 `ValueBody::Unparsed` hard-fail path. Target shape for the tokenizer lane is a fully lowered constant such as `data ascii_scan_order: List<CharClass> = [Whitespace, Digit, IdentStart, IdentContinue]`, which requires a top-level list/sum-value carrier (for example a new `ValueBody::List(Vec<FieldValue>)`, with unit sum constructors represented via existing `FieldValue::Variant`). A separate load-set prerequisite also surfaced: `Dag::new()`'s default std fixture set in `bootstrap.rs::std_fixtures()` does not include `dsl/std/unicode.dag`, so wiring `tokenize.dag` to `std.unicode::CharClass` needs a bootstrap/load-set decision before type checking can resolve `CharClass`. Dissolution direction: extend `ValueBody` with the additional top-level variants needed by downstream consumers, and decide how `std.unicode` enters the runtime bootstrap authority set. Alternative design call: shift authority shape for config declarations from `data` bodies to extractor-over-unparsed-source. Owner: unassigned; substrate-capability scope — routes through a self-hosting / substrate-capability lane, not through the consuming lanes. **Audit pattern — recursive application.** The 2026-04-23 Character-level row claimed "no substrate capability gap" and was factually wrong; retracted 2026-04-24 by the update to that row. The 2026-04-24 initial authoring of *this* row then described the remaining gap incorrectly (claimed field-level shapes were blocked when they were actually supported, citing the stale DOWNSTREAM entry as authority); corrected later the same day after independent audit caught the drift. Both incidents instantiate the rule: **claims about substrate capability must be verified against live `ValueBody` / `FieldValue` / `lower_*` code paths before the claim lands — reading a summary authority (ROADMAP row, DOWNSTREAM entry) is necessary but not sufficient, because authority text itself drifts.**
- **`emit_rust_module` gap: SurfaceLiteral → LiteralBits rename**: substrate declares `LitInt/LitBool/LitString` in `LiteralBits`; Rust enum is `Int/Bool/String`; no rename facility in `emit_rust_module` causes connective-producing slices (per #612's staging choice notes) to fail on this. Dissolution: add per-variant name mapping in spec or rename-facility in emit. Owner: unassigned; blocks deeper SG-3g slices.
- **`emit_rust_module` gap: `render_variant_constructor` fails on external tuple variants**: `src/v3/compiler/src/emit/rust_target.rs:4296` unconditionally emits `Variant { _0: value }` which is wrong for external tuple variants. Dissolution: spec-driven variant-constructor template per source kind. Owner: unassigned; blocks deeper SG-3g slices.
- **File-preference rank is a ratified-parallel-authority scaffold**: `Dag::declaration_name_preference_rank` (`src/v3/compiler/src/dag.rs:1985-2015`) and its mirror in `collect_symbols` (`src/v3/compiler/src/lower.rs:1258-1278`) resolve same-named declarations by preferring `src/v3/` over `dsl/`. This is a P2 Boundary Discipline violation — a stable lookup rule that *normalizes* duplicate authority instead of dissolving it. Held as a scaffold rather than deleted because the duplicates carry v3-only substrate content (`v3.std.substrate` imports, `ElementRef` / `PortId` references, partitioned `EffectShape`, diagnostic-assertion surface) that cannot yet live under `dsl/std/` — v2 CI recursively ingests `dsl/` and cannot parse v3 grammar. Dissolution trigger: when every module currently duplicated between `dsl/std/` and `src/v3/std/` (or `src/v3/spec/`) has converged to a single canonical home, delete the rank function + mirror policy and remove the duplicate-authority preference rule entirely. After convergence, `declaration_by_name` must resolve against the single surviving authority or fail closed on multiple matches; no rank-based bridge remains. Convergence checklist: (1) `module std.effects` — `dsl/std/effects.dag` ↔ `src/v3/std/effects.dag` (v3 copy imports `v3.std.substrate`, partitions `EffectShape`, uses `ElementRef<OperationEffect>`); (2) `module std.verification` — `dsl/std/verification.dag` ↔ `src/v3/std/verification.dag` (surfaces are disjoint today: v2 `AssertKind/TestClaim/TestCase` vs v3 `DiagnosticKind / DiagnosticReference / PortStateExpectation`; convergence requires a design call on which surface wins); (3) embedded `http_path` mirror inside `src/v3/std/effects.dag:118-260` (resolves via either staging `http_path.dag` in v3 or rewiring imports to consume `std.http_path` directly once v3-substrate staging lands). Owner: unblocked by v3.std.substrate staging lane + v3-grammar CI convergence.

### Post-merge debt (2026-04-21 deferred-from-wave)

Tracked debt not dispatched with the 2026-04-21 lane wave (Lane 1e P3.0, SG-2c-4, SG-4b-3, cascade cleanup, algebra inhabitance quartet; **SG-3g-b is closed** — see `docs/history/sg-3g-b-lower-helpers-wire-in.md`). Opportunistic slots — no blocking dependencies, can be picked up whenever bandwidth frees.

- **SG-2c growth-discipline checkpoint (2026-04-22)**: SG-2c-1…5 have landed; the **parse_primary** cluster (SG-2c-6 + SG-2c-7) is closed. Decision doc [docs/briefs/sg-2c-proper-capability-gap.md](docs/briefs/sg-2c-proper-capability-gap.md) names the SG-2c-proper capability blocker concretely (recursive `.dag` function bodies over `List<Token>` with cursor threading + `Token` variant match + `Result<(Surface*, Int), Diagnostic>` short-circuit — load-bearing sub-gap is list-body emission per `src/v3/std/list.dag:13-15`); the **next** open question is the pivot-vs-tiny-extractions trade (see that doc) — do not pre-queue a long tail of one-row `parse_tables` extractions; land capability work for SG-2c proper in parallel. Parallel-authority risk from the accumulated `parse_tables` row families remains judged modest-not-accelerating (exempted under the std/-consolidation `parse_tables.dag` precedent-rule bullet).

- **`container_template_algebra_rows` string table is duplicate authority**: `dsl/std/types.dag:133-146` declares the renamed map (PR #651 collapsed both consumers onto a single `container_template_algebra` query); rows still duplicate the authoritative type aliases at `:211-213` (`type List<element> = FreeMonoid<element>`, etc.) across Pascal/snake spellings plus algebra self-identities. Dissolution trigger: when `.dag` gains alias reflection, the rows derive from the aliases directly and the hand-maintained table goes away. P2 Boundary Discipline / P1 receipt. Owner: unassigned; small scope.
- **Emitter render-helper consolidation (Phase 3.1 candidate after 1e P3.0)**: `named_variant_id` / `render_named_template` / `primitive_type_id_for_port` / `walk_to_disj` / `algebra_field_for_operator` exist 3-5× across `src/v3/compiler/src/emit.rs` + `emit/python_target.rs` + `emit/rust_target.rs` (plus `walk_to_disj_decl` also in `lower.rs` and `infer.rs`). Collapse into one emitter-shared module. Different scope from 1e P3.0 which targets `behavior_result_port` / `port_is_consumed_from`; this is the render-side tranche. P2 Boundary Discipline receipt. Owner: queued behind 1e P3.0 as the natural follow-up.
- **Stale §-style and line-number cross-references to pre-rewrite `INVARIANTS.md` — PARTIAL (T-Receipts W1).** The ledger-named refs in `docs/design-db16-refined-generic-substitution.md`, `docs/perf/clone-elimination.md`, `docs/lane2-compile-time-proofs.md`, `src/v3/M1_DESIGN.md`, `src/v3/SELF_HOSTING.md`, `docs/history/sg-4b-1-fix-declaration-lookup-cleanup.md`, and `docs/briefs/p0-bug-no-profile-sentinel.md` now use stable `INVARIANTS.md#<id>` / principle anchors instead of line-number or pre-rewrite § references. Broader stale INVARIANTS prose outside that named set remains a follow-up sweep; this receipt only closes the listed references.
- **`Strict Forward Progress` subdoc ↔ heading drift**: pre-existing inconsistency between `docs/invariants/strict-forward-progress.md` (bounded-execution content) and the reviewer-usage meaning of the SFP rule name (dissolution-progress). Flagged explicitly in both P4 and P5 of INVARIANTS.md post-#620 as future cleanup. Dissolution: either rename the subdoc to reflect its bounded-execution content (and author a new dissolution-progress subdoc), or split the subdoc into two. Small scope once the design call is made on naming. Owner: unassigned.
- **v3 lens capability honesty pass (supersedes the complexity-receipt lane as originally scoped)**: the 2026-04-21 audit (see [docs/v3-lens-capability-register.md](docs/v3-lens-capability-register.md)) found that `src/v3/lenses/complexity.dag` does not subsume v2's `complexity.dag` — v2 produces symbolic `CostExpr` / `SizeExpr` with work/span/certainty/asymptotic-class, v3 produces a single integer depth per port. The same pattern holds for `cost.dag` (PROXY — no named size variables, Dimension wiring deferred), `idempotency.dag` (STUB — Rust oracle), and `parallelism.dag` (STUB — fail-closed placeholder). The originally-claimed "~13× LOC reduction with structurally stronger fail-closed guarantees" was comparing different functions of the program; retracted. **Follow-up work:** (1) the cementing-test discipline generalizes from one lens to every `regen.dag` entry (semantic-equality goldens per lens); (2) `docs/briefs/complexity-v2-v3-comparison-receipt.md` to be rewritten against the register — scope widens from "complexity receipt + registry rename" to "Band C honesty pass" once the register is merged. Substrate dissolution path: port `DescentEvidence` / `CallPattern` / `SubValueRelation` into the v3 substrate, wire per-call producers, and keep the E-M receipt that `MethodSemantics` is structurally subsumed rather than ported. Emit dissolution path: `match` lowering over user-defined sums (currently blocks `.dag` authority for idempotency + parallelism). Both are programs of work, not single lanes. Owner: unassigned; brief rewrite queues for after the 2026-04-21 wave's 5 lanes land and this register is in tree.
- **Substrate carrier port program (scopes the "substrate dissolution path" half of the honesty pass above)**: design doc at [docs/design-substrate-carrier-port-program.md](docs/design-substrate-carrier-port-program.md) (2026-04-22). Decomposes the `DescentEvidence` / `CallPattern` / `SubValueRelation` / per-call producer work, with the `MethodSemantics` lane now **closed via M-b structural subsumption**. T/C/I port the carrier *types*; **E-P** scopes the per-call descent-evidence producer (v2 stores this on `ExprCall.descent_evidence`; v3 has no analogous attachment on `TransformNode`) and is where carrier parity actually closes for the live call-site slice. **E-M receipt:** v3 has no `ExprMethodCall` side slot; v2 `PlainMethodSemantics`, `AlgebraMethodSemantics`, and `ServiceMethodSemantics` map to `TransformTarget::{Callable, FieldProject, Operator}` plus typed declaration/effect metadata, so no ported `MethodSemantics` carrier or Rust mirror is added. Related: per-method metadata surface from `*_templates()` in `dsl/std/algebra.dag` (surfaced by Lane G / PR #654) — scoped in §6a of the design doc as a deferred follow-up, not part of the core four-carrier port. Closes **method carrier-parity** for `cost.dag` + `complexity.dag`; full `BEHAVIORALLY COMPLETE` for `cost.dag` additionally requires E-P and the register's separate non-carrier blockers (`Dimension<SymbolicCost>` grammar/data-body wiring, named `SizeVariable` value semantics) that are outside this program, and opens P4 Decidability progress. Does NOT touch the emit-gap half (idempotency / parallelism); that stays in the receipt-closure wave. Owner: unassigned; remaining carrier-program lane is E-P plus the separately tracked non-carrier blockers.
  - **Lane E-T — port `DescentEvidence` + proof structure (S)**: port carriers + lattice fns into v3-reachable `std/termination.dag`. No deps. Acceptance: parse/lower/emit green; port-progress receipt recorded in the design doc (not in the lens capability register, which is lens-only).
  - **Lane E-C — port `CallPattern` + lowering (S)**: port `SizeBound` / `CallPattern` / `ShrinkFactor` / `IterationPrimitive` / `LoweringTarget` / `IterationDimension` + `lower_call_pattern`. Requires E-T. Acceptance: `cost.dag` begins PROXY-partial progress.
  - **Lane E-I — port `SubValueRelation` + inductive fields + cost algebra (M)**: port Tier 1 (structural) + Tier 2 (lattice) + Tier 3 (`CostBound` + master theorem). Requires E-T, E-C. Pre-flight: verify v3 lowers `CostBound`'s self-referential-through-`List` shape. Acceptance: carrier types ported with round-trip tests; no lens row moves yet (producer still missing — see E-P).
  - **Lane E-P — per-call descent-evidence provenance (M)**: decide attachment shape (on-substrate vs lens-derived vs side-table) and land the producer that v3 lacks today. v2 stores this on `ExprCall.descent_evidence` (`src/v2/00_core.dag:199`); v3's `TransformNode` (`src/v3/std/substrate.dag:264-270`) has no analogue. Requires E-T, E-C, E-I. Acceptance: v3 call produces `SubValueRelation` per-call readable by a lens; v2-oracle-vs-v3 cementing test against `expr_call_descent_evidence`; carrier parity for `cost.dag` / `complexity.dag` on the non-method-dispatch slice closes here.
  - **Lane E-M — `MethodSemantics` port-or-subsume (S–M)**: **closed via M-b structural subsumption.** Transitive facts (`CollectionSizeEffect` / `CostShape` / `AlgebraFieldTemplate`) remain algebra-declaration facts; v3 method-like calls dispatch through `TransformTarget::{Callable, FieldProject, Operator}` and typed declaration/effect metadata. No `src/v3/std/method_semantics.dag`, no Rust mirror, and no `ExprMethodCall` attachment point.
- **CI ratchet architecture — PARTIAL (T-Receipts W1 second bundle)**: surfaced during the 2026-04-21 reflective analysis (`53b3110..ae8825a` range). Commits `37cd6128`, `4898983e`, `f84ed355` hardened the ratchet against CI-log instability, but `2d8396df` widened the exemption list. `scripts/slow-test-exemptions.txt` currently has 43 active non-comment exemptions (83 total file lines in the original brief included comments). **Landed partial:** `scripts/check-test-timeout.sh` now has a meta-ratchet floor of 43 active exemptions, and [docs/debt/ci-ratchet-exemption-audit-2026-04-24.md](docs/debt/ci-ratchet-exemption-audit-2026-04-24.md) classifies every current exemption from the existing annotated reasons. **Still open:** fresh CI-shaped timing audit, stale exemption deletion if any entries measure under 2s, and per-exempt duration budgets once fresh timings exist. Dispatch brief: `docs/briefs/ci-ratchet-architecture-audit.md`.
- **`IntegrationRsScan` / `integration_rs_active_line_contains` in `src/v3/compiler/tests/integration/common/mod.rs` is a latent source of byte-constant workarounds in downstream tests** (T-Receipts, surfaced by T-Demo #686). The scanner is "deliberately narrow" and does not model Rust character literals, so any test author touching a file in its scan path cannot write `b'\\'` / `b'"'` directly and must substitute numeric byte constants (e.g., `92` / `34`) instead. Historical evidence: #686's text-slicing helper introduced `DAG_ESCAPE_BYTE` / `DAG_QUOTE_BYTE` constants for exactly this reason; #705 deleted the helper when `TestRunner::run_suite` obsoleted it, and the constants went with it — so there is **no surviving in-tree instance of the workaround today**. The scanner constraint itself remains, which means the next test author landing in the scan path will recreate the pattern. Dissolution: either widen the scanner to model Rust char literals, or replace its scan with a structural reader that doesn't need to exclude char-literal syntax. Small scope; cosmetic for now, but the constraint is a workaround attractor. Owner: unassigned; P2 Boundary Discipline / T-Receipts bundle.
- **Stale-receipt sweep in `docs/briefs/` — PARTIAL (T-Receipts W1).** The cited `DeclarationLookup` cleanup brief now carries a historical-receipt banner stating the lane is closed by SG-4b-1-fix and that `DeclarationLookup` / `find_declaration` are gone from `src/v3/lenses/variant_payload.dag`; its stale INVARIANTS refs were also updated to stable anchors. Remaining broad `docs/briefs/` sweep stays open for later post-wave stale claims. **Include:** `docs/design-pure-bootstrap.md`'s hand-maintained count ("78 hand-maintained .rs files") was stale vs. the SG-0 census — **CLOSED in this R1 PR**: all five occurrences (lines 7, 49, 82, 105, 242 of the design doc) now point at the live census, reframed as deltas against the illustrative 78-file baseline, or scoped to the non-test + TESTING residual framing. No frozen absolute count remains.
- **Compiler–`std/` consolidation program — specific migrations**: end-state defined in [docs/thesis/compiler-std-consolidation.md](docs/thesis/compiler-std-consolidation.md). Each of the migrations below is a standalone lane; together they collapse the compiler-specific type surface toward the positive definition (pipeline + regen + lens-specific return-type carriers + accessor). **Ratchet:** count of `type` declarations in `src/v3/compiler/*.dag` AND `src/v3/lenses/*.dag` that are NOT in the positive-def set AND NOT exempted → 0. Positive-def: pipeline/regen types + lens-API return carriers (`Origin`, `UnusedParameter`, `VariantPayloadShapeLookup` 3-variant, `TemplateArgumentBinding` semantic carrier, etc.). Exempted: `parse_tables.dag` (7 types, pending SG-2c-proper per-row classification). In-ratchet: **strict 2-variant Missing|Found Lookup-pattern carriers still authored in lenses (1: `TemplateArgumentLookup` — `complexity.dag` and `cost.dag` both import `v3.std.lookup::Lookup` instead of local `CostLookup` / `SymbolicCostLookup`)** + **workaround-shaped infer-helper coproducts with named dissolution triggers (`TemplateArgumentsMatch`, `TemplateArgumentCursor`, `NormalizedInstantiationArgs`)**. Primary count: 4 (1 + 3). See thesis doc table for per-file disposition.
  - **`tokenize.dag` → `std/tokenize.dag`** — landed: `Token`, `TokenKind`, `KeywordTokenKind`, `PunctTokenKind`, `LocalPunctSpec`, `StringEscapeSpec` now live in `src/v3/std/tokenize.dag`; compiler-local duplicates deleted. Ratchet drop: 25 → 19.
  - **`runtime_mirrors.dag` → `src/v3/std/parse_surface.dag` (tranche 2)** — landed: `DagDifference`, `Surface*`, and related carriers (~14 types) now live in `v3.std.parse_surface`; `runtime_mirrors.dag` deleted. Ratchet drop: 19 → 5. Parse-rule/fragment dissolution triggers (`parse_parser_body.txt`, SG-2b/SG-3f) unchanged.
  - **`parse_tables.dag`** — decide per-type whether `BinaryOpLevel` / `BinaryOpRow` / `TopLevelItemKwRow` / `SoftKeywordIdentRow` / `BracketRow` / `PrimaryPrefixRow` / `PrimaryAtomRow` stay compiler-API (dispatch-row shapes) or move to `std/syntax.dag` (language-level precedence/dispatch facts). Owner: whoever picks up **SG-2c proper** parser cutover; migration-gate: SG-2c proper completion (the `.dag` parser reveals which rows it dispatches on vs which are pure data). Precedent rule: if any individual row needs to move before SG-2c proper lands, the mover-lane sets the classification and the rest follow the same rule when touched.
  - **`src/v3/std/*.dag` → `dsl/std/*.dag`** — the whole v3-specific std tree collapses when the file-preference-scaffold dissolves (gated on v2 retirement or `dsl/std/` learning v3 grammar). Largest single consolidation; tracked separately in the **"File-preference rank is a ratified-parallel-authority scaffold"** row (earlier in the 2026-04-21 receipt-closure wave post-merge-debt section).
  - **`Node` → `std/node.dag`** — already captured in `project_node_to_std` memory as prior pattern. Relevant to this program as a precedent.
  - **Generic `Lookup<T>` in `std/` — dissolve per-lens `SymbolicCostLookup` / `TemplateArgumentLookup` duplicates** — **landed for `complexity.dag` and `cost.dag`:** `src/v3/std/lookup.dag` + `import v3.std.lookup` + generated `lens_cost_generated.rs` / `lens_cost_symbolic_generated.rs`. SymbolicCost-monomorphized constructors live in `src/v3/std/algebra.dag` (keeps `std.lookup` free of algebra imports). **Remaining:** `infer_helpers.dag` still declares its own 2-variant `TemplateArgumentLookup`. Scope is strict 2-variant shape only — carriers with additional semantic variants (e.g., `VariantPayloadShapeLookup`'s `NotPayloadProduct`, `TemplateArgumentBinding`'s `Conflict | NoOp | Append`) stay lens-API. Owner: unassigned; migration-gate: **in progress** for the one remaining lens-local carrier.
  - **`LanguageSpec` dual authority (`dsl/std/languages.dag` vs `src/v3/std/emit_model.dag`)** — thesis-incompatible parallel authority for the "everything the emitter needs for a target language" carrier. `dsl/std/languages.dag:438` defines `type LanguageSpec` and `:1244` instantiates `data rust_spec: LanguageSpec`. `src/v3/std/emit_model.dag:276` defines a **second, differently-shaped** `type LanguageSpec`, and `src/v3/spec/rust.dag:1186` instantiates `data rust_language: LanguageSpec` against it. The Rust emitter at `src/v3/compiler/src/emit/rust_target.rs:705-707` calls `.rust_language_spec()`, which resolves to the v3-local carrier — bypassing the shared-std authority the consolidation program is built around. Direct contradiction of the compiler–std/ consolidation thesis (P2 single-authority). Dissolution is a carrier-merge design call, not a rename: the two shapes are not trivially compatible (v3 has `statements` / `expressions` / `control_flow` / `literals` / `modules` / ... fields; shared-std `languages.dag` has a different decomposition). Decide canonical shape, migrate emitter consumption, delete loser. Owner: unassigned; migration-gate: reconciliation design + emitter migration. Surfaced by 2026-04-23 meta-review (flagged as highest-value novel finding).
  - **Operator ontology — three co-existing authorities** — partially tracked via the `parse_tables.dag` migration row above and the Class 5 Gap 1 Bool-inhabits-`BooleanAlgebra` row in the 2026-04-21 post-merge-debt wave, but not called out as one unified debt item. Three surfaces: (1) `dsl/std/syntax.dag:19,80,107` — shared-std `BinOp` / `AlgebraFieldKind` / `OperatorSpec`; (2) `src/v3/std/substrate.dag:133-160` — compiler-local `ArithmeticOp` / `ComparisonOp` / `LogicalOp` / `OperatorKind` (distinct ontology); (3) `src/v3/compiler/operators.dag:5-72` — symbol↔`OperatorKind`↔`algebra_field_name` mapping. Cross-validation against `dag_operators` (in `dsl/extdeps/languages/dag/syntax.dag`) happens via string-match in `src/v3/compiler/src/regen_parse_tables_emit.rs` (`:20-33, :284-291`), not structural identity. P2 Boundary Discipline. Dissolution: decide whether `OperatorKind` promotes to shared-std (v3 ontology wins) or dissolves into `BinOp` + algebra-field projection (shared-std wins); migrate consumers off the loser; delete. Owner: unassigned; migration-gate: coordinates with SG-2c proper `parse_tables.dag` decision — each row's per-type classification depends on which operator ontology survives. Surfaced by 2026-04-23 meta-review.

- **Target emit-spec fabrication sentinels — `__EMIT_BUG_{0}__`** (**rescoped** after Lane F investigation, PR #657): originally framed as "converge three target specs on native compile-time error forms." Investigation (see `docs/briefs/emit-bug-sentinel-convergence-2026-04-22.md`) found all four sentinels (including Rust's `compile_error!`) are **already fail-closed in their target's native sense** — all cause target-compile failure. The legibility concern is real but not a P3 violation at the template layer. **Real dissolution is upstream** at `src/v2/05_emit.dag:996, 1046` where the emitter chooses to emit sentinel output when it detects an error condition (`n_is_type_var || n_is_error`, or anonymous-coproduct branch) — it should produce a Diagnostic and halt emission instead. **Dissolution trigger**: when `05_emit.dag:996, 1046` are refactored to produce Diagnostic + halt, `error_type_template` becomes unreachable; the four template declarations are then deleted. No defensive-fallback carve-out — per INVARIANTS P5 Progress Is Dissolution, unreachable scaffold is removed, not retained "just in case." Surfaced by 2026-04-22 exploratory analysis; rescoped by Lane F. Owner: unassigned (F-rescope brief drafted); M scope.

- **`algebra.dag` declaration-vs-template surface asymmetry** (**reclassified** after Lane G investigation, PR #654): originally framed as "parallel authority, derive templates from declaration." Investigation found the two surfaces are **not redundant** — `*_templates()` carries per-method metadata (`size_effect`, `cost_shape`, `callback_element_position`) consumed by complexity/cost lenses; the type declaration has no field-level slot for this metadata. Attaching it would require substrate extensions (field-level annotations, currently partial per DB-11). **Real dissolution is a modeling question**: where should per-method metadata live structurally? Three options scoped in the Lane E substrate-carrier-port design doc (options 1/2/3 per the coordination note). **Dissolution trigger**: when Lane E's design lands a chosen option for per-method metadata carriers, `*_templates()` is replaced by the chosen structural form, and the declaration/template surface converges to one. Related to but separate from the four named substrate carriers. Surfaced by 2026-04-22 exploratory analysis; reclassified by Lane G. Owner: blocked on Lane E design doc; L scope once unblocked (substrate work).

- **`extdeps/browser.dag` typed-carrier service-boundary collapse**: `dsl/extdeps/browser.dag:21-32` declares typed carriers (`BrowserConfig`, opaque `BrowserContext` / `Page` / `Element` handles, imports `Url`), but service ops use raw `String`: `Launch.input.headless: String = "false"`, `output.context_id: String`; `Goto.input.url: String`, `wait_until: String`, `output.final_url: String`; selector/query/evaluate outputs all remain `String` (`:42-47,56-60,75-103`). Same class as the LLM service flattening and the GitHub auth model bypass (both tracked separately). P1 Modeling Faithfulness / M8 "dispatch structural, not string extraction". **Dissolution trigger**: when the transport layer (REST/shell/file) can consume typed carriers end-to-end rather than stringly — this requires transport-level capability support that currently forces the String fallback. The Lane H brief (PR #658) pilots the pattern on browser.dag specifically; if that lands cleanly, LLM + GitHub dissolutions can consume the same pattern. Until then, the transport-capability gap is the upstream blocker. Surfaced by 2026-04-22 exploratory analysis. Owner: unassigned (pilot in-flight on Lane H #658); M per service family, contingent on transport capability.

- **Character-level under-consumption in tokenize + syntax authorities (mixed: consumption gap for steps 1+3, substrate gap for step 2 — see Status block below)**: `src/v3/compiler/tokenize.dag` and `dsl/extdeps/languages/dag/syntax.dag` both slice the ASCII/Unicode codepoint space in two parallel non-canonical forms. **Reserved individual codepoints as opaque strings**: `StringEscapeSpec.suffix`, `output_codepoint: Int`, `LocalPunctSpec.pattern`, `string_literal_delimiter`, `line_comment_prefix`, `OperatorSpec.symbol`, `dag_keyword_set` keys — all encode specific codepoints as byte strings. **Reserved codepoint classes as hidden Rust predicates**: `is_ascii_whitespace` / `is_ascii_digit` / `is_ascii_alphabetic` / `is_ascii_alphanumeric` plus a bare `byte == b'_'` `push_str`'d into `regen_tokenize.rs:700,727,750,752`. Not mentioned in `.dag` at all. **The character-level concepts already exist in `dsl/std/`** and are imported cross-tree today (e.g., `src/v3/compiler/regen.dag:3` imports `std.types`): `std.types::Char = Int` (Unicode scalar, U+0000–U+10FFFF), `std.string_type::String = FreeMonoid<Char>`, `std.unicode` (`DisplayWidth`, `UnicodeBlock`, block/width classification), `std.encoding` (`ASCII | UTF8 | Latin1 | Text | Binary | Unknown` lattice with `ASCII <: UTF8 <: Text` — literally the ASCII→Unicode causal chain), `std.bit::Byte`. **Framing**: phase-1 is a consumption gap — the modeling is done, and the tokenizer/syntax authorities were not using it. Phase-2 is substrate/load-set work: top-level structural `data` bodies need list/sum carriers, and `std.unicode` must enter the bootstrap/load set before `tokenize.dag` can type-check rows against `CharClass`. **Dissolution (follow-up lane)**: (1) add `CharClass = Whitespace | Digit | IdentStart | IdentContinue` (or superset) to `std.unicode` as a sibling to `DisplayWidth`, plus classification predicates as data/functions; (2) retype the opaque-string fields in `tokenize.dag` (`suffix: Char`, `output_codepoint: Char`, `pattern: List<Char>`, `string_literal_delimiter: Char`, `line_comment_prefix: List<Char>`) and the parallel fields in `syntax.dag` (`OperatorSpec.symbol: List<Char>`, keyword-set keys as `List<Char>`); (3) rewire `regen_tokenize` to read the class-predicate list structurally rather than hardcoding host-stdlib method names. Scaffold note lives in the `tokenize.dag` header. **Status (2026-04-24, post-PR #693 + quiet-gull-882 triage):** phase-1 landed — step (1) done (`CharClass` + `char_in_class` in `dsl/std/unicode.dag`, ASCII-aligned); step (3) partially done via a `tokenize_char_class.rs` Rust mirror (gate-locked 0..=127 against Rust ASCII helpers per `sub_charclass_in_std_unicode_gate`), so generated tokenizer carries no `is_ascii_*` calls. Phase-2 (step 2 — the `.dag`-native retype to `Char` / `List<Char>` / `CharClass` variants inside `data` bodies) is confirmed blocked on Class 5 Gap 3's top-level `ValueBody` boundary plus the `std.unicode` bootstrap/load-set decision. See the Class 5 Gap 3 row above for the substrate-capability dissolution direction and authority pointers. See the `char_in_class` interpreter-parity follow-up row below for the bridge-finish step (retiring the host-stdlib-only ratchet on `tokenize_char_class.rs`). Owner: phase-1 landed (PR #693); phase-2 routes to substrate/self-hosting capability work, not a T-Sub-only surface fix. If the `tokenize.dag → std/tokenize.dag` consolidation migration (from the compiler–std consolidation program above) is dispatched before Class 5 Gap 3 closes, the phase-1 Rust mirror piggybacks on it; the full structural `.dag`-native path lands only when Class 5 Gap 3 closes.
- **`char_in_class` interpreter parity (tokenizer bridge finish, PR #693):** `dsl/std/unicode.dag::char_in_class` is canonical; `src/v3/compiler/src/tokenize_char_class.rs` stays hand-synced until M1(2.8) class-5 gap #3 allows structural scanner rows. Today unit tests ratchet against `u8::is_ascii_*` only. **Next step:** once `char_in_class` is evaluable from the v3 compiler test harness, assert code-point parity on `0..=127` against `byte_matches` and retire the host-stdlib-only ratchet / duplicate sync prose. Same migration gate as deleting `tokenize_char_class.rs` when `regen_tokenize` reads classes from lowered `tokenize.dag`.

### Post-merge debt (2026-04-23 thesis-doc surface — PR #672)

Two `[target]` items surfaced by `docs/thesis/compositional-modeling.md` (Part 4) that did not have their own ledger rows at authoring time. Per the doc-authority single-ledger rule ([`docs/thesis/doc-authority.md`](docs/thesis/doc-authority.md)), gaps point to one follow-up artifact — this ledger row. Both rows land here so the story doc's `[target]` claims have a single-authority gap pointer.

- **Unit-mismatch enforcement for typed value wrappers with phantom Unit / Currency parameters**: The thesis target (per `docs/thesis/compositional-modeling.md` Part 4) is types like `Duration<Unit>` and `Money<Currency>` where a phantom parameter distinguishes `Second` from `Millisecond` or `USD` from `EUR`, operations preserve the parameter, and `Duration<Second> + Duration<Millisecond>` is a compile error without explicit conversion. The live tree does not yet support this shape. The closest named type, `Dimension<Carrier>` at `src/v3/std/dimensions.dag:61`, is a **one-parameter proof-dimension framework** (`name` / `witness_of` / `compose` / `identity` / `break_diagnostic`) used for behavioral analysis — not a typed value wrapper, and not a phantom-parameter value type. Rust's `uom` crate provides the library-level equivalent at runtime via phantom associated types. **Dissolution trigger**: (1) substrate support for typed value wrappers with phantom parameters that propagate through arithmetic; (2) a mechanism for such wrappers to inhabit an algebra (abelian group with compare, no multiplication) so operations are defined structurally. Adjacent to DB-18 (user-defined parametric algebra attachment). Surfaced by `docs/thesis/compositional-modeling.md` Part 4. P1 Modeling Faithfulness. Owner: unassigned; M scope.
- **`Secret<T>` nominal-wrapper graduation**: `dsl/std/types.dag:237` currently declares `Secret = String`, a type alias. The thesis claim (per `docs/thesis/compositional-modeling.md` Part 4) is that `Secret<T>` should be a nominal opaque wrapper with construction restricted to `std.secrets::acquire` (no `Show` instance, no `String` coercion, no `Debug` derivation). Alias form cannot carry those restrictions; the substrate distinction between nominal-opaque and alias types is the structural delta. **Dissolution trigger**: substrate support for nominal-opaque types with construction-restriction modeling (a `where only X may construct` shape), plus migration of the existing `Secret` alias to the nominal form. Adjacent to DB-11 (alias-RHS `where` parsing, earlier in this section) and to the compiler-std consolidation program. Surfaced by `docs/thesis/compositional-modeling.md` Part 4. P2 Boundary Discipline (construction restriction is a single-authority concern). Owner: unassigned; M scope.

### Reviewer-noise class — a practice, not a debt

- **Integration-reflection cadence**: every ~few days, run a reflective + exploratory analysis pair. The 2026-04-15 and 2026-04-18 passes caught items individual PR reviews missed (authority split across PRs, silent cross-PR name-based lookups, the `CreateComment` drift, the `repeat_string` bug). Worth institutionalizing.

## What NOT to build yet

- Any fourth per-language emit file before Stage 1e consolidation finishes.
- Advanced diagnostics beyond the shipped correction surfaces.
- Async or concurrent emission strategies before the lane plan closes the earlier authority work.

## Open design questions

- Bound source tracking for structural descent evidence.
- Closure-context rules across `Bind` into `Loop`.
- Carrier refinement for Tier-2 safety proofs.
- Effect composition details across sequential and branched execution.
- Lens storage and materialization once more of the compiler self-hosts.
