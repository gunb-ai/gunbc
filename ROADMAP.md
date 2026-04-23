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

- **(a) compiles as a `.dag` declaration** — achievable from Day 1; DB-15's TestClaim schema is already in tree.
- **(b) evaluates true at release** — requires T-TestGen's runner closure. T-TestGen is therefore the gate-enabling lane, not just one lane among many.

The release gate IS a `.dag` program. This is the thesis eating its own dogfood.

**Debt paydown continues in parallel.** R1 does not freeze the tracked-debt ledger. The lane structure names forward deliverables; the ledger below keeps dispatching — CI ratchet audit, stale-brief sweep, INVARIANTS cross-ref cleanup, scheduled-deletion work. Treat debt work as a continuous **T-Receipts** track (bundle 2-4 items per PR per the standing preference).

### Goals (the six non-negotiables)

1. **Complexity lens at v2 parity or beyond — no debt.** Structured `CostExpr(work, span, asymptotic_class)` from structure. Lane T-LaneE.
2. **Test generation + integration + service simulation — first-class.** Tests are `.dag` data. Lane T-TestGen.
3. **Multi-target emission.** Rust production-grade; Python/Go demonstrably working. Lane T-Emit.
4. **Impossible-bugs demo suite.** Enumerated bug classes with compile-time proofs (see THESIS `Enumerable impossible-bug classes`). Lane T-Demo.
5. **Arbitrary lens composition.** User-authored lenses. Lane T-LensAPI.
6. **Self-hosting (Pure Bootstrap).** Zero hand-authored compiler files; generated escape hatch acceptable. Lanes T-PB-A + T-PB-B.

Enablers (prerequisites for the goals): T-P0 (bug sweep), T-Sub (surface syntax completion).

### Nine lanes

Each lane owns one concrete `.dag` gate. Lane owners do the comprehensive decomposition; this section holds intent and acceptance only.

| Lane | Size | Covers | Cross-ref into debt ledger |
|------|------|--------|----------------------------|
| T-P0 | S | P0 sweep (repeat_string, REST_OPS, no_profile_sentinel) | §P0 — real bugs |
| T-Sub | S | `match` over user sums, `CharClass` in std.unicode, type-alias `where` | §P4 (bit.dag refinements), Character-level under-consumption |
| T-Emit | M | Rust harden, #650 generic-bound fidelity, Python/Go reconcile | SurfaceLiteral→LiteralBits, variant-constructor template |
| T-LaneE | XL | Complexity lens v2 parity via substrate-carrier-port | Existing Lane E-T/C/I/P/M program |
| T-TestGen | L | Testgen runner, service simulation, first-class TestClaim | DB-15 follow-up |
| T-LensAPI | M-L | User-authored lenses + composition | Lens capability honesty pass |
| T-PB-A | XL | Compiler self-emits (fixed-point); `EXPECTED_HAND_AUTHORED` 95 → 0 non-test (generated escape hatch OK) | Compiler–std consolidation program, hand-Rust census |
| T-PB-B | M | Tests-as-data (no hand-Rust tests) | DB-15 + T-TestGen |
| T-Demo | M | Two canonical fixtures + impossible-bugs suite + narrative | — (new) |

### Lane acceptance — `.dag` gates

Full `TestClaim` declarations live in the lane briefs. Each predicate below is either in today's DB-15 schema or scheduled for T-TestGen extension.

- **T-P0.** `p0_repeat_string_correct · p0_no_fabrication_sentinel · p0_rest_ops_aligned`
- **T-Sub.** `sub_match_over_user_sum · sub_type_alias_where_lowers · sub_charclass_in_std_unicode`
- **T-Emit.** `emit_rust_fixtures_rustc_green · emit_generic_bounds_survive · emit_omni_demo_fixtures_green`
- **T-LaneE.** `complexity_merge_sort_is_nlogn · complexity_v3_matches_v2_oracle`
- **T-TestGen.** `testgen_structural_coverage · testgen_mock_backed_integration_safe · testgen_manual_claim_is_first_class`
- **T-LensAPI.** `user_authored_lens_compiles · lens_composition_associative · lens_output_is_queryable_data`
- **T-PB-A.** `pb_zero_hand_authored_nontest · pb_self_compile_fixed_point`
- **T-PB-B.** `pb_test_file_generated_from_dag`
- **T-Demo.** `fixture_compiler_nerd_canonical · fixture_integration_canonical · impossible_bug_class_suite`

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

- T-LaneE and T-PB-A are XL and independent — run fully parallel from W0.
- T-LensAPI is decoupled, starts W0.
- T-TestGen is the serial hinge for T-PB-B.

### Relationship to existing milestone status

