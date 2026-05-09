# R2 Release Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827; refreshed 2026-04-28 post-#1078 merge to absorb 6→7 manager count + structural-acceptance-per-lane-close discipline + cross-program closure ledger across all 6 other R2 managers). Eligible to spawn pre-R1-close per `r2-structure.md` Transition mechanics step 4 (no technical R1 dependency). NEW manager — owns release coordination + smallest cross-cutting deliverables.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of **7** standing R2 managers (count rose from 6 to 7 with Evaluator added 2026-04-28 per #1078); **the closure-ledger and demo-coordination single authority** for R2.
- **Program scope source:** [`docs/r2-structure.md` §"Goals" items 5, 6 + cross-cutting deliverables](../r2-structure.md).
- **Cross-program coordination:** receives lane-close signals from all 6 other managers (Substrate, Modeling, Grounding, Impossible-Bugs, Pure Bootstrap, Evaluator); surfaces R2 demo artifacts to user.
- **Structural-acceptance-per-lane-close discipline (Director-locked 2026-04-28):** **the demo IS the structural acceptance gate.** Each lane's closure PR proves the lane closed correctly via a structural acceptance gate (e.g., `omni_layers_share_one_node_tree`, `bound_declaration_carries_three_variants_structurally`, `lens_complexity_n_log_n_fold_correct`). The demo is the gate's evidence — not a separate ad-hoc artifact. R2 Release Manager coordinates visibility (surfacing gates as they fire); the *authoring* of each gate is owned by the lane's manager. This avoids "process gate" framing where demos exist as separate authoring work disconnected from lane closure.

## Program scope (T-Release)

R2's smallest cross-cutting deliverables + release coordination + demo + discipline-framework reporting + closure ledger.

