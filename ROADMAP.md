# gunbc Roadmap

Single source of truth for project status, active work, and deferred items. Long-form receipts and historical narratives now live under `docs/history/` and `docs/db-history/` so this file can stay operational.

> Design spec: [docs/v3-spec.md](docs/v3-spec.md)
> Validation: [docs/v3-validation-experiments.md](docs/v3-validation-experiments.md)
> Lineage: [docs/design-lineage.md](docs/design-lineage.md)
> **Lens capability register: [docs/v3-lens-capability-register.md](docs/v3-lens-capability-register.md) — read before dispatching any brief that assumes "v3 subsumes v2 X".**
> **Compiler–`std/` consolidation end state: [docs/thesis/compiler-std-consolidation.md](docs/thesis/compiler-std-consolidation.md) — new types in `src/v3/compiler/*.dag` or `src/v3/std/*.dag` require a home-check against the positive definition (pipeline / regen / lens-body / accessor). Everything else schedules migration to `std/`.**

## How this doc is organized

Read this file for the live plan, milestone state, and current DB status lines. Read [docs/history/roadmap-post-ab-lane-plan.md](docs/history/roadmap-post-ab-lane-plan.md), [docs/history/roadmap-active-deferrals.md](docs/history/roadmap-active-deferrals.md), and [docs/history/roadmap-scheduled-deletions.md](docs/history/roadmap-scheduled-deletions.md) for full receipts and narrative detail.

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
- **`container_to_algebra` table duplicates type aliases** — `dsl/std/types.dag:140-153` is the string-keyed table; `types.dag:211-214` declares `type List<T> = FreeMonoid<T>` etc. Dissolution: derive the table from the aliases or delete it.
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

- **`src/v3/compiler/parse_parser_body.txt` — 1350 LOC hand-authored recursive-descent parse algorithm**: PR #589 retired `parse.rs` from the `.rs` census, but the algorithm moved here as a `.txt` fragment `include_str!`'d into `regen_parse` output. SG-0 now counts `.txt` scaffolds via `EXPECTED_HAND_AUTHORED_FRAGMENTS` in `sg0_census_test.rs` — the sole census authority for crate-root scaffolds. (Not added to `compiler.dag::stage0.hand_maintained_src`: that list models `source_dir` companions; its freshness/copy consumers never walk the crate root, so an entry there would be dead — extending the model to cover one crate-root file would be premature per `feedback_enumerate_before_substrate`.) SG-3f surface reflection is now landed: `runtime_mirrors.dag` owns the full Surface* carrier schema, Rust/Go/Python realization rows exist, parser-output mirror tests cover recursive sums/patterns/literals, and SG-3f-d consumption proof compiles/emits/links/runs against reflected surface values. Dissolution trigger is now solely structural `parse.dag` ownership via SG-2b proper. Owner: unblocked by SG-3f; queue behind SG-2b proper dispatch.

### Post-merge debt (2026-04-21 receipt-closure wave)