R1 absorbs what was "Post-A/B Lane Plan" and L1.5 forward work, framed by release deliverable rather than architectural stage. The status table below stays accurate for backward-looking context; R1 is the forward-looking companion.

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
| **Post-A/B** Lane plan | 🟡 Planned / active | Four lanes own all remaining thesis obligations. |
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

The four-lane plan remains the project’s active structure for the remaining thesis work.

See [docs/history/roadmap-post-ab-lane-plan.md](docs/history/roadmap-post-ab-lane-plan.md) for the full embedded plan and [docs/post-l15-phase-plan.md](docs/post-l15-phase-plan.md) for the master dependency graph.

## Active deferrals — follow-up work from merged PRs

The full deferral ledger moved to [docs/history/roadmap-active-deferrals.md](docs/history/roadmap-active-deferrals.md). The live DB-track status lines are kept here for quick review.

- `DB-1`: diagnostics-as-corrections shipped end to end; malformed-correction production carrier remains follow-up. See [docs/db-history/db-1.md](docs/db-history/db-1.md).
- `DB-3`: user-declared dimensions core shipped; generic `.dag` lowering and example-authoring follow-ups remain. See [docs/db-history/db-3.md](docs/db-history/db-3.md).
- `DB-7`: symbolic-cost algebra shipped; typed polynomial-degree and related carrier cleanups remain follow-up. See [docs/db-history/db-7.md](docs/db-history/db-7.md).
- `DB-8`: fixed-point ratchet infrastructure landed; full self-hosting cycle remains gated on Lane 1e. See [docs/db-history/db-8.md](docs/db-history/db-8.md).
- `DB-9`: mutual-recursion lowering shipped under the R2 substrate shape. See [docs/db-history/db-9.md](docs/db-history/db-9.md).
- `DB-10`: `data` value semantics shipped; the historical trade-off receipt moved out of line. See [docs/db-history/db-10.md](docs/db-history/db-10.md).
- `DB-11`: Parameter / generic **`where`** refinement lowered (see `test_3a3_*`); out-of-fragment rejection and narrowing receipts moved out of line. **`type X = … where …` on type aliases is not closed:** the handwritten parser still skips the alias RHS clause (`parse_type_rhs_after_eq` → `skip_where_clause`, `src/v3/compiler/src/parse.rs`). Std surfaces must not treat alias refinements as enforced until that gap lands (see **Type-alias `where` parsing gap** in [docs/db-history/db-11.md](docs/db-history/db-11.md)).
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

### Debt classification — framing

Items in this ledger fall into three categories. The count alone is a poor health signal; the **flow** (items arriving vs. items dissolved) is the real signal.

- **`[honest-debt]`** — genuine mistakes or bugs caught by review. Small set (P0s, a few emit bugs). These deserve unambiguous blame semantics and fast dispatch.
- **`[transitional]`** — bridges with named dissolution triggers (file-preference rank, `parse_parser_body.txt`, dual v2/v3 `std/` authority). Not debt in the blame sense; pre-paid scaffold that dissolves by construction when its trigger fires.
- **`[invariant-reveal]`** — patterns flagged because the thesis sharpened after they were authored (fail-closed discipline, no string-keyed lookups, partitioned `EffectShape`, structural authority). These are **evidence the language grew**, not evidence of sloppiness. An empty `[invariant-reveal]` bucket would mean the thesis stopped evolving.

Per-row tagging is a follow-up sweep. Dominant classification by section:

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
- **`algebra.dag:267-323` signature/comment mismatch**: comments promise `index: (FreeMonoid<T>, Nat) -> T?`, `map: (FreeMonoid<T>, fn T→U) -> FreeMonoid<U>`, `fold: (FreeMonoid<T>, U, fn U,T→U) -> U`, `merge: (Map<K,V>, Map<K,V>, fn V,V→V) -> Map<K,V>`; actual declarations drop the source/second-argument types. Dissolution: reconcile declarations with comments (comments likely correct; declarations drop the polymorphism).

### Post-merge debt (2026-04-20 cleanup brief)

- **`src/v3/compiler/parse_parser_body.txt` — 1350 LOC hand-authored recursive-descent parse algorithm**: PR #589 retired `parse.rs` from the `.rs` census, but the algorithm moved here as a `.txt` fragment `include_str!`'d into `regen_parse` output. SG-0 now counts `.txt` scaffolds via `EXPECTED_HAND_AUTHORED_FRAGMENTS` in `sg0_census_test.rs` — the sole census authority for crate-root scaffolds. (Not added to `compiler.dag::stage0.hand_maintained_src`: that list models `source_dir` companions; its freshness/copy consumers never walk the crate root, so an entry there would be dead — extending the model to cover one crate-root file would be premature per `feedback_enumerate_before_substrate`.) SG-3f surface reflection is now landed: `src/v3/std/parse_surface.dag` (`v3.std.parse_surface`) owns the full Surface* carrier schema, Rust/Go/Python realization rows exist, parser-output mirror tests cover recursive sums/patterns/literals, and SG-3f-d consumption proof compiles/emits/links/runs against reflected surface values. Dissolution trigger is now solely structural `parse.dag` ownership via SG-2b proper. Owner: unblocked by SG-3f; queue behind SG-2b proper dispatch.

