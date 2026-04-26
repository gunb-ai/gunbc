# R2 Impossible-Bugs Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827). Spawns on R1 close per Transition mechanics step 4. NEW manager.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of 6 standing R2 managers.
- **Program scope source:** [`docs/r2-structure.md` §"Goals" item 4](../r2-structure.md) + [`THESIS.md §"Enumerable impossible-bug classes"`](../../THESIS.md) (R2+ tags authority).
- **Cross-program coordination:** classes that surface substrate gaps escalate to Substrate Manager rather than expanding T-ImpossibleBugs scope.

## Program scope (T-ImpossibleBugs)

R2's Goal 4 — **Remaining R2+ impossible-bug classes**. Three classes currently tagged `[R2+]` in `ROADMAP.md §"Lane acceptance — .dag gates"` (T-Demo row); THESIS §"Enumerable impossible-bug classes" is authority on scheduling tags.

| Class | Status (at brief authoring) | Substrate gating |
|---|---|---|
| Nested-optional flatten | DESIGN/SCOPING DOC AUTHORED (PR #798 — bright-moth-390 redirect) | gated on cardinality refinement; coordinate with Substrate Manager |
| Unhandled diagnostic paths | DESIGN/SCOPING DOC AUTHORED (PR #801 — sunny-deer-629 redirect) | Tier 2 substrate; may require coordination with Substrate Manager |
| Unenumerated effects | DESIGN AUTHORED (PR #808 — closed-system effects model is canonical reference); Fn→Arrow refactor pre-prereq (PR #805) | post-effects-design; Fn→Arrow may be prereq |

Each class becomes an impossible-bug-by-construction at R2 close — not a runtime check, but a structural fact carried by the substrate that the lens / type checker reads.

## Owned deliverables (through R2 close)

For each class:
- Convert design/scoping doc to worker brief (Director-authored per coordination on inbox #828).
- Worker dispatched (potentially gated on substrate work; coordinate with Substrate Manager).
- Migration / implementation that makes the bug class structurally impossible to author.
- Structural test demonstrating the impossible-bug status (e.g., synthetic input → compile-time diagnostic, not runtime panic).
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

Design/scoping (already authored):
- Nested-optional flatten — `docs/briefs/t-impossiblebugs-nested-optional-flatten.md` (or similar; per PR #798)
- Unhandled diagnostic paths — `docs/briefs/t-impossiblebugs-unhandled-diagnostic-paths.md` (or similar; per PR #801)
- Unenumerated effects — `docs/briefs/t-impossiblebugs-unenumerated-effects-design.md` (per #808)

Pending — Director-authored worker briefs (convert design → worker):
- Nested-optional flatten worker
- Unhandled diagnostic paths worker
- Unenumerated effects worker (with Fn→Arrow prereq sequencing per #805)

## Working state (fill on spawn)

Class status table refreshes here as work lands. Pre-spawn placeholder.

## Cross-refs

- Parent: `docs/r2-structure.md` §"Impossible-Bugs Manager"
- Goal authority: `THESIS.md §"Enumerable impossible-bug classes"` (R2+ tags)
- Effects framing: PR #808 (closed-system effects model + 5-behavior fold)
- Effects prereq: PR #805 (Fn→Arrow refactor)
- Adjacent designs: PRs #798 (nested-optional), #801 (unhandled diagnostics)
