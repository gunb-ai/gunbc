# R2 Impossible-Bugs Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827). Spawns on R1 close per Transition mechanics step 4. NEW manager.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of 6 standing R2 managers.
- **Program scope source:** [`docs/r2-structure.md` §"Goals" item 4](../r2-structure.md) + [`THESIS.md §"Enumerable impossible-bug classes"`](../../THESIS.md) (R2+ tags authority).
- **Cross-program coordination:** classes that surface substrate gaps escalate to Substrate Manager rather than expanding T-ImpossibleBugs scope.

## Program scope (T-ImpossibleBugs)

R2's Goal 4 — **Remaining R2+ impossible-bug classes**. Three classes currently tagged `[R2+]` in `ROADMAP.md §"Lane acceptance — .dag gates"` (T-Demo row); THESIS §"Enumerable impossible-bug classes" is authority on scheduling tags.

| Class | Authored briefs (canonical filenames) | Status (at brief authoring) | Substrate gating |
|---|---|---|---|
| Nested-optional flatten | `t-impossiblebugs-nested-optional-flatten-design.md` (PR #798) + `t-impossiblebugs-nested-optional-flatten-worker.md` (DESIGN/SCOPING shape — produces substrate proposal, not implementation) | Worker brief authored | gated on cardinality refinement; coordinate with Substrate Manager |
| Unhandled diagnostic paths | `t-impossiblebugs-unhandled-diagnostic-paths-design.md` (PR #801) + `t-impossiblebugs-unhandled-diagnostic-paths-worker.md` (DESIGN/SCOPING shape — produces substrate proposal, not implementation) | Worker brief authored | Tier 2 substrate; may require coordination with Substrate Manager |
| Unenumerated effects | `t-impossiblebugs-unenumerated-effects-design.md` (PR #808 — **canonical authority**, closed-system effects model + 5-behavior fold) | Both prior worker briefs (`t-impossiblebugs-unenumerated-effects-worker.md` + `t-impossiblebugs-unenumerated-effects-parser-worker.md`) are **SUPERSEDED 2026-04-25** by the design doc; route new work through the design doc, not the superseded workers | post-effects-design; Fn→Arrow refactor (PR #805) is independent prereq for vestigial `params + return_type` cleanup, not directly gating effects framing |

Each class becomes an impossible-bug-by-construction at R2 close — not a runtime check, but a structural fact carried by the substrate that the lens / type checker reads.

## Owned deliverables (through R2 close)

For each class:
- Worker brief is **already authored** (see Program scope table above for canonical filenames). Note that nested-optional + unhandled-diagnostic-paths workers are DESIGN/SCOPING shape — they produce substrate proposals, not direct implementation. Effects worker is SUPERSEDED; route through the design doc.
- Manager dispatches the existing worker (potentially gated on substrate work; coordinate with Substrate Manager).
- Sub-substrate work surfaced by DESIGN/SCOPING workers becomes a Substrate Manager hand-off (escalate per cross-program rule below).
- Implementation phase (post-substrate-proposal) that makes the bug class structurally impossible to author.
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

**Already authored — canonical filenames:**
- Design: `docs/briefs/t-impossiblebugs-nested-optional-flatten-design.md` (PR #798)
- Worker (DESIGN/SCOPING shape): `docs/briefs/t-impossiblebugs-nested-optional-flatten-worker.md`
- Design: `docs/briefs/t-impossiblebugs-unhandled-diagnostic-paths-design.md` (PR #801)
- Worker (DESIGN/SCOPING shape): `docs/briefs/t-impossiblebugs-unhandled-diagnostic-paths-worker.md`
- Design (canonical authority): `docs/briefs/t-impossiblebugs-unenumerated-effects-design.md` (PR #808)

**SUPERSEDED — do not author against, route through design doc:**
- `docs/briefs/t-impossiblebugs-unenumerated-effects-worker.md` (SUPERSEDED 2026-04-25 by design doc)
- `docs/briefs/t-impossiblebugs-unenumerated-effects-parser-worker.md` (SUPERSEDED 2026-04-25 by design doc; closed-system framing dissolves the proposed parser surface)

**Pending — Manager dispatches existing workers + handles design-doc routing for effects:**
- Dispatch nested-optional-flatten worker (DESIGN/SCOPING produces substrate proposal → escalate to Substrate Manager)
- Dispatch unhandled-diagnostic-paths worker (DESIGN/SCOPING → escalate to Substrate Manager if Tier 2 substrate gap surfaces)
- Author effects implementation worker against the canonical design doc (post-supersede; closed-system mechanism + 5-behavior fold). PR #805 Fn→Arrow refactor stays dispatchable as independent vestigial-syntax cleanup.

## Working state (fill on spawn)

Class status table refreshes here as work lands. Pre-spawn placeholder.

## Cross-refs

- Parent: `docs/r2-structure.md` §"Impossible-Bugs Manager"
- Goal authority: `THESIS.md §"Enumerable impossible-bug classes"` (R2+ tags)
- Effects framing: PR #808 (closed-system effects model + 5-behavior fold)
- Effects prereq: PR #805 (Fn→Arrow refactor)
- Adjacent designs: PRs #798 (nested-optional), #801 (unhandled diagnostics)