### Post-merge debt (2026-04-21 receipt-closure wave)

- **Class 5 Gap 1 — `Bool` inhabits `BooleanAlgebra<Bool>` grounding not wired**: surfaced during #610's closing. Current `OperatorKind::Logical(_)` dispatch in the three emitters is hardcoded (`if let OperatorKind::Logical(_)` branches bypass the algebra-field resolution that arithmetic operators use). The substrate already models `BooleanAlgebra<T>` with `meet/join/complement` per `dsl/std/algebra.dag:230-236`, and comments there state *"Bool inhabits BooleanAlgebra"* — but the `inhabits` edge isn't structurally landed. Dissolution: wire `Bool inhabits BooleanAlgebra<Bool>` + extend `resolve_operator_arrow` for `OperatorKind::Logical(_)` + delete the hardcoded emitter branches. Owner: 1e-2b lane (Path A).
- **`emit_rust_module` gap: SurfaceLiteral → LiteralBits rename**: substrate declares `LitInt/LitBool/LitString` in `LiteralBits`; Rust enum is `Int/Bool/String`; no rename facility in `emit_rust_module` causes connective-producing slices (per #612's staging choice notes) to fail on this. Dissolution: add per-variant name mapping in spec or rename-facility in emit. Owner: unassigned; blocks deeper SG-3g slices.
- **`emit_rust_module` gap: `render_variant_constructor` fails on external tuple variants**: `src/v3/compiler/src/emit/rust_target.rs:4296` unconditionally emits `Variant { _0: value }` which is wrong for external tuple variants. Dissolution: spec-driven variant-constructor template per source kind. Owner: unassigned; blocks deeper SG-3g slices.
- **File-preference rank is a ratified-parallel-authority scaffold**: `Dag::declaration_name_preference_rank` (`src/v3/compiler/src/dag.rs:1985-2015`) and its mirror in `collect_symbols` (`src/v3/compiler/src/lower.rs:1258-1278`) resolve same-named declarations by preferring `src/v3/` over `dsl/`. This is a P2 Boundary Discipline violation — a stable lookup rule that *normalizes* duplicate authority instead of dissolving it. Held as a scaffold rather than deleted because the duplicates carry v3-only substrate content (`v3.std.substrate` imports, `ElementRef` / `PortId` references, partitioned `EffectShape`, diagnostic-assertion surface) that cannot yet live under `dsl/std/` — v2 CI recursively ingests `dsl/` and cannot parse v3 grammar. Dissolution trigger: when every module currently duplicated between `dsl/std/` and `src/v3/std/` (or `src/v3/spec/`) has converged to a single canonical home, delete the rank function + mirror policy and remove the duplicate-authority preference rule entirely. After convergence, `declaration_by_name` must resolve against the single surviving authority or fail closed on multiple matches; no rank-based bridge remains. Convergence checklist: (1) `module std.effects` — `dsl/std/effects.dag` ↔ `src/v3/std/effects.dag` (v3 copy imports `v3.std.substrate`, partitions `EffectShape`, uses `ElementRef<OperationEffect>`); (2) `module std.verification` — `dsl/std/verification.dag` ↔ `src/v3/std/verification.dag` (surfaces are disjoint today: v2 `AssertKind/TestClaim/TestCase` vs v3 `DiagnosticKind / DiagnosticReference / PortStateExpectation`; convergence requires a design call on which surface wins); (3) embedded `http_path` mirror inside `src/v3/std/effects.dag:118-260` (resolves via either staging `http_path.dag` in v3 or rewiring imports to consume `std.http_path` directly once v3-substrate staging lands). Owner: unblocked by v3.std.substrate staging lane + v3-grammar CI convergence.
- **`DeclarationLookup` parallel authority in `src/v3/lenses/variant_payload.dag` — RESOLVED (SG-4b-1-fix).** Originally a fail-open `find_declaration(decls, target) -> DeclarationLookup = LookupMissing | LookupFound(Declaration)` path duplicated `Dag::declaration(id)` and converted Track-9 constructor-validation failures into normal lens-consumer outcomes (C-8 violation). The SG-4b-1-fix lane dissolved this: the file now imports `declaration_by_id` from `std.substrate` (the substrate accessor) and exports `VariantPayloadShapeLookup = DeclarationMissing | NotPayloadProduct | Found(VariantPayloadShape)` as its 3-variant lens-API carrier. `find_declaration` and `DeclarationLookup` are gone. This row kept as a historical cross-reference; candidate for the 2026-04-21 stale-receipt sweep (move to `docs/history/` or delete).

### Post-merge debt (2026-04-21 deferred-from-wave)

Tracked debt not dispatched with the 2026-04-21 lane wave (Lane 1e P3.0, SG-2c-4, SG-4b-3, cascade cleanup, algebra inhabitance quartet; **SG-3g-b is closed** — see `docs/briefs/sg-3g-b-lower-helpers-wire-in.md`). Opportunistic slots — no blocking dependencies, can be picked up whenever bandwidth frees.

- **SG-2c growth-discipline checkpoint (2026-04-22)**: SG-2c-1…5 have landed; the **parse_primary** cluster (SG-2c-6 + SG-2c-7) is closed. Decision doc [docs/briefs/sg-2c-proper-capability-gap.md](docs/briefs/sg-2c-proper-capability-gap.md) names the SG-2c-proper capability blocker concretely (recursive `.dag` function bodies over `List<Token>` with cursor threading + `Token` variant match + `Result<(Surface*, Int), Diagnostic>` short-circuit — load-bearing sub-gap is list-body emission per `src/v3/std/list.dag:13-15`); the **next** open question is the pivot-vs-tiny-extractions trade (see that doc) — do not pre-queue a long tail of one-row `parse_tables` extractions; land capability work for SG-2c proper in parallel. Parallel-authority risk from the accumulated `parse_tables` row families remains judged modest-not-accelerating (exempted under the std/-consolidation `parse_tables.dag` precedent-rule bullet).

- **`algebra.dag` signature/comment reconciliation**: `dsl/std/algebra.dag:265-302` — `FreeMonoid<T>`'s comments promise `index(FreeMonoid<T>, Nat) -> T?`, `map(FreeMonoid<T>, fn(T)->U) -> FreeMonoid<U>`, `fold(FreeMonoid<T>, init: U, fn(U,T)->U) -> U`; the declarations drop the carrier/source parameters (`index: fn(Int) -> T`, `map: fn(fn(T)->T) -> FreeMonoid<T>`, `fold: fn(T, fn(T,T)->T) -> T`). `PartialFunction<K,V>` at `:307-325` has the same issue — comment promises `merge(Map<K,V>, Map<K,V>, fn(V,V)->V) -> Map<K,V>`, declaration is `merge: fn(PartialFunction<K,V>) -> PartialFunction<K,V>` (missing fn argument). One of the two (comments or declarations) is wrong; reconcile. P1 Modeling Faithfulness receipt. Owner: unassigned; small scope.
- **`container_template_algebra_rows` string table is duplicate authority**: `dsl/std/types.dag:133-146` declares the renamed map (PR #651 collapsed both consumers onto a single `container_template_algebra` query); rows still duplicate the authoritative type aliases at `:211-213` (`type List<element> = FreeMonoid<element>`, etc.) across Pascal/snake spellings plus algebra self-identities. Dissolution trigger: when `.dag` gains alias reflection, the rows derive from the aliases directly and the hand-maintained table goes away. P2 Boundary Discipline / P1 receipt. Owner: unassigned; small scope.
- **`pipeline_authority.rs` structural read — RESOLVED (#637).** `ordered_pipeline_stages()` in `src/v3/compiler/src/pipeline_authority.rs` is now the authority function: it reads `PipelineStageBinding` declarations structurally from the Dag, and its docstring states "the declaration order of the bindings in `pipeline.dag` is the ordering authority." `pipeline_compile_order_names()` at `:185` is a thin wrapper around `ordered_pipeline_stages(dag).map(.stage_name)` — no text slicing. The text-slicing lives in `compile_body_stage_names_from_source()` at `:130-180`, called only from `reconcile_with_compile_body()` at `:110-128` as a **drift-check fail-closed reconciliation** — if the text-sliced compile-body stage order disagrees with the structural authority, compilation fails. The drift check is a legitimate belt-and-suspenders pattern, not a P2 violation. This row kept as a historical cross-reference; candidate for the stale-receipt sweep. Earlier review characterization of "substrate-capability blocker for ordered-body reflection" was incorrect — the authority moved to `PipelineStageBinding` declaration order, so no capability gap remains here.
- **Emitter render-helper consolidation (Phase 3.1 candidate after 1e P3.0)**: `named_variant_id` / `render_named_template` / `primitive_type_id_for_port` / `walk_to_disj` / `algebra_field_for_operator` exist 3-5× across `src/v3/compiler/src/emit.rs` + `emit/python_target.rs` + `emit/rust_target.rs` (plus `walk_to_disj_decl` also in `lower.rs` and `infer.rs`). Collapse into one emitter-shared module. Different scope from 1e P3.0 which targets `behavior_result_port` / `port_is_consumed_from`; this is the render-side tranche. P2 Boundary Discipline receipt. Owner: queued behind 1e P3.0 as the natural follow-up.
- **Stale §-style and line-number cross-references to pre-rewrite `INVARIANTS.md`**: Pro's post-#620 observation. Line-number refs in `docs/design-db16-refined-generic-substitution.md:297,298,413`, `docs/perf/clone-elimination.md:101`, `docs/lane2-compile-time-proofs.md:182`; §-style heading-name refs in `src/v3/M1_DESIGN.md` (§"No short-term solutions" / §"No bridges"), `src/v3/SELF_HOSTING.md` (§"Recursive syntax is sugar"), `docs/briefs/sg-4b-1-fix-declaration-lookup-cleanup.md` (§C-8 / §Track 9), `docs/briefs/p0-bug-no-profile-sentinel.md` (§"No fabrication sentinels"). Grep navigation still works because the strings survive as bullets in the new INVARIANTS.md, but line numbers have shifted and §-as-section framing is now approximate. Dissolution: sweep replacing line refs with `INVARIANTS.md#<id>` anchors (post-#620) or name-based refs. Cosmetic, low urgency. Owner: unassigned.
- **`Strict Forward Progress` subdoc ↔ heading drift**: pre-existing inconsistency between `docs/invariants/strict-forward-progress.md` (bounded-execution content) and the reviewer-usage meaning of the SFP rule name (dissolution-progress). Flagged explicitly in both P4 and P5 of INVARIANTS.md post-#620 as future cleanup. Dissolution: either rename the subdoc to reflect its bounded-execution content (and author a new dissolution-progress subdoc), or split the subdoc into two. Small scope once the design call is made on naming. Owner: unassigned.
- **"Return first match" cascade cleanup** (dispatched in the 2026-04-21 wave currently in flight; listed here only to cross-reference): `src/v3/compiler/src/dag.rs:1999` and `:2023`, and ROADMAP:169 row, all inherit the "return the first match or fail-closed on multiple" phrasing from #625. Correct dissolution target is fail-closed-only. Owner: 2026-04-21 wave (cascade cleanup micro-lane).
- **v3 lens capability honesty pass (supersedes the complexity-receipt lane as originally scoped)**: the 2026-04-21 audit (see [docs/v3-lens-capability-register.md](docs/v3-lens-capability-register.md)) found that `src/v3/lenses/complexity.dag` does not subsume v2's `complexity.dag` — v2 produces symbolic `CostExpr` / `SizeExpr` with work/span/certainty/asymptotic-class, v3 produces a single integer depth per port. The same pattern holds for `cost.dag` (PROXY — no named size variables, Dimension wiring deferred), `idempotency.dag` (STUB — Rust oracle), and `parallelism.dag` (STUB — fail-closed placeholder). The originally-claimed "~13× LOC reduction with structurally stronger fail-closed guarantees" was comparing different functions of the program; retracted. **Follow-up work:** (1) the cementing-test discipline generalizes from one lens to every `regen.dag` entry (semantic-equality goldens per lens); (2) `docs/briefs/complexity-v2-v3-comparison-receipt.md` to be rewritten against the register — scope widens from "complexity receipt + registry rename" to "Band C honesty pass" once the register is merged. Substrate dissolution path: port `DescentEvidence` / `CallPattern` / `SubValueRelation` / `MethodSemantics` into the v3 substrate (currently blocks genuine equivalence for complexity + cost). Emit dissolution path: `match` lowering over user-defined sums (currently blocks `.dag` authority for idempotency + parallelism). Both are programs of work, not single lanes. Owner: unassigned; brief rewrite queues for after the 2026-04-21 wave's 5 lanes land and this register is in tree.
- **Substrate carrier port program (scopes the "substrate dissolution path" half of the honesty pass above)**: design doc at [docs/design-substrate-carrier-port-program.md](docs/design-substrate-carrier-port-program.md) (2026-04-22). Decomposes the `DescentEvidence` / `CallPattern` / `SubValueRelation` / `MethodSemantics` port into five ordered lanes (T → C → I → P → M), with per-carrier shape analysis, dependency ordering, and per-lane stop-signals. T/C/I port the carrier *types*; **E-P** scopes the per-call descent-evidence producer (v2 stores this on `ExprCall.descent_evidence`; v3 has no analogous attachment on `TransformNode`) and is where carrier parity actually closes for the non-method-dispatch slice. M is a design-decision lane gated on T/C/I/P evidence. Related: per-method metadata surface from `*_templates()` in `dsl/std/algebra.dag` (surfaced by Lane G / PR #654) — scoped in §6a of the design doc as a deferred follow-up, not part of the core four-carrier port. Closes **carrier-parity** for `cost.dag` + `complexity.dag` (their drop-lists lose the four carrier entries); full `BEHAVIORALLY COMPLETE` for `cost.dag` additionally requires the register's separate non-carrier blockers (`Dimension<SymbolicCost>` grammar/data-body wiring, named `SizeVariable` value semantics) that are outside this program, and opens P4 Decidability progress. Does NOT touch the emit-gap half (idempotency / parallelism); that stays in the receipt-closure wave. Owner: unassigned; lanes dispatchable in the order T → C → I → P → M.
  - **Lane E-T — port `DescentEvidence` + proof structure (S)**: port carriers + lattice fns into v3-reachable `std/termination.dag`. No deps. Acceptance: parse/lower/emit green; port-progress receipt recorded in the design doc (not in the lens capability register, which is lens-only).
  - **Lane E-C — port `CallPattern` + lowering (S)**: port `SizeBound` / `CallPattern` / `ShrinkFactor` / `IterationPrimitive` / `LoweringTarget` / `IterationDimension` + `lower_call_pattern`. Requires E-T. Acceptance: `cost.dag` begins PROXY-partial progress.
  - **Lane E-I — port `SubValueRelation` + inductive fields + cost algebra (M)**: port Tier 1 (structural) + Tier 2 (lattice) + Tier 3 (`CostBound` + master theorem). Requires E-T, E-C. Pre-flight: verify v3 lowers `CostBound`'s self-referential-through-`List` shape. Acceptance: carrier types ported with round-trip tests; no lens row moves yet (producer still missing — see E-P).
  - **Lane E-P — per-call descent-evidence provenance (M)**: decide attachment shape (on-substrate vs lens-derived vs side-table) and land the producer that v3 lacks today. v2 stores this on `ExprCall.descent_evidence` (`src/v2/00_core.dag:199`); v3's `TransformNode` (`src/v3/std/substrate.dag:264-270`) has no analogue. Requires E-T, E-C, E-I. Acceptance: v3 call produces `SubValueRelation` per-call readable by a lens; v2-oracle-vs-v3 cementing test against `expr_call_descent_evidence`; carrier parity for `cost.dag` / `complexity.dag` on the non-method-dispatch slice closes here.
  - **Lane E-M — `MethodSemantics` port-or-subsume (S–M)**: transitive carriers (`CollectionSizeEffect` / `CostShape` / `AlgebraFieldTemplate`) are already in `dsl/std/algebra.dag:408-430`, so M-a is a routine port of ~4 carriers (not a promotion-out-of-compiler-internal). M-b is structural subsumption via v3's `TransformTarget` model. Call is "does v3 structural resolution already carry these facts?" — queue after E-T/C/I land.
- **CI ratchet architecture — exemption widening erodes the per-test timeout ratchet**: surfaced during the 2026-04-21 reflective analysis (`53b3110..ae8825a` range). Commits `37cd6128`, `4898983e`, `f84ed355` hardened the ratchet against CI-log instability, but `2d8396df` widened the exemption list. `scripts/slow-test-exemptions.txt` is 83 lines with no monotonic-shrink rule — `feedback_ratchet_only_down` violated at the meta level (the primary ratchet can drift upward through exemption growth). Dissolution: audit the exemption list, categorize each (stale / paydown / structural), delete stale entries, add a meta-ratchet that fails CI on exemption-count growth or per-exempt budget drift. Dispatch brief: `docs/briefs/ci-ratchet-architecture-audit.md`. Owner: unassigned; queue as infrastructure-principal lane, soon.
- **Stale-receipt sweep in `docs/briefs/`**: surfaced during the 2026-04-21 reflective analysis. Briefs under `docs/briefs/` and prose references in adjacent docs still point at debt that the final code has partially or fully dissolved — the `DeclarationLookup` cleanup is the cited example where brief prose names debt the code has largely resolved. Integration drag, not code regression. Dissolution: sweep `docs/briefs/` for superseded claims post-wave, convert closed briefs to historical receipts under `docs/history/` or add a "Closed by #NNN" line, update inline prose in sibling docs that cite the resolved debt. Small scope, cosmetic. Owner: unassigned; do soon after the 2026-04-21 wave lands (most of its outputs will trigger more brief-staleness). **Include:** `docs/design-pure-bootstrap.md`'s hand-maintained count ("78 hand-maintained .rs files") is stale — live `EXPECTED_HAND_AUTHORED` in the SG-0 census has 89+ entries under `src/v3/compiler/`. Either refresh the number or delete it and point at the live census; detected during #642 review.
- **Compiler–`std/` consolidation program — specific migrations**: end-state defined in [docs/thesis/compiler-std-consolidation.md](docs/thesis/compiler-std-consolidation.md). Each of the migrations below is a standalone lane; together they collapse the compiler-specific type surface toward the positive definition (pipeline + regen + lens-specific return-type carriers + accessor). **Ratchet:** count of `type` declarations in `src/v3/compiler/*.dag` AND `src/v3/lenses/*.dag` that are NOT in the positive-def set AND NOT exempted → 0. Positive-def: pipeline/regen types + lens-API return carriers (`Origin`, `UnusedParameter`, `VariantPayloadShapeLookup` 3-variant, `TemplateArgumentBinding` semantic carrier, etc.). Exempted: `parse_tables.dag` (7 types, pending SG-2c-proper per-row classification). In-ratchet: **strict 2-variant Missing|Found Lookup-pattern carriers across lenses (3 today: `CostLookup`, `SymbolicCostLookup`, `TemplateArgumentLookup`)** + **workaround-shaped infer-helper coproducts with named dissolution triggers (`TemplateArgumentsMatch`, `TemplateArgumentCursor`)**. Baseline: 5. See thesis doc table for per-file disposition.
  - **`tokenize.dag` → `std/tokenize.dag`** — landed: `Token`, `TokenKind`, `KeywordTokenKind`, `PunctTokenKind`, `LocalPunctSpec`, `StringEscapeSpec` now live in `src/v3/std/tokenize.dag`; compiler-local duplicates deleted. Ratchet drop: 25 → 19.
  - **`runtime_mirrors.dag` → `src/v3/std/parse_surface.dag` (tranche 2)** — landed: `DagDifference`, `Surface*`, and related carriers (~14 types) now live in `v3.std.parse_surface`; `runtime_mirrors.dag` deleted. Ratchet drop: 19 → 5. Parse-rule/fragment dissolution triggers (`parse_parser_body.txt`, SG-2b/SG-3f) unchanged.
  - **`parse_tables.dag`** — decide per-type whether `BinaryOpLevel` / `BinaryOpRow` / `TopLevelItemKwRow` / `SoftKeywordIdentRow` / `BracketRow` / `PrimaryPrefixRow` / `PrimaryAtomRow` stay compiler-API (dispatch-row shapes) or move to `std/syntax.dag` (language-level precedence/dispatch facts). Owner: whoever picks up **SG-2c proper** parser cutover; migration-gate: SG-2c proper completion (the `.dag` parser reveals which rows it dispatches on vs which are pure data). Precedent rule: if any individual row needs to move before SG-2c proper lands, the mover-lane sets the classification and the rest follow the same rule when touched.
  - **`src/v3/std/*.dag` → `dsl/std/*.dag`** — the whole v3-specific std tree collapses when the file-preference-scaffold dissolves (gated on v2 retirement or `dsl/std/` learning v3 grammar). Largest single consolidation; tracked separately in the **"File-preference rank is a ratified-parallel-authority scaffold"** row (earlier in the 2026-04-21 receipt-closure wave post-merge-debt section).
  - **`Node` → `std/node.dag`** — already captured in `project_node_to_std` memory as prior pattern. Relevant to this program as a precedent.
  - **Generic `Lookup<T>` in `std/` — dissolve per-lens `CostLookup` / `SymbolicCostLookup` / `TemplateArgumentLookup` duplicates** — three lenses currently declare their own 2-variant `Missing | Found(T)` carrier. Collapse to a single `std/Lookup<T>` generic consumed by every lens that returns a possibly-missing result. 3 types dissolve into 1. Scope is strict 2-variant shape only — carriers with additional semantic variants (e.g., `VariantPayloadShapeLookup`'s `NotPayloadProduct`, `TemplateArgumentBinding`'s `Conflict | NoOp | Append`) stay lens-API. Owner: unassigned; migration-gate: **ready-to-dispatch** (no capability gap — pure type-declaration work; generics already work per the earlier ParseStep discussion).

- **Target emit-spec fabrication sentinels — `__EMIT_BUG_{0}__`** (**rescoped** after Lane F investigation, PR #657): originally framed as "converge three target specs on native compile-time error forms." Investigation (see `docs/briefs/emit-bug-sentinel-convergence-2026-04-22.md`) found all four sentinels (including Rust's `compile_error!`) are **already fail-closed in their target's native sense** — all cause target-compile failure. The legibility concern is real but not a P3 violation at the template layer. **Real dissolution is upstream** at `src/v2/05_emit.dag:996, 1046` where the emitter chooses to emit sentinel output when it detects an error condition (`n_is_type_var || n_is_error`, or anonymous-coproduct branch) — it should produce a Diagnostic and halt emission instead. **Dissolution trigger**: when `05_emit.dag:996, 1046` are refactored to produce Diagnostic + halt, `error_type_template` becomes unreachable and the four template declarations can dissolve or remain as defensive fallback. Surfaced by 2026-04-22 exploratory analysis; rescoped by Lane F. Owner: unassigned (F-rescope brief drafted); M scope.

- **`algebra.dag` declaration-vs-template surface asymmetry** (**reclassified** after Lane G investigation, PR #654): originally framed as "parallel authority, derive templates from declaration." Investigation found the two surfaces are **not redundant** — `*_templates()` carries per-method metadata (`size_effect`, `cost_shape`, `callback_element_position`) consumed by complexity/cost lenses; the type declaration has no field-level slot for this metadata. Attaching it would require substrate extensions (field-level annotations, currently partial per DB-11). **Real dissolution is a modeling question**: where should per-method metadata live structurally? Three options scoped in the Lane E substrate-carrier-port design doc (options 1/2/3 per the coordination note). **Dissolution trigger**: when Lane E's design lands a chosen option for per-method metadata carriers, `*_templates()` is replaced by the chosen structural form, and the declaration/template surface converges to one. Related to but separate from the four named substrate carriers. Surfaced by 2026-04-22 exploratory analysis; reclassified by Lane G. Owner: blocked on Lane E design doc; L scope once unblocked (substrate work).

- **`extdeps/browser.dag` typed-carrier service-boundary collapse**: `dsl/extdeps/browser.dag:21-32` declares typed carriers (`BrowserConfig`, opaque `BrowserContext` / `Page` / `Element` handles, imports `Url`), but service ops use raw `String`: `Launch.input.headless: String = "false"`, `output.context_id: String`; `Goto.input.url: String`, `wait_until: String`, `output.final_url: String`; selector/query/evaluate outputs all remain `String` (`:42-47,56-60,75-103`). Same class as the LLM service flattening and the GitHub auth model bypass (both tracked separately). P1 Modeling Faithfulness / M8 "dispatch structural, not string extraction". **Dissolution trigger**: when the transport layer (REST/shell/file) can consume typed carriers end-to-end rather than stringly — this requires transport-level capability support that currently forces the String fallback. The Lane H brief (PR #658) pilots the pattern on browser.dag specifically; if that lands cleanly, LLM + GitHub dissolutions can consume the same pattern. Until then, the transport-capability gap is the upstream blocker. Surfaced by 2026-04-22 exploratory analysis. Owner: unassigned (pilot in-flight on Lane H #658); M per service family, contingent on transport capability.

- **Character-level under-consumption in tokenize + syntax authorities (consumption gap, not substrate gap)**: `src/v3/compiler/tokenize.dag` and `dsl/extdeps/languages/dag/syntax.dag` both slice the ASCII/Unicode codepoint space in two parallel non-canonical forms. **Reserved individual codepoints as opaque strings**: `StringEscapeSpec.suffix`, `output_codepoint: Int`, `LocalPunctSpec.pattern`, `string_literal_delimiter`, `line_comment_prefix`, `OperatorSpec.symbol`, `dag_keyword_set` keys — all encode specific codepoints as byte strings. **Reserved codepoint classes as hidden Rust predicates**: `is_ascii_whitespace` / `is_ascii_digit` / `is_ascii_alphabetic` / `is_ascii_alphanumeric` plus a bare `byte == b'_'` `push_str`'d into `regen_tokenize.rs:700,727,750,752`. Not mentioned in `.dag` at all. **The character-level concepts already exist in `dsl/std/`** and are imported cross-tree today (e.g., `src/v3/compiler/regen.dag:3` imports `std.types`): `std.types::Char = Int` (Unicode scalar, U+0000–U+10FFFF), `std.string_type::String = FreeMonoid<Char>`, `std.unicode` (`DisplayWidth`, `UnicodeBlock`, block/width classification), `std.encoding` (`ASCII | UTF8 | Latin1 | Text | Binary | Unknown` lattice with `ASCII <: UTF8 <: Text` — literally the ASCII→Unicode causal chain), `std.bit::Byte`. **Framing**: this is a consumption gap — the modeling is done, the tokenizer and syntax authorities just aren't using it. Only one substrate delta is needed. **Dissolution (follow-up lane)**: (1) add `CharClass = Whitespace | Digit | IdentStart | IdentContinue` (or superset) to `std.unicode` as a sibling to `DisplayWidth`, plus classification predicates as data/functions; (2) retype the opaque-string fields in `tokenize.dag` (`suffix: Char`, `output_codepoint: Char`, `pattern: List<Char>`, `string_literal_delimiter: Char`, `line_comment_prefix: List<Char>`) and the parallel fields in `syntax.dag` (`OperatorSpec.symbol: List<Char>`, keyword-set keys as `List<Char>`); (3) rewire `regen_tokenize` to read the class-predicate list structurally rather than hardcoding host-stdlib method names. Scaffold note lives in the `tokenize.dag` header. Owner: unassigned; migration-gate: **ready-to-dispatch** (no substrate capability gap). If the `tokenize.dag → std/tokenize.dag` consolidation migration (from the compiler–std consolidation program above) is dispatched first, the rewrite piggybacks on it so the structurally-labeled borrow lands in its final home.

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
