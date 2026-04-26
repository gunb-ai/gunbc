# R2 Release Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827). Spawns on R1 close per Transition mechanics step 4. NEW manager — owns release coordination + smallest cross-cutting deliverables.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of 6 standing R2 managers; **the closure-ledger and demo-coordination single authority** for R2.
- **Program scope source:** [`docs/r2-structure.md` §"Goals" items 5, 6 + cross-cutting deliverables](../r2-structure.md).
- **Cross-program coordination:** receives lane-close signals from all 5 other managers (Grounding, Substrate, Modeling, Impossible-Bugs, Pure Bootstrap); surfaces R2 demo artifacts to user.

## Program scope (T-Release)

R2's smallest cross-cutting deliverables + release coordination + demo + discipline-framework reporting + closure ledger.

| Deliverable | Source | Status (at brief authoring) |
|---|---|---|
| §6a per-method-metadata follow-through (Goal 5) | `docs/design-substrate-carrier-port-program.md §6a` (decision locked at `:171`; live receipt at `:173`; dissolution trigger at `:175`) | DECISION LOCKED — Option 3 unified `MethodContract` carrier; live receipt landed (`src/v3/std/algebra.dag` declares `MethodContract`; `src/v3/lenses/cost.dag` imports via `method_contract_cost_shape` minimal demo consumer). Remaining R2 work: bulk migration of `cost.dag` / `complexity.dag` to live call-site lookup + dissolution-trigger tracking (`size_effect` / `cost_shape` / `callback_element_position` field-by-field retirement). |
| R2 closure demo coordination (Goal 6) | `docs/r2-structure.md` Goal 6 + Demo discipline section | LIVE on spawn (no separate brief; see Demo discipline) |
| B-wave Tier 0 through-merge | #810 §5 dispatch ordering | IN FLIGHT (B1 #820, B2 #817, B3 #821 — implementation iteration) |
| B-wave Tier 2 brief authoring | #810 §5 (B5 Loop construction-closure audit; B6 file-preference rank checklist; B7 priority hint to Pure Bootstrap Manager) | **AUTHORED** — `r2-release-b5-*.md`, `r2-release-b6-*.md`, `r2-release-b7-*.md` (PM portion per inbox #828). B6 resolution: extend `declaration_name_preference_rank` / `collect_symbols` convergence checklist (`std.computation` / `std.induction` / `std.termination`). |
| Discipline framework central reporting | INVARIANTS.md §P5 "Dispatch-Discipline Mechanisms" (c) + ROADMAP.md "Integration-reflection cadence" velocity-tripwire | LIVE on spawn (mechanism wired via PRs #810 + #812) |
| Thesis-claim coverage mapping | `docs/r2-structure.md` Open call 1 | NOT YET AUTHORED (lands as part of R1→R2 transition step 4) |
| R2 closure ledger | `docs/r2-structure.md` Manager structure section 6 | LIVE on spawn (tracks lane-close green status across all 5 other managers) |
| v2 retirement coordination | `docs/r2-structure.md` Transition mechanics v2-retirement note | LIVE on spawn (post-R2 operational; tracked but not gated) |

## Owned deliverables (through R2 close)

### Core deliverables (must complete by R2 close)

1. **§6a per-method-metadata follow-through brief** — **Pick is closed.** Design-call decision is locked as Option 3 unified `MethodContract` carrier per [`docs/design-substrate-carrier-port-program.md §6a:171`](../design-substrate-carrier-port-program.md), with live receipt at `:173`: `src/v3/std/algebra.dag` declares `MethodContract`; `src/v3/lenses/cost.dag` imports it via `method_contract_cost_shape` minimal demo consumer. The pick-worker brief at [`docs/briefs/t-permethodmetadata-pick-worker.md`](t-permethodmetadata-pick-worker.md) (landed PR #794) authored the pick + lock + minimal demo and is **closed in scope** — its "Do not migrate all consumer lenses ... bulk migration is post-pick work" clause names the residual this manager owns. R2 follow-through deliverables (NOT a re-pick): (a) bulk migration of `cost.dag` / `complexity.dag` to live call-site `MethodContract` lookup; (b) track the field-by-field dissolution trigger from `:175` (`size_effect` → cardinality-refined signatures; `cost_shape` → typed cost surface; `callback_element_position` → typed higher-order callback parameter shape). No duplicate decision authority — pick is closed; follow-through is post-pick scope.
2. **B5 Loop construction-closure audit brief** — Tier 2 from #810 §5. **MUST start with construction-closure audit, NOT marker design.** If audit confirms Loop only reachable through recursive-function lowering, deliverable is structural integration test. If audit refutes, marker brief.
3. **B6 file-preference rank checklist completion brief** — trivial S, one-line ROADMAP fix per #810 §5.
4. **B7 priority-hint relay to Pure Bootstrap Manager** — lift `patch_lower_helpers_*` retirement to PB-Tier1 priority. Cross-manager signal, not a worker brief.
5. **Thesis-claim coverage mapping** (Open call 1 from `docs/r2-structure.md`) — table mapping every Tier-1 / Tier-2 / Tier-3 THESIS claim to R1-closed / R2-gated / post-R2-external disposition. Lands as part of R1 closure → R2 promotion transition.

### Standing-state deliverables (continuous through R2)

6. **B1, B2, B3 Tier 0 through-merge** — coordinate worker iteration on the 3 fail-closed P3 leak fixes already in flight (#820, #817, #821). Surface STOP-AND-ESCALATE signals if workers bounce repeatedly.
7. **R2 demo coordination** — surface "it runs" artifacts at each lane close per Demo discipline (single authority per `docs/r2-structure.md` 2026-04-26 rework).
8. **Closure ledger** — track lane-close green status across all 5 other managers; surface unblocked work to idle workers; coordinate v2 retirement post-R2.
9. **Velocity-tripwire reporting** — each integration-reflection cadence pass, report introduction:dissolution PR ratio across all manager-authored work; surface ≥3:1 readings to Director per INVARIANTS.md §P5 (c). Manual sweep for dissolution-bearing feature PRs first per calibration caveat.
10. **Substrate Manager bottleneck watch** — per `docs/r2-structure.md` watch condition: if Substrate becomes the new bottleneck (workers idle >7 days waiting for Substrate-authored briefs), surface to Director and recommend B4 split into dedicated B4 Identity-Carrier Manager.

## Cross-program dependencies

**Produces:**
- Closure ledger updates → Director (weekly health check) + user (R2 demo surfacing).
- Velocity-tripwire alerts → Director (when ≥3:1 ratio fires).
- Bottleneck watch alerts → Director (when Substrate Manager workers idle >7 days).

**Consumes (lane-close signals from all 5 other managers):**
- Grounding Manager — T-Ground lane-close signals
- Substrate Manager — T-Substrate sub-lane / B4 Phase close signals
- Modeling Manager — T-Modeling item-close signals
- Impossible-Bugs Manager — T-ImpossibleBugs class-close signals
- Pure Bootstrap Manager — post-R1 PB lane-close signals

**Adjacent territory:** none (Release is cross-cutting).

## Pre-spawn vs post-spawn authority

- **Pre-spawn (now, before R1 close):** Director + PM coordinate on brief authoring per inbox #828 split. PM authors the manager skeleton (this file) + the §6a / B5 / B6 / B7 / thesis-claim-mapping briefs that are this manager's PM-portion deliverables (per inbox #828). Both stop authoring once R2 spawns.
- **Post-spawn (R2 promotion onward):** Manager owns all worker-brief authoring autonomously per "Autonomous dispatch authority" below. Director's role narrows to cross-program conflict resolution + scope-change escalation.

## Autonomous dispatch authority

- Authors all T-Release owned deliverables without Director: worker briefs (§6a follow-through, B5, B6, thesis-claim coverage mapping) and cross-manager signals (B7 priority-hint relay).
- Dispatches workers against owned deliverables.
- Owns the **central reporting** layer of P5 dispatch-discipline (per `docs/r2-structure.md`); per-brief paired-dispatch and per-PR gate enforcement happens at each manager's authoring point, not at Release.
- Resolves Release-internal scope refinements; escalates blockers and scope changes to Director.

## Reporting cadence

- **To Director (cross-program):** velocity-tripwire alerts (≥3:1 ratio), bottleneck watch alerts (Substrate or any manager idle >7 days), closure-ledger weekly health check.
- **To user (release coordination):** R2 demo artifacts at each lane close (per Demo discipline section); closure ledger surfacing.
- **Blockers + scope changes:** Director.

## Sub-briefs (authored / pending)

Authored (pre-spawn PM portion, inbox #828):
- [`r2-release-6a-follow-through-worker.md`](r2-release-6a-follow-through-worker.md) — §6a bulk migration + dissolution-trigger tracking
- [`r2-release-b5-loop-construction-closure-audit-worker.md`](r2-release-b5-loop-construction-closure-audit-worker.md)
- [`r2-release-b6-file-preference-rank-checklist-worker.md`](r2-release-b6-file-preference-rank-checklist-worker.md)
- [`r2-release-b7-priority-hint-relay-to-pure-bootstrap.md`](r2-release-b7-priority-hint-relay-to-pure-bootstrap.md) — cross-manager signal brief

Still pending at R1→R2 transition (not pre-spawn worker briefs):
- Thesis-claim coverage mapping table (Open call 1; lands at R1 close → R2 promotion)

Post-spawn: manager-authored worker briefs autonomously per "Pre-spawn vs post-spawn authority" above.

## Working state (fill on spawn)

Closure ledger snapshot + velocity-tripwire ratio refreshes here at each integration-reflection cadence pass. Pre-spawn placeholder.

## Cross-refs

- Parent: `docs/r2-structure.md` §"R2 Release Manager"
- Discipline framework: INVARIANTS.md §P5 "Dispatch-Discipline Mechanisms" (a)/(b)/(c)
- Cadence wiring: ROADMAP.md "Integration-reflection cadence" + velocity-tripwire reporting
- §6a source: `docs/design-substrate-carrier-port-program.md §6a` (decision locked Option 3; live receipt landed)
- Thesis-claim authority: `THESIS.md §"Thesis claims — complete list"`
- B-wave source: `docs/briefs/debt-paydown-synthesis-2026-04-25.md` §5 (B1-B7 dispatch ordering)
