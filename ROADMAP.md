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
- `DB-11`: `where` refinement shipped; the out-of-fragment rejection and narrowing receipts moved out of line. See [docs/db-history/db-11.md](docs/db-history/db-11.md).
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