| Deliverable | Source | Status (at brief authoring) |
|---|---|---|
| §6a per-method-metadata follow-through (Goal 5) | `docs/design-substrate-carrier-port-program.md §6a` (decision locked at `:171`; live receipt at `:173`; dissolution trigger at `:175`) | DECISION LOCKED — Option 3 unified `MethodContract` carrier; live receipt landed (`src/v3/std/algebra.dag` declares `MethodContract`; `src/v3/lenses/cost.dag` imports via `method_contract_cost_shape` minimal demo consumer). **Adjacent landing:** T-Cost-Dimension fail-closed symbolic-cost analysis (#1003) — DominateScanAcc conjunctive accumulator + dimension-evidence fail-closed semantics. Remaining R2 work: bulk migration of `cost.dag` / `complexity.dag` to live call-site lookup + dissolution-trigger tracking (`size_effect` / `cost_shape` / `callback_element_position` field-by-field retirement). |
| R2 closure demo coordination (Goal 6) | `docs/r2-structure.md` Goal 6 + structural-acceptance-per-lane-close discipline | LIVE on spawn (no separate brief; demo IS the structural acceptance gate per Director directive) |
| B-wave Tier 0 through-merge | #810 §5 dispatch ordering | **LANDED** — B1 [#820](https://github.com/gunb-ai/gunbc/pull/820), B2 [#817](https://github.com/gunb-ai/gunbc/pull/817), B3 [#821](https://github.com/gunb-ai/gunbc/pull/821) merged to `main` **2026-04-26 (UTC)** per GitHub merge metadata (fail-closed P3 leak fixes; receipts on those PRs). |
| B-wave Tier 2 (B5/B6/B7) | #810 §5 + [`debt-paydown-synthesis-2026-04-25.md` §"Tier 2" items 5–7](debt-paydown-synthesis-2026-04-25.md) | **LANDED in code/docs:** **B5** RESOLVED 2026-04-27 — structural gate `src/v3/compiler/tests/integration/r2_b5_loop_construction_closure_test.rs`; synthesis §5; [`r2-release-b5-loop-construction-closure-audit-worker.md`](r2-release-b5-loop-construction-closure-audit-worker.md) closed as receipt. **B6** RESOLVED 2026-04-26 — checklist + ROADMAP row per synthesis §6. **B7** RESOLVED 2026-04-27 — [#1014](https://github.com/gunb-ai/gunbc/pull/1014) per synthesis §7 (`patch_lower_helpers_*` retirement). Cross-manager brief: [`r2-release-b7-priority-hint-relay-to-pure-bootstrap.md`](r2-release-b7-priority-hint-relay-to-pure-bootstrap.md). |
| Discipline framework central reporting | INVARIANTS.md#p5-progress-is-dissolution "Dispatch-Discipline Mechanisms" (c) + ROADMAP.md "Integration-reflection cadence" velocity-tripwire | LIVE on spawn (mechanism wired via PRs #810 + #812) |
| Thesis-claim coverage mapping | `docs/r2-structure.md` Open call 1 + `docs/thesis/r2-r3-thesis-mapping.md` (LIVE post-#1078) | LANDED via #1078 — initial mapping complete; refresh as lanes close |
| R2 closure ledger | `docs/r2-structure.md` Manager structure section 7 | LIVE on spawn (tracks lane-close green status across all 6 other managers; sub-gate progress for T-LensProducer-Retirement R3 continuation) |
| v2 retirement coordination | `docs/r2-structure.md` Transition mechanics v2-retirement note | LIVE on spawn (**post-R3** operational per `docs/r2-structure.md` §"R2 Release Manager" — moved from post-R2 to post-R3 with the R3 structured-program reframe; tracked but not gated) |
| **v2 release-doc-authority guardrail follow-up** *(NEW 2026-04-28)* | `docs/r2-structure.md` §"v2 guardrail requirements" + `scripts/check-release-doc-authority.sh` | **NOT YET AUTHORED** — narrow PR scoped to release-state.yaml + projection-checker per gpt-5-5-pro meta-review; tighten retraction-pattern foot-gun (currently exempts unrelated "retracted" prose) |

## Owned deliverables (through R2 close)

### Core deliverables (must complete by R2 close)

1. **§6a per-method-metadata follow-through brief** — **Pick is closed.** Design-call decision is locked as Option 3 unified `MethodContract` carrier per [`docs/design-substrate-carrier-port-program.md §6a:171`](../design-substrate-carrier-port-program.md), with live receipt at `:173`: `src/v3/std/algebra.dag` declares `MethodContract`; `src/v3/lenses/cost.dag` imports it via `method_contract_cost_shape` minimal demo consumer. The pick-worker brief at [`docs/briefs/t-permethodmetadata-pick-worker.md`](t-permethodmetadata-pick-worker.md) (landed PR #794) authored the pick + lock + minimal demo and is **closed in scope** — its "Do not migrate all consumer lenses ... bulk migration is post-pick work" clause names the residual this manager owns. R2 follow-through deliverables (NOT a re-pick): (a) bulk migration of `cost.dag` / `complexity.dag` to live call-site `MethodContract` lookup; (b) track the field-by-field dissolution trigger from `:175` (`size_effect` → cardinality-refined signatures; `cost_shape` → typed cost surface; `callback_element_position` → typed higher-order callback parameter shape). No duplicate decision authority — pick is closed; follow-through is post-pick scope.
2. **B5 Loop construction-closure audit brief** — Tier 2 from #810 §5. **CLOSED (2026-04-27)** — closure-holds path: structural test + synthesis §5 RESOLVED + ROADMAP receipt; marker brief retired. See [`r2-release-b5-loop-construction-closure-audit-worker.md`](r2-release-b5-loop-construction-closure-audit-worker.md) (historical receipt).
3. **B6 file-preference rank checklist completion brief** — **CLOSED (2026-04-26)** — synthesis Tier 2 §6 + `declaration_name_preference_rank` / `collect_symbols` checklist extension per landed work.
4. **B7 priority-hint relay to Pure Bootstrap Manager** — **CLOSED (2026-04-27)** — retirement landed [#1014](https://github.com/gunb-ai/gunbc/pull/1014) per synthesis §7; cross-manager brief remains archival signal doc.
5. **Thesis-claim coverage mapping refresh** (Open call 1 from `docs/r2-structure.md`) — initial mapping landed via #1078 at `docs/thesis/r2-r3-thesis-mapping.md`; refresh as lanes close to track per-claim disposition.
6. **v2 release-doc-authority guardrail follow-up** *(NEW 2026-04-28)* — narrow PR scoped per gpt-5-5-pro meta-review: structured `release-state.yaml` + projection-checker that catches structured-state drift (lane counts, dependency-graph membership, thesis-claim disposition); tighten retraction-pattern foot-gun. Pinned-limitation test (`test_foot_gun_currently_allowed`) flips to a contract assertion when this lands.

### Standing-state deliverables (continuous through R2)

7. **B1, B2, B3 Tier 0 through-merge** — **complete at HEAD** — #820 / #817 / #821 merged **2026-04-26 (UTC)** per GitHub merge metadata. Standing: surface regressions only; no open Tier-0 coordination queue for this tranche.
8. **R2 demo coordination** — surface "it runs" artifacts at each lane close per **structural-acceptance-per-lane-close discipline** (the demo IS the structural gate; no separate authoring).
9. **Closure ledger** — track lane-close green status across all 6 other managers; surface unblocked work to idle workers; coordinate v2 retirement post-R3 (per `docs/r2-structure.md` §"R2 Release Manager" — v2 retirement moved from post-R2 to post-R3 with the R3 structured-program reframe); track sub-gate progress for T-LensProducer-Retirement R3 continuation per Director directive. **Live artifact: [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md)** — also carries the signal-receiver protocol (cross-manager queue channel; what counts as a receipt; cadence touchpoints with velocity-tripwire reporting).
10. **Velocity-tripwire reporting** — each integration-reflection cadence pass, report introduction:dissolution PR ratio across all manager-authored work; surface ≥3:1 readings to Director per INVARIANTS.md#p5-progress-is-dissolution (c). Manual sweep for dissolution-bearing feature PRs first per calibration caveat.
11. **Substrate Manager bottleneck watch** — per `docs/r2-structure.md` watch condition: if Substrate becomes the new bottleneck (workers idle >7 days waiting for Substrate-authored briefs), surface to Director and recommend B4 split into dedicated B4 Identity-Carrier Manager.

## Cross-program dependencies

**Produces:**
- Closure ledger updates → Director (weekly health check) + user (R2 demo surfacing).
- Velocity-tripwire alerts → Director (when ≥3:1 ratio fires).
- Bottleneck watch alerts → Director (when Substrate Manager workers idle >7 days, or any manager's workers idle >7 days).
- B7 priority-hint relay → Pure Bootstrap Manager.

**Consumes (lane-close signals from all 6 other managers):**
- Substrate Manager — T-Substrate sub-lane / B4 Phase / T-Substrate-Lens-Primitive close signals; **R3 continuation: T-CostLens-Composition** sub-gate progress
- Modeling Manager — T-Modeling item-close signals (4 items)
- Grounding Manager — T-Ground lane-close signals (11 lanes post-engine-reframe)
- Impossible-Bugs Manager — T-ImpossibleBugs class-close signals (3 classes)
- Pure Bootstrap Manager — post-R1 PB lane-close signals + **R3 continuation: T-LensProducer-Retirement (3 sub-gates) / T-FixedPoint / T-Tier3-Dissolution / 3 distributed bridge retirements**
- Evaluator Manager — T-Evaluator sub-lane close signals (5 sub-lanes); PR-A through PR-E design-lock cadence progress

**Adjacent territory:** none (Release is cross-cutting).

## Locked design decisions consumed (per #1078 dialogue)

Worker briefs MUST consume these without re-litigation:

- **Structural-acceptance-per-lane-close discipline**: demo IS the structural gate; no separate demo authoring work. Release coordinates visibility, doesn't author gates.
- **Manager count 7** (post-Evaluator-add): closure ledger spans 6 other managers (Substrate / Modeling / Grounding / Impossible-Bugs / Pure Bootstrap / Evaluator).
- **R2 manager continuation pattern** (`r3-structure.md` §"Manager structure"): Substrate continues with **T-Numeric-Construction (reframed 2026-05-01 from T-Int128)** + T-Anthropic-Wire + T-CostLens-Composition; PB continues with T-LensProducer-Retirement + T-FixedPoint + T-Tier3-Dissolution + T-V2-Retirement + 3 distributed bridges; Verification continues with T-V-L4-L7-Direct + T-V-L5-Corpus + T-Free-Consequences-Demonstration + bridge-ledger; Modeling/Impossible-Bugs archive at R2 close. Closure ledger reports R3 continuation readiness.
- **T-LensProducer-Retirement XL framing** (Director cascade Item 8): one program with 3 internal sub-gates; closure ledger reports sub-gate progress within the one-program lane.
- **T-Bridge-Retirement distribution map** (Director cascade Item 4): retirement work distributed (3 PB + 2 Substrate); ledger gate centralized at Verification (R3). Closure ledger tracks distribution map progress.

## Pre-spawn vs post-spawn authority

- **Pre-spawn (post-#1078-merge):** brief authoring + cross-cutting deliverables locked. Manager spawns once R2 promotion fires; pre-R1-close spawn allowed (no technical R1 dependency).
- **Post-spawn (R2 promotion onward):** Manager owns all worker-brief authoring autonomously per "Autonomous dispatch authority" below. Director's role narrows to cross-program conflict resolution + scope-change escalation.

## Autonomous dispatch authority

- Authors all T-Release owned deliverables without Director: worker briefs (§6a follow-through, thesis-claim coverage mapping refresh, v2 guardrail follow-up). Tier-2 B-wave worker briefs (B5/B6/B7) + Tier-0 through-merge are **closed at HEAD** (receipts: synthesis Tier 2 §5–7 + PRs #820/#817/#821/#1014); archival briefs remain for cross-manager traceability.
- Dispatches workers against owned deliverables.
- Owns the **central reporting** layer of P5 dispatch-discipline (per `docs/r2-structure.md`); per-brief paired-dispatch and per-PR gate enforcement happens at each manager's authoring point, not at Release.
- Resolves Release-internal scope refinements; escalates blockers and scope changes to Director.
- **Cross-program signal authority:** weekly closure-ledger summary → Director; R2 demo surfacing → user; B7 priority-hint relay → PB Manager; bottleneck watch alerts → Director.

## Reporting cadence

- **To Director (cross-program):** velocity-tripwire alerts (≥3:1 ratio), bottleneck watch alerts (any manager's workers idle >7 days), closure-ledger weekly health check, structural-acceptance gate landings.
- **To user (release coordination):** R2 demo artifacts at each lane close (per structural-acceptance-per-lane-close discipline); closure ledger surfacing.
- **From all 6 managers:** lane-close / sub-lane-close / class-close signals via cross-manager queue.
- **Blockers + scope changes:** Director.

## Acceptance — `.dag` gates

Release Manager doesn't author lane-level structural gates (those belong to lane-owning managers per the structural-acceptance-per-lane-close discipline). Release Manager's own acceptance:

- `r2_closure_ledger_landed` — closure ledger document tracking all 6 other managers' green-status / sub-gate-progress (post-spawn standing artifact). **Satisfied by [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md)** (LIVE; carries per-manager rows, T-LensProducer-Retirement 3-sub-gate decomposition, reserved R1-residual incoming surface, and the signal-receiver protocol).
- `velocity_tripwire_reporting_landed` — introduction:dissolution PR ratio surfaced to Director on integration-reflection cadence; ≥3:1 fires alert
- `thesis_claim_coverage_mapping_refresh_landed` — `docs/thesis/r2-r3-thesis-mapping.md` refreshed as lanes close; per-claim disposition tracked
- `release_doc_authority_v2_guardrail_landed` — structured release-state.yaml + projection-checker; foot-gun pinned-limitation test flips to contract assertion
- `r2_close_signal_to_director_authored` — when **all 6 other managers' R2-scope lanes are complete** (Substrate R2 sub-lanes incl. T-Substrate-Lens-Primitive + B4 / Modeling 4 items / Grounding 11 lanes / Impossible-Bugs 3 classes / Pure Bootstrap R2 lanes incl. Tier 3 mirror dissolutions + kernel_algebra_profile / Evaluator 5 sub-lanes), manager surfaces R2-close signal + R3 continuation readiness signal to Director. **Distinguishing R2-scope-complete from manager-archives:** Modeling and Impossible-Bugs Managers archive at R2 close; Substrate, PB, and Verification Managers continue into R3 with R3-scoped lanes (**T-Numeric-Construction** [reframed 2026-05-01 from T-Int128] / T-Anthropic-Wire / T-CostLens-Composition for Substrate; T-LensProducer-Retirement / T-FixedPoint / T-Tier3-Dissolution / T-V2-Retirement / 3 distributed bridges for PB; T-V-L4-L7-Direct / T-V-L5-Corpus / T-Free-Consequences-Demonstration + bridge-ledger for Verification). The R2 close gate fires on R2-scope completion, not on archive — PB's R2 lanes (Tier 3 mirror dissolutions, kernel_algebra_profile consumer plumbing) must be complete; R3 continuation lanes do not gate R2 close.

## Sub-briefs (authored / pending)

Authored (pre-spawn PM portion, inbox #828):
- [`r2-release-6a-follow-through-worker.md`](r2-release-6a-follow-through-worker.md) — §6a bulk migration + dissolution-trigger tracking
- [`r2-release-b5-loop-construction-closure-audit-worker.md`](r2-release-b5-loop-construction-closure-audit-worker.md)
- [`r2-release-b6-file-preference-rank-checklist-worker.md`](r2-release-b6-file-preference-rank-checklist-worker.md)
- [`r2-release-b7-priority-hint-relay-to-pure-bootstrap.md`](r2-release-b7-priority-hint-relay-to-pure-bootstrap.md) — cross-manager signal brief

Landed via #1078:
- Initial thesis-claim coverage mapping (`docs/thesis/r2-r3-thesis-mapping.md`)
- Release-doc authority consumer + self-test (`scripts/check-release-doc-authority.sh` + `scripts/test-check-release-doc-authority.sh` + CI wiring)

Pending — post-spawn manager-authored autonomously:
- **v2 release-doc-authority guardrail follow-up** worker brief (release-state.yaml + projection-checker; per gpt-5-5-pro meta-review)
- Thesis-claim coverage mapping **refresh** worker briefs (as lanes close — incremental refresh, not new authoring)
- Closure ledger weekly summary template (post-spawn standing artifact)

## Working state (fill on spawn)

Spawn refresh, 2026-04-28 (post-#1078, status-refresh against landed PRs):

- 7-manager structure live (count rose from 6 to 7 with Evaluator added).
- Closure ledger spans all 6 other managers (Substrate / Modeling / Grounding / Impossible-Bugs / PB / Evaluator) + sub-gate progress for T-LensProducer-Retirement (3 internal sub-gates per Director directive).
- Initial thesis-claim coverage mapping landed via #1078; refresh authority lives here.
- v2 guardrail follow-up named as next narrow PR (release-state.yaml + projection-checker).
- Structural-acceptance-per-lane-close discipline locked: Release coordinates visibility, doesn't author gates.

**Significant pre-spawn landings to track in initial closure-ledger snapshot:**
- **T-ImpossibleBugs:** all 3 main implementations landed (#890 + #962 follow-ups; #969 Int/Int totalization; #971 unenumerated effects lens landing).
- **T-Substrate:** ValueBody::Map (#1017 + #1068 tightening); NominalOpacity fail-closed enforcement (#937); B4.8 Phase-2 site dissolution (#1069); B4.2 first-consumer wiring landed.
- **T-Ground (per-target work pre-cadence):** Rust IntegerRangeFact mirror dissolved (#1005); Python primitives.dag (#1080); Go primitives tranche 1 + additional (`ac765ce10` + #1046).
- **T-PB-Runtime foundation:** ExecuteCommand typed-outcome hardening (#1049); T-PB-B boundary coverage (#1082).
- **Adjacent:** T-Cost-Dimension fail-closed analysis (#1003); SourceFiltering canonical authority (#1004); tokenizer charclass scanner-order retype (`242c65d07`).
- **B-wave Tier 0 status:** **LANDED** — #820 / #817 / #821 on `main`, merged **2026-04-26 (UTC)** (GitHub merge timestamps — not 2026-04-27). Release Manager brief text reconciled 2026-04-29. Tier 2 B5/B6/B7 items per synthesis §5–7 also landed (B5 closure narrative remains **2026-04-27** per synthesis Tier 2 §5). Remaining Release-owned work is §6a follow-through, v2 guardrail follow-up, and thesis refresh — not B-wave blind pass.

## Cross-refs

- Parent: `docs/r2-structure.md` §"R2 Release Manager"
- Discipline framework: INVARIANTS.md#p5-progress-is-dissolution "Dispatch-Discipline Mechanisms" (a)/(b)/(c)
- Cadence wiring: ROADMAP.md "Integration-reflection cadence" + velocity-tripwire reporting
- Structural-acceptance-per-lane-close discipline: `docs/r2-structure.md` (Director-locked 2026-04-28)
- §6a source: `docs/design-substrate-carrier-port-program.md §6a` (decision locked Option 3; live receipt landed)
- Thesis-claim authority: `THESIS.md §"Thesis claims — complete list"` + `docs/thesis/r2-r3-thesis-mapping.md`
- B-wave source: `docs/briefs/debt-paydown-synthesis-2026-04-25.md` §5 (B1-B7 dispatch ordering)
- Release-doc authority guardrail: `scripts/check-release-doc-authority.sh` + `scripts/test-check-release-doc-authority.sh`
- v2 guardrail requirements: `docs/r2-structure.md` §"v2 guardrail requirements"
- R2 manager continuation pattern: `docs/r3-structure.md` §"Manager structure"
- INVARIANTS substrate-fact-introduction procedure: `INVARIANTS.md` §P1