- **Class 5 Gap 1 — `Bool` inhabits `BooleanAlgebra<Bool>` grounding not wired**: surfaced during #610's closing. Current `OperatorKind::Logical(_)` dispatch in the three emitters is hardcoded (`if let OperatorKind::Logical(_)` branches bypass the algebra-field resolution that arithmetic operators use). The substrate already models `BooleanAlgebra<T>` with `meet/join/complement` per `dsl/std/algebra.dag:230-236`, and comments there state *"Bool inhabits BooleanAlgebra"* — but the `inhabits` edge isn't structurally landed. Dissolution: wire `Bool inhabits BooleanAlgebra<Bool>` + extend `resolve_operator_arrow` for `OperatorKind::Logical(_)` + delete the hardcoded emitter branches. Owner: 1e-2b lane (Path A).
- **`emit_rust_module` gap: SurfaceLiteral → LiteralBits rename**: substrate declares `LitInt/LitBool/LitString` in `LiteralBits`; Rust enum is `Int/Bool/String`; no rename facility in `emit_rust_module` causes connective-producing slices (per #612's staging choice notes) to fail on this. Dissolution: add per-variant name mapping in spec or rename-facility in emit. Owner: unassigned; blocks deeper SG-3g slices.
- **`emit_rust_module` gap: `render_variant_constructor` fails on external tuple variants**: `src/v3/compiler/src/emit/rust_target.rs:4296` unconditionally emits `Variant { _0: value }` which is wrong for external tuple variants. Dissolution: spec-driven variant-constructor template per source kind. Owner: unassigned; blocks deeper SG-3g slices.
- **File-preference rank is a ratified-parallel-authority scaffold**: `Dag::declaration_name_preference_rank` (`src/v3/compiler/src/dag.rs:1985-2015`) and its mirror in `collect_symbols` (`src/v3/compiler/src/lower.rs:1258-1278`) resolve same-named declarations by preferring `src/v3/` over `dsl/`. This is a P2 Boundary Discipline violation — a stable lookup rule that *normalizes* duplicate authority instead of dissolving it. Held as a scaffold rather than deleted because the duplicates carry v3-only substrate content (`v3.std.substrate` imports, `ElementRef` / `PortId` references, partitioned `EffectShape`, diagnostic-assertion surface) that cannot yet live under `dsl/std/` — v2 CI recursively ingests `dsl/` and cannot parse v3 grammar. Dissolution trigger: when every module currently duplicated between `dsl/std/` and `src/v3/std/` (or `src/v3/spec/`) has converged to a single canonical home, delete the rank function + mirror policy and remove the duplicate-authority preference rule entirely. After convergence, `declaration_by_name` must resolve against the single surviving authority or fail closed on multiple matches; no rank-based bridge remains. Convergence checklist: (1) `module std.effects` — `dsl/std/effects.dag` ↔ `src/v3/std/effects.dag` (v3 copy imports `v3.std.substrate`, partitions `EffectShape`, uses `ElementRef<OperationEffect>`); (2) `module std.verification` — `dsl/std/verification.dag` ↔ `src/v3/std/verification.dag` (surfaces are disjoint today: v2 `AssertKind/TestClaim/TestCase` vs v3 `DiagnosticKind / DiagnosticReference / PortStateExpectation`; convergence requires a design call on which surface wins); (3) embedded `http_path` mirror inside `src/v3/std/effects.dag:118-260` (resolves via either staging `http_path.dag` in v3 or rewiring imports to consume `std.http_path` directly once v3-substrate staging lands). Owner: unblocked by v3.std.substrate staging lane + v3-grammar CI convergence.
- **`DeclarationLookup` parallel authority in `src/v3/lenses/variant_payload.dag`**: a fail-open `find_declaration(decls, target) -> DeclarationLookup = LookupMissing | LookupFound(Declaration)` path duplicates `Dag::declaration(id)` and converts Track-9 constructor-validation failures into normal lens-consumer outcomes (C-8 violation). Originally surfaced via #609's infer_helpers work; that file was cleaned up independently, but the pattern persists in variant_payload.dag. Dissolution: replace `find_declaration` consumers with a typed lookup that consumes `Dag::declaration(id)`-equivalent authority, delete the parallel path. Owner: SG-4b-1-fix lane (rescoped).

### Post-merge debt (2026-04-21 deferred-from-wave)

Tracked debt not dispatched with the 2026-04-21 lane wave currently in flight (Lane 1e P3.0, SG-3g-b, SG-2c-4, SG-4b-3, cascade cleanup, algebra inhabitance quartet). Opportunistic slots — no blocking dependencies, can be picked up whenever bandwidth frees.

- **`algebra.dag` signature/comment reconciliation**: `dsl/std/algebra.dag:265-302` — `FreeMonoid<T>`'s comments promise `index(FreeMonoid<T>, Nat) -> T?`, `map(FreeMonoid<T>, fn(T)->U) -> FreeMonoid<U>`, `fold(FreeMonoid<T>, init: U, fn(U,T)->U) -> U`; the declarations drop the carrier/source parameters (`index: fn(Int) -> T`, `map: fn(fn(T)->T) -> FreeMonoid<T>`, `fold: fn(T, fn(T,T)->T) -> T`). `PartialFunction<K,V>` at `:307-325` has the same issue — comment promises `merge(Map<K,V>, Map<K,V>, fn(V,V)->V) -> Map<K,V>`, declaration is `merge: fn(PartialFunction<K,V>) -> PartialFunction<K,V>` (missing fn argument). One of the two (comments or declarations) is wrong; reconcile. P1 Modeling Faithfulness receipt. Owner: unassigned; small scope.
- **`container_to_algebra` string table is duplicate authority**: `dsl/std/types.dag:128-141` declares `container_to_algebra: Map<String, String>` mapping `"List" -> "FreeMonoid"`, `"Set" -> "BooleanAlgebra"`, `"Map" -> "PartialFunction"` — but the authoritative type aliases already exist at `:206-208` (`type List<element> = FreeMonoid<element>`, etc.). The file's own comment at `:126-127` carries the dissolution TODO. Dissolution: delete the map, point consumers at the typed aliases via structural identity. P2 Boundary Discipline / P1 receipt. Owner: unassigned; small scope.
- **`pipeline_authority.rs` reconstructs stage order from source text**: `src/v3/compiler/src/pipeline_authority.rs:225-275` tokenizes+parses `pipeline.dag`, slices `body_span` as raw source, iterates `body.lines()` for stage names. But `src/v3/compiler/pipeline.dag:17-23` declares `PipelineStageBinding` and `:49-83` declares per-stage bindings — the typed authority already exists. Dissolution: replace text-slicing with typed substrate read of `PipelineStageBinding` rows. P2 Boundary Discipline receipt. Owner: unassigned; small-medium scope.
- **Emitter render-helper consolidation (Phase 3.1 candidate after 1e P3.0)**: `named_variant_id` / `render_named_template` / `primitive_type_id_for_port` / `walk_to_disj` / `algebra_field_for_operator` exist 3-5× across `src/v3/compiler/src/emit.rs` + `emit/python_target.rs` + `emit/rust_target.rs` (plus `walk_to_disj_decl` also in `lower.rs` and `infer.rs`). Collapse into one emitter-shared module. Different scope from 1e P3.0 which targets `behavior_result_port` / `port_is_consumed_from`; this is the render-side tranche. P2 Boundary Discipline receipt. Owner: queued behind 1e P3.0 as the natural follow-up.
- **Stale §-style and line-number cross-references to pre-rewrite `INVARIANTS.md`**: Pro's post-#620 observation. Line-number refs in `docs/design-db16-refined-generic-substitution.md:297,298,413`, `docs/perf/clone-elimination.md:101`, `docs/lane2-compile-time-proofs.md:182`; §-style heading-name refs in `src/v3/M1_DESIGN.md` (§"No short-term solutions" / §"No bridges"), `src/v3/SELF_HOSTING.md` (§"Recursive syntax is sugar"), `docs/briefs/sg-4b-1-fix-declaration-lookup-cleanup.md` (§C-8 / §Track 9), `docs/briefs/p0-bug-no-profile-sentinel.md` (§"No fabrication sentinels"). Grep navigation still works because the strings survive as bullets in the new INVARIANTS.md, but line numbers have shifted and §-as-section framing is now approximate. Dissolution: sweep replacing line refs with `INVARIANTS.md#<id>` anchors (post-#620) or name-based refs. Cosmetic, low urgency. Owner: unassigned.
- **`Strict Forward Progress` subdoc ↔ heading drift**: pre-existing inconsistency between `docs/invariants/strict-forward-progress.md` (bounded-execution content) and the reviewer-usage meaning of the SFP rule name (dissolution-progress). Flagged explicitly in both P4 and P5 of INVARIANTS.md post-#620 as future cleanup. Dissolution: either rename the subdoc to reflect its bounded-execution content (and author a new dissolution-progress subdoc), or split the subdoc into two. Small scope once the design call is made on naming. Owner: unassigned.
- **"Return first match" cascade cleanup** (dispatched in the 2026-04-21 wave currently in flight; listed here only to cross-reference): `src/v3/compiler/src/dag.rs:1999` and `:2023`, and ROADMAP:169 row, all inherit the "return the first match or fail-closed on multiple" phrasing from #625. Correct dissolution target is fail-closed-only. Owner: 2026-04-21 wave (cascade cleanup micro-lane).
- **v3 lens capability honesty pass (supersedes the complexity-receipt lane as originally scoped)**: the 2026-04-21 audit (see [docs/v3-lens-capability-register.md](docs/v3-lens-capability-register.md)) found that `src/v3/lenses/complexity.dag` does not subsume v2's `complexity.dag` — v2 produces symbolic `CostExpr` / `SizeExpr` with work/span/certainty/asymptotic-class, v3 produces a single integer depth per port. The same pattern holds for `cost.dag` (PROXY — no named size variables, Dimension wiring deferred), `idempotency.dag` (STUB — Rust oracle), and `parallelism.dag` (STUB — fail-closed placeholder). The originally-claimed "~13× LOC reduction with structurally stronger fail-closed guarantees" was comparing different functions of the program; retracted. **Follow-up work:** (1) the cementing-test discipline generalizes from one lens to every `regen.dag` entry (semantic-equality goldens per lens); (2) `docs/briefs/complexity-v2-v3-comparison-receipt.md` to be rewritten against the register — scope widens from "complexity receipt + registry rename" to "Band C honesty pass" once the register is merged. Substrate dissolution path: port `DescentEvidence` / `CallPattern` / `SubValueRelation` / `MethodSemantics` into the v3 substrate (currently blocks genuine equivalence for complexity + cost). Emit dissolution path: `match` lowering over user-defined sums (currently blocks `.dag` authority for idempotency + parallelism). Both are programs of work, not single lanes. Owner: unassigned; brief rewrite queues for after the 2026-04-21 wave's 5 lanes land and this register is in tree.
- **CI ratchet architecture — exemption widening erodes the per-test timeout ratchet**: surfaced during the 2026-04-21 reflective analysis (`53b3110..ae8825a` range). Commits `37cd6128`, `4898983e`, `f84ed355` hardened the ratchet against CI-log instability, but `2d8396df` widened the exemption list. `scripts/slow-test-exemptions.txt` is 83 lines with no monotonic-shrink rule — `feedback_ratchet_only_down` violated at the meta level (the primary ratchet can drift upward through exemption growth). Dissolution: audit the exemption list, categorize each (stale / paydown / structural), delete stale entries, add a meta-ratchet that fails CI on exemption-count growth or per-exempt budget drift. Dispatch brief: `docs/briefs/ci-ratchet-architecture-audit.md`. Owner: unassigned; queue as infrastructure-principal lane, soon.
- **Stale-receipt sweep in `docs/briefs/`**: surfaced during the 2026-04-21 reflective analysis. Briefs under `docs/briefs/` and prose references in adjacent docs still point at debt that the final code has partially or fully dissolved — the `DeclarationLookup` cleanup is the cited example where brief prose names debt the code has largely resolved. Integration drag, not code regression. Dissolution: sweep `docs/briefs/` for superseded claims post-wave, convert closed briefs to historical receipts under `docs/history/` or add a "Closed by #NNN" line, update inline prose in sibling docs that cite the resolved debt. Small scope, cosmetic. Owner: unassigned; do soon after the 2026-04-21 wave lands (most of its outputs will trigger more brief-staleness). **Include:** `docs/design-pure-bootstrap.md`'s hand-maintained count ("78 hand-maintained .rs files") is stale — live `EXPECTED_HAND_AUTHORED` in the SG-0 census has 89+ entries under `src/v3/compiler/`. Either refresh the number or delete it and point at the live census; detected during #642 review.
- **Compiler–`std/` consolidation program — specific migrations**: end-state defined in [docs/thesis/compiler-std-consolidation.md](docs/thesis/compiler-std-consolidation.md). Each of the migrations below is a standalone lane; together they collapse the compiler-specific type surface toward the positive definition (pipeline + regen + lens-specific return-type carriers + accessor). **Ratchet:** count of `type` declarations in `src/v3/compiler/*.dag` AND `src/v3/lenses/*.dag` that are NOT in the positive-def set AND NOT exempted → 0. Positive-def: pipeline/regen types + lens-API return carriers (e.g., `Origin`, `UnusedParameter`). Exempted: `parse_tables.dag` (4 types, pending SG-2c-proper per-row classification). In-ratchet: tokenize (6) + runtime_mirrors (14) + generic-Lookup-pattern carriers across lenses (~5). Baseline: ~25 (exact lens breakdown firming up as Lookup generalization lane scopes). See thesis doc table for per-file disposition.
  - **`tokenize.dag` → `std/tokenize.dag`** — move `Token`, `TokenKind`, `KeywordTokenKind`, `PunctTokenKind`, `LocalPunctSpec`, `StringEscapeSpec` to `std/`. ~6 types. Owner: unassigned; migration-gate: none (types already composable).
  - **`runtime_mirrors.dag` → `std/syntax.dag` (or new `std/parse_surface.dag`)** — move `Surface*` carriers (~14 types). Owner: unassigned; migration-gate: `SG-2b/SG-3f` parse-rule cutover (the existing 🟡 SCAFFOLD trigger in `runtime_mirrors.dag` already names this).
  - **`parse_tables.dag`** — decide per-type whether `BinaryOpLevel` / `BinaryOpRow` / `TopLevelItemKwRow` / `BracketRow` stay compiler-API (dispatch-row shapes) or move to `std/syntax.dag` (language-level precedence/dispatch facts). Owner: whoever picks up **SG-2c proper** parser cutover; migration-gate: SG-2c proper completion (the `.dag` parser reveals which rows it dispatches on vs which are pure data). Precedent rule: if any individual row needs to move before SG-2c proper lands, the mover-lane sets the classification and the rest follow the same rule when touched.
  - **`src/v3/std/*.dag` → `dsl/std/*.dag`** — the whole v3-specific std tree collapses when the file-preference-scaffold dissolves (gated on v2 retirement or `dsl/std/` learning v3 grammar). Largest single consolidation; tracked separately in the **"File-preference rank is a ratified-parallel-authority scaffold"** row (earlier in the 2026-04-21 receipt-closure wave post-merge-debt section).
  - **`Node` → `std/node.dag`** — already captured in `project_node_to_std` memory as prior pattern. Relevant to this program as a precedent.
  - **Generic `Lookup<T>` in `std/` — dissolve per-lens `CostLookup` / `TemplateArgumentLookup` / `DeclarationLookup` duplicates** — each lens currently declares its own 2-variant `Missing | Found(T)` carrier. Collapse to a single `std/Lookup<T>` generic consumed by every lens that returns a possibly-missing result. ~5 types dissolve into 1. Owner: unassigned; migration-gate: none (pure type-declaration work; generics already work per the earlier ParseStep discussion).

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
