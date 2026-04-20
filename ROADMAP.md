# gunbc Roadmap

Single source of truth for project status, active work, and deferred items. Long-form receipts and historical narratives now live under `docs/history/` and `docs/db-history/` so this file can stay operational.

> Design spec: [docs/v3-spec.md](docs/v3-spec.md)
> Validation: [docs/v3-validation-experiments.md](docs/v3-validation-experiments.md)
> Lineage: [docs/design-lineage.md](docs/design-lineage.md)

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

- **`src/v3/compiler/parse_parser_body.txt` — 1350 LOC hand-authored recursive-descent parse algorithm**: PR #589 retired `parse.rs` from the `.rs` census, but the algorithm moved here as a `.txt` fragment `include_str!`'d into `regen_parse` output. SG-0 now counts `.txt` scaffolds (`EXPECTED_HAND_AUTHORED_FRAGMENTS`) and `compiler.dag::hand_maintained_src` names the file so both census authorities track it. Dissolution trigger: structural `parse.dag` ownership via SG-2b proper or SG-3f surface reflection follow-on. Owner: queued behind SG-3f.

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
