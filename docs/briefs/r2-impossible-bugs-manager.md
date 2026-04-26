# R2 Impossible-Bugs Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827). Spawns on R1 close per Transition mechanics step 4. NEW manager.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of 6 standing R2 managers.
- **Program scope source:** [`docs/r2-structure.md` §"Goals" item 4](../r2-structure.md) + [`THESIS.md §"Enumerable impossible-bug classes"`](../../THESIS.md) (R2+ tags authority).
- **Cross-program coordination:** classes that surface substrate gaps escalate to Substrate Manager rather than expanding T-ImpossibleBugs scope.

## Program scope (T-ImpossibleBugs)

R2's Goal 4 — **Remaining R2+ impossible-bug classes**. Three classes currently tagged `[R2+]` in `ROADMAP.md §"Lane acceptance — .dag gates"` (T-Demo row); THESIS §"Enumerable impossible-bug classes" is authority on scheduling tags.

| Class | Design authority + implementation worker (post PR #836 merge) | Implementation status | Substrate gating |
|---|---|---|---|
| Nested-optional flatten | Design: `t-impossiblebugs-nested-optional-flatten-design.md` (PR #798) — closed in scope; Implementation: [`r2-impossible-bugs-nested-optional-flatten-worker.md`](r2-impossible-bugs-nested-optional-flatten-worker.md) (PR #836 — landed) | IMPLEMENTATION WORKER LANDED — dispatchable Day-1 post-spawn | UNGATED per design doc audit (substrate-constructor invariant; cardinality bridge already past per Director reframe) |
| Unhandled diagnostic paths | Design: `t-impossiblebugs-unhandled-diagnostic-paths-design.md` (PR #801) — closed in scope, §4 Director-actionable recommends totality-by-omission; Implementation: [`r2-impossible-bugs-unhandled-diagnostic-paths-worker.md`](r2-impossible-bugs-unhandled-diagnostic-paths-worker.md) (PR #836 — landed) | IMPLEMENTATION WORKER LANDED — dispatchable Day-1 post-spawn (per-class; `Int / Int` first slice) | UNGATED per design doc §4 (totality-by-omission via algebra retype, not predicate-entailment substrate); siblings (indexing, quotient, remainder) queue separately |
| Unenumerated effects | Design: `t-impossiblebugs-unenumerated-effects-design.md` (PR #808) — **canonical authority**, closed-system effects model + 5-behavior fold; Implementation: [`r2-impossible-bugs-unenumerated-effects-worker.md`](r2-impossible-bugs-unenumerated-effects-worker.md) (PR #836 — landed) | IMPLEMENTATION WORKER LANDED — dispatchable Day-1 post-spawn (closed-system per design doc §Q1-Q3 + §Q6) | UNGATED — closed-system structural derivation does not require new substrate; prior worker briefs (`t-impossiblebugs-unenumerated-effects-worker.md` + `-parser-worker.md`) SUPERSEDED 2026-04-25 |

Each class becomes an impossible-bug-by-construction at R2 close — not a runtime check, but a structural fact carried by the substrate that the lens / type checker reads.

## Owned deliverables (through R2 close)

For each class:
- Implementation worker brief **landed on main via PR #836 merge** (see Program scope table above for canonical filenames + UNGATED status). The earlier DESIGN/SCOPING workers (`t-impossiblebugs-nested-optional-flatten-worker.md`, `t-impossiblebugs-unhandled-diagnostic-paths-worker.md`) and the SUPERSEDED effects workers were absorbed by the design docs + new implementation workers — do not re-dispatch the older workers.
- Manager dispatches the implementation worker for each class (each ungated per design-doc audit; substrate gaps surface during execution as STOP-AND-ESCALATE → Substrate Manager).
- Substrate-gap discoveries (if any class's implementation surfaces a real substrate need beyond what design docs accounted for) become a Substrate Manager hand-off (escalate per cross-program rule below). This is the exception path; the expected path is direct implementation per design-doc Director-actionable recommendation.
- Structural test demonstrating the impossible-bug status (e.g., synthetic input → compile-time diagnostic, not runtime panic) — included in each implementation worker brief's acceptance.
- Class-close signal to R2 Release Manager.

## Cross-program dependencies

**Produces (none).**

**Consumes (potentially):**
- Substrate Manager — cardinality refinement (for nested-optional flatten; if substrate gap surfaces beyond what's in T-Substrate sub-lanes)
- Substrate Manager — Tier 2 runtime-safety substrate (for unhandled diagnostic paths; Tier 2 substrate work is post-R1 territory)

**Adjacent designs (already landed):**
- `#808` — closed-system effects model + 5-behavior fold framing (canonical for unenumerated effects work)
- `#805` — Fn→Arrow refactor pre-prereq (may gate unenumerated effects worker)

**Escalation rule:** if a class turns out to need substrate work outside what's already scoped in T-Substrate sub-lanes, **STOP and escalate to Substrate Manager** rather than expanding T-ImpossibleBugs scope. Substrate Manager re-scopes; T-ImpossibleBugs class waits on substrate.

## Pre-spawn vs post-spawn authority

- **Pre-spawn (now, before R1 close):** Director + PM coordinate on brief authoring per inbox #828 split. PM authors the manager skeleton (this file); Director authors any worker-level briefs not yet existing per the manager's "Pending" sub-briefs list. Both stop authoring once R2 spawns.
- **Post-spawn (R2 promotion onward):** Manager owns all worker-brief authoring autonomously per "Autonomous dispatch authority" below. Director's role narrows to cross-program conflict resolution + scope-change escalation.

## Autonomous dispatch authority

- Authors all T-ImpossibleBugs worker briefs without Director.
- Dispatches workers against worker briefs (subject to substrate-gating coordination with Substrate Manager).
- Resolves T-ImpossibleBugs-internal scope refinements; escalates substrate gaps to Substrate Manager and blockers/scope changes to Director.
- Per `docs/r2-structure.md` P5 dispatch-discipline: every T-ImpossibleBugs worker brief names dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance.

## Reporting cadence

- Class-close → R2 Release Manager (closure ledger).
- Substrate-gap discoveries → Substrate Manager (cross-manager queue).
- Blockers + scope changes → Director.

## Sub-briefs (authored / pending)

**Design docs landed (closed in scope — next-step recommendations are authority for implementation worker briefs):**
- `docs/briefs/t-impossiblebugs-nested-optional-flatten-design.md` (PR #798) — design + scoping closed; implementation path: substrate-constructor invariant per design doc §Director-actionable.
- `docs/briefs/t-impossiblebugs-unhandled-diagnostic-paths-design.md` (PR #801) — design + scoping closed; implementation path: totality-by-omission per design doc §4 Director-actionable recommendation.
- `docs/briefs/t-impossiblebugs-unenumerated-effects-design.md` (PR #808) — canonical authority; implementation path: closed-system structural derivation per design doc §Q1-Q3 + §Q6.

**Implementation worker briefs landed on main via PR #836 merge — manager dispatches against these:**
- [`docs/briefs/r2-impossible-bugs-nested-optional-flatten-worker.md`](r2-impossible-bugs-nested-optional-flatten-worker.md) — M; ungated per design doc (substrate-constructor invariant; cardinality bridge already past per Director audit).
- [`docs/briefs/r2-impossible-bugs-unhandled-diagnostic-paths-worker.md`](r2-impossible-bugs-unhandled-diagnostic-paths-worker.md) — M; per-class totality-by-omission (Int / Int divide-by-zero first; siblings queue separately).
- [`docs/briefs/r2-impossible-bugs-unenumerated-effects-worker.md`](r2-impossible-bugs-unenumerated-effects-worker.md) — M; closed-system structural derivation; no substrate gate.

**SUPERSEDED — do not dispatch (implementation workers above replace these):**
- `docs/briefs/t-impossiblebugs-nested-optional-flatten-worker.md` — SUPERSEDED by `r2-impossible-bugs-nested-optional-flatten-worker.md`; the older DESIGN/SCOPING worker's role was absorbed into the design doc + implementation worker pair.
- `docs/briefs/t-impossiblebugs-unhandled-diagnostic-paths-worker.md` — SUPERSEDED by `r2-impossible-bugs-unhandled-diagnostic-paths-worker.md`; same pattern.
- `docs/briefs/t-impossiblebugs-unenumerated-effects-worker.md` (SUPERSEDED 2026-04-25 by design doc PR #808; further dissolved by `r2-impossible-bugs-unenumerated-effects-worker.md` now landing as the implementation authority).
- `docs/briefs/t-impossiblebugs-unenumerated-effects-parser-worker.md` (SUPERSEDED 2026-04-25 by design doc; closed-system framing dissolves the proposed parser surface).

**Pending — Manager dispatches the 3 implementation workers landed via PR #836:**
- Dispatch nested-optional-flatten implementation worker (ungated; dispatchable Day-1 post-spawn).
- Dispatch unhandled-diagnostic-paths implementation worker (ungated; per-class — `Int / Int` first slice).
- Dispatch unenumerated-effects implementation worker (ungated; closed-system per design doc).
- PR #805 Fn→Arrow refactor stays dispatchable as independent vestigial-syntax cleanup (not a thesis-claim closure dependency).

## Working state (fill on spawn)

Class status table refreshes here as work lands. Pre-spawn placeholder.

## Cross-refs

- Parent: `docs/r2-structure.md` §"Impossible-Bugs Manager"
- Goal authority: `THESIS.md §"Enumerable impossible-bug classes"` (R2+ tags)
- Effects framing: PR #808 (closed-system effects model + 5-behavior fold)
- Effects prereq: PR #805 (Fn→Arrow refactor)
- Adjacent designs: PRs #798 (nested-optional), #801 (unhandled diagnostics)
