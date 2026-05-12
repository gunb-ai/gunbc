# ctrl/ → .dag migration — project plan

**Status**: DRAFT (2026-05-12). Authored per operator directive 2026-05-12T18:30Z: "make a project plan for this... separate tree (beside the director) that models all dependencies out... migrate as much of ctrl/ onto dag as possible ASAP."

**Authority**: this is a Mgr-tier proposal for a NEW parallel program tree (parallel to gunbc R3-close Director `zesty-bear-812`). Operator ratifies + spawns Director session for the program; PM (`deep-wolf-155`) authored as project-tier proposal.

**Companion doc**: `docs/design-decomposition-algebra.md` — algebra scope; consume as substrate authority.

---

## §1. Mission

**Replace the existing dashboard with .dag code ASAP.** Per operator directive 2026-05-12T~19:05Z: this is not a "future authority" project; it's a direct replacement. ctrl/ TS dies when .dag emission proves out. Apply the gunbc thesis (lens-not-pass; cost-of-change = 1 file; coproduct dissolution at C-checkpoints; M9 DFS-the-concept-DAG-before-defining) to a real product surface.

**Compositional-modeling discipline**: the intent layer is THIN — "replace dashboard with .dag." The substrate work underneath is where the rigor goes. Per `MODELING.md` M9: every new type DFS-traces back to existing `dsl/std/` primitives; no parallel hierarchies; no opaque-string bridges; no heuristic passes. Per `feedback_lenses_not_passes.md`: lenses over physics, no heuristic enforcement.

**Why now**: today's 7 parser-class operator-tier-bypass surfaces in 8h (3 distinct fingerprints) validate the heuristic-pass-on-unstructured-text cost. The ctrl/ decomposition-algebra series (PRs #1192-#1197) is in flight and the right substrate to consume. Existing .dag precedents (`workflows/review.dag`, `research/.../inbox_delivery_slice.dag`) prove the approach.

**ASAP execution**: Phase 1 algebra substrate + Phase 1.5 subsystem modeling + Phase 3 emission targets all run **in parallel** as critical-path. No "deferred until Phase N" — emission targets land as fast as substrate, so per-subsystem cut-over (Phase 4) can fire as soon as the trio converges for that subsystem. Per `feedback_holistic_over_patches.md`: systematic fix, not patches.

---

## §2. Audit summary — ctrl/ subsystems

Inventoried from `gh api repos/gunb-ai/ctrl/contents/` (2026-05-12). Major subsystems:

### Already partially .dag-modeled
| Subsystem | Existing .dag | Status |
|---|---|---|
| PR review workflow | `workflows/review.dag` | Partial — pure helpers; awaits `uses` + extdep chain compile |
| Branch review config | `workflows/branch_review.dag` | Partial |
| Review config data | `workflows/review_config.dag` | Partial |
| DAG modeling proposal | `scripts/session-dashboard/DAG_MODELING_PROPOSAL.md` | Design-only doc |

### TS-only subsystems with documented designs (.md only)
| Subsystem | Documented at | Migration priority |
|---|---|---|
| Decomposition algebra | ctrl PRs #1192/#1193/#1195/#1197 | **HIGH** — substrate for everything else |
| PR-review verdict parser | (across api-reviewer + chatgpt-reviewer) | **HIGH** — today's pain |
| Session lifecycle | `scripts/session-dashboard/SESSION_LIFECYCLE.md` | HIGH — spawn/archive emergence |
| Inbox | `scripts/session-dashboard/INBOX_DESIGN.md` | MEDIUM |
| Dashboard messaging | `scripts/session-dashboard/CONTROL_PLANE_MESSAGE_AUDIT.md` + `control_plane_messages.mjs` | MEDIUM |
| Container lifecycle | `scripts/session-dashboard/CONTAINER_LIFECYCLE.md` + `container_runtime.mjs` | MEDIUM |
| Reviews API | `scripts/session-dashboard/REVIEWS_DESIGN.md` | HIGH (consumes verdict parser) |
| Review posting unification | `scripts/session-dashboard/REVIEW_POSTING_UNIFICATION_DESIGN.md` | MEDIUM |
| Scheduler resilience | `scripts/session-dashboard/SCHEDULER_RESILIENCE_DESIGN.md` | LOW |
| Sessions on remote node | `scripts/session-dashboard/SESSIONS_ON_REMOTE_NODE_DESIGN.md` | LOW |
| Provider path fallback | `scripts/session-dashboard/PROVIDER_PATH_FALLBACK_DESIGN.md` | LOW |
| Settings / API keys | `scripts/session-dashboard/SETTINGS_API_KEYS_DESIGN.md` | LOW |
| Test discipline | `scripts/session-dashboard/TEST_DISCIPLINE_DESIGN.md` | LOW |
| Chat reliability/architecture | 3 .md files | LOW |
| Analyses pipeline | `analyses_api.mjs` / `analyses_sync.mjs` / `analyses_table.mjs` | LOW |
| CI integration | `ci.mjs` | LOW |
| DAG API (work-items HTTP) | `dag_api.mjs` | **HIGH** — sibling of decomposition algebra |

### Existing top-level docs (constitutional)
`AGENTS.md` / `AUDIT.md` / `CODING.md` / `INVARIANTS.md` / `REVIEW_CONTROL_PLANE.md` / `SCOPE_PORTABILITY.md` / `TESTING.md` — these are project-level authority docs that the .dag substrate must respect.

---

## §3. What can migrate NOW — comprehensive subsystem catalog

**Strategy**: model **service contracts** (types + typed function signatures + pure helpers), NOT just types. Per `feedback_lenses_not_passes.md` + the existing demo `research/market/viability/demos/agent-ctrl-session-dashboard/inbox_delivery_slice.dag` (which proves: `service InboxDeliverySlice { fn ... }` shape is workable today even without emission).

**Authority claim correction (post-codex inline BLOCKING #4 2026-05-12T19:08Z)**: per INVARIANTS P2, **declarations alone are staging, not landed authority**. A merged `.dag` model is 🟡 STAGED, NOT 🟢 AUTHORITY. Authority requires a generated consumer or emission target that exercises the substrate — until then the .dag file is "proposed shape pending realization." This is consistent with the existing `feedback_no_textual_enforcement_bridges.md` discipline (textual claims of authority don't substitute for structural enforcement).

**What "staged" means concretely for a Phase 1.5 PR**:
- The `.dag` file lands as a structurally-checked, compiler-validated artifact (verifiable structure)
- It is NOT yet substrate authority for runtime behavior — the TS implementation in `ctrl/` remains authoritative until cut-over
- The dissolution trigger that retires the staged state and fires authority: **emission target lands + parity test passes + cut-over PR deletes the TS file** (the trio from §6 convergence)

The phased framing (Phase 1.5 + Phase 3 + Phase 4 in parallel) explicitly times the staged→authority transition per subsystem; no subsystem becomes authoritative until its full trio converges.

**Total audit** (2026-05-12 via gh API on `gunb-ai/ctrl`):
- 237 .mjs files in `scripts/session-dashboard/` organized into ~16 subsystems
- ~20 .mjs files in `scripts/chatgpt-reviewer/` (browser-based review automation)
- 3 .mjs files in `scripts/api-reviewer/` (CLI-based review backends)
- 29 design docs in `scripts/session-dashboard/*.md` (constitutional + per-subsystem)
- 3 existing partial .dag files in `workflows/` (review, branch_review, review_config)
- 1 existing demo .dag at `research/market/viability/demos/agent-ctrl-session-dashboard/inbox_delivery_slice.dag` — **promote to substrate** (≈90% already there)

**Staging discipline (post-codex Finding #4 2026-05-12T19:08Z)**: every "doable NOW" row below is **staged with an explicit dissolution trigger**, NOT authoritative-on-arrival. A landed `.dag` model is NOT yet substrate authority — the file is one half of the contract; the other half is a consumer that exercises it. Per Practice 4 dissolution-receipt discipline: every staged carrier has a named trigger that retires the staged state. For all Phase 1.5 modeling PRs the trigger is **"per-subsystem realization receipt + consumer parity"**: emission-target projection ✓ + parity-test ✓ + cut-over PR deletes TS file ✓. Until that trigger fires, the `.dag` file is 🟡 STAGED, not 🟢 AUTHORITY.

### Catalog (in priority order)

| # | Subsystem | Source files | TS LOC | Model now? | Notes |
|---|---|---|---|---|---|
| 1 | **Review verdict** | `findings_extract.mjs`, `findings_parser.mjs`, `findings_store.mjs`, `findings_triage.mjs`, `review_subprocess_runner.mjs`, `pr_feedback_format.mjs`, `api-reviewer/*` (3 files) | ~2000 | ✓ NOW | Today's pain; in flight per operator (Q-D) |
| 2 | **Decomposition algebra (work-item)** | `dag_api.mjs`, `lib/dag_writes.mjs`, `lib/dag_schema.mjs` (per ctrl PRs #1192-#1197) | ~800 | ✓ NOW + Phase 1 | Algebra substrate from companion §9 |
| 3 | **Inbox delivery** | `inbox_policies.mjs`, `inbox_schema.mjs` + `INBOX_DESIGN.md` | ~600 | ✓ NOW | **DEMO EXISTS** at research/.../inbox_delivery_slice.dag — promote |
| 4 | **Control plane messages** | `control_plane_messages.mjs`, `control_plane_inject.mjs`, `send_eligibility.mjs` + 3 `CONTROL_PLANE_*.md` | ~1500 | ✓ NOW | Dashboard-message routing + sender-marker discipline |
| 5 | **Session lifecycle** | `sessions_schema.mjs`, `watcher.mjs`, `container_runtime.mjs`, `runtime_tmux.mjs`, `runtime_helpers.mjs` + `SESSION_LIFECYCLE.md`, `CONTAINER_LIFECYCLE.md` | ~2500 | ✓ NOW | Spawn/idle/archive emergence |
| 6 | **Review pipeline (extended)** | extend existing `workflows/review.dag` + `review_scheduler.mjs`, `reviews_schema.mjs` + `REVIEWS_DESIGN.md`, `REVIEW_POSTING_UNIFICATION_DESIGN.md` | ~1800 | ✓ NOW | Promote existing .dag; consumes #1 |
| 7 | **Pools (capacity / billing / dispatch)** | `pools_api.mjs`, `pools_billing.mjs`, `pools_dispatch.mjs`, `pools_schedule.mjs`, `pools_schema.mjs`, `pools_validate.mjs`, `pools_writes.mjs` | ~2000 | ✓ NOW | Big subsystem; 7-file group |
| 8 | **PR digests** | `pr_attached_urls.mjs`, `pr_ci_digest.mjs`, `pr_conflict_digest.mjs`, `pr_merge_ready_digest.mjs`, `pr_rest_fallback.mjs` | ~1200 | ✓ NOW | Pure-function-heavy; easy candidates |
| 9 | **Scheduler** | `scheduler.mjs`, `review_scheduler.mjs` + `SCHEDULER_RESILIENCE_DESIGN.md` | ~1000 | ✓ NOW | Decision contract pure; trigger execution gated |
| 10 | **Work-advancement prompts** | `work_advancement_prompts.mjs` | ~400 | ✓ NOW | Template construction = pure functions |
| 11 | **Analyses pipeline** | `analyses_api.mjs`, `analyses_sync.mjs`, `analyses_sync_targets.mjs`, `analyses_table.mjs` | ~1500 | ✓ NOW | Sync + table queries |
| 12 | **CI integration** | `ci.mjs` | ~500 | ✓ NOW | Poll + gate decisions pure |
| 13 | **chatgpt-reviewer (browser automation)** | `scripts/chatgpt-reviewer/*` ~20 files | ~3000 | ◐ PARTIAL NOW | Browser automation = side-effectful; model CONTRACT today, execution deferred |
| 14 | **api-reviewer (CLI backends)** | `scripts/api-reviewer/*` 3 files | ~600 | ✓ NOW | Backend selection + invocation contract |
| 15 | **Server / HTTP routes** | `server.mjs` | ~2000 | ◐ PARTIAL NOW | Route TABLE doable today; handler bodies gated on Phase 3 HTTP emission |
| 16 | **Utility helpers** | `disk_pressure.mjs`, `effort_picker.mjs`, `parse_int_env.mjs`, `transcript_excerpt.mjs` | ~400 | ✓ NOW (fold) | Small; fold into consuming subsystems rather than standalone |

**Total TS LOC under audit**: ~21,800 lines across 16 subsystems. Realistic to model all 16 in 3-4 weeks of parallel worker dispatch (Phase 1.5).

### Independent NOW (no Phase 1 dependency)

Items **3, 5, 8, 10, 11, 12, 14, 16** can dispatch immediately as Phase 1.5 work without waiting for Phase 1 algebra substrate. ~8 parallel workers feasible.

Items **2, 4, 6, 7, 9, 13, 15** consume the algebra substrate or have non-trivial cross-dependencies; dispatch after Phase 1 lands or Phase 1.5 first-wave validates the pattern.

Item **1** (review verdict) is already in flight per operator directive.

### Subsystems that GATE on emission targets (Phase 3+)

| Concern | Why gated |
|---|---|
| HTTP route handlers | Need `dsl/extdeps/http/server.dag` for actual server replacement |
| SQL schema + migrations | Need `dsl/extdeps/sql/migration.dag` for migration emission |
| Audit event persistence | Need `dsl/extdeps/audit/event.dag` for event-log emission |
| Browser DOM automation | Need `dsl/extdeps/browser/dom.dag` for Puppeteer-class emission |
| Cron / launchd / hooks | OS-level scheduling — need shell/cron emission |
| Provider authentication | Auth + secret handling — separate Phase 3 concern |

Each is a distinct R4-class extdeps workstream. None are blocking for Phase 1.5 modeling.

---

## §4. Phases NOT doable NOW (gated)

**Phase 2 — Friendly CLI projection** (Rust binary):
- Requires Phase 1 substrate landed
- Adds `dashboard-ops` as Rust binary projection of `dsl/ctrl/cli_surface.dag`
- Replaces bash dashboard-ops over time

**Phase 3 — Emission targets** (gates Phase 4):
- `dsl/extdeps/http/server.dag` — HTTP REST handler carriers
- `dsl/extdeps/sql/migration.dag` — SQL schema emission
- `dsl/extdeps/audit/event.dag` — audit-event emission

Each is real R4-class substrate work, parallel to T-WAD's `dsl/extdeps/github/actions.dag`.

**Phase 4 — Ctrl/ cut-over**:
- Stop hand-maintaining `ctrl/lib/dag_writes.mjs`, `ctrl/scripts/session-dashboard/*.mjs`
- Generated emission replaces TS implementation
- Byte-identity / behavior-parity verification

**Phase 5 — Generalize**:
- Once first ctrl/ subsystem cuts over cleanly, others follow with no new emission-target work
- Each new subsystem migration = add types + projections in `dsl/ctrl/*.dag`

---

## §5. Program tree (Ctrl-Migration Director under PM/CEO)

**UPDATED per operator directive 2026-05-12T~19:20Z**: deep-wolf-155 (PM) operates at CEO-tier above the gunbc R3-close Director `zesty-bear-812`. Ctrl-Migration Director is **a child under deep-wolf-155**, not parallel-root. Spawn fired via `dashboard-ops work-items create` at `node://adhoc-dc298bc7-9f7` 2026-05-12T19:20:39Z.

```
operator (Brian)
└── deep-wolf-155 (CEO/PM, root session)
    ├── zesty-bear-812 (gunbc R3-close Director, operationally under PM)
    │   └── R3-close subtree (3 R3 Mgrs + workers)
    └── Ctrl-Migration Director (NEW child, just spawned at adhoc-dc298bc7-9f7)
        ├── Substrate Mgr            (Phase 1 — critical path)
        │   └── algebra substrate (`dsl/std/process_algebra.dag` + Attestation)
        ├── Subsystem-Modeling Mgr   (Phase 1.5 — critical path, 8-14 workers across waves)
        │   └── 16 subsystem .dag modeling PRs per §3 catalog
        ├── Emission-Targets Mgr     (Phase 3 — critical path, PARALLEL not deferred)
        │   └── HTTP extdeps → SQL extdeps → audit-event extdeps
        └── Verification Mgr         (Phase 4 — convergence + cut-over)
            └── parity tests; per-subsystem cut-over PRs; TS deletion
```

**4 Mgrs spawn on Day 1**, not 3 — Emission-Targets Mgr is NOT deferred. Phase 3 work starts in parallel with Phase 1 + 1.5.

**Auto-spawn mechanics**: per the dashboard ladder, `dashboard-ops work-items create` creates a work-item bound to the caller; the dashboard auto-spawns a child session within ~30s with the title as charter. The Director then runs its own `dashboard-ops work-items create` for each Mgr to populate the next tier.

**Decisions per ladder**:
- PM/CEO (deep-wolf-155): inter-program coordination between zesty-bear-812 (R3-close) and the new Ctrl-Migration Director; ratifies cross-program signals; surfaces structural conflicts to operator
- Director: scopes phases, ratifies cross-Mgr signals, surfaces to PM, enforces compositional-modeling discipline (M9 DFS before any new carrier)
- Substrate Mgr: own Phase 1 — single algebra substrate file + Practice 4 receipts; lands `dsl/std/process_algebra.dag`
- Subsystem-Modeling Mgr: own Phase 1.5 — dispatches 8 parallel workers Wave 1 (Day 2-5), 6 more Wave 2 (Day 6-10); each lands one subsystem `.dag` PR with service-contract shape
- Emission-Targets Mgr: own Phase 3 — HTTP/SQL/audit-event extdeps; **starts Day 3** authoring HTTP extdeps brief (parallel with Substrate Mgr Phase 1)
- Verification Mgr: own Phase 4 — byte-identity / behavior-parity tests; spawns cut-over PRs as each subsystem's trio converges (algebra ✓ + subsystem-modeled ✓ + emission-target-landed ✓)

**Coordination between programs**: PM (deep-wolf-155) bridges zesty-bear-812 (gunbc R3-close) and Ctrl-Migration Director. Cross-program substrate-shape conflicts route through PM. Each Director owns their program's scope independently; PM owns the inter-program interface.

**Compositional-modeling enforcement**: Director's standing brief discipline = every new carrier requires (1) M9 DFS trace to existing `dsl/std/` primitive showing why-not-reuse, (2) Practice 4 dissolution receipt for any open enum, (3) cost-of-change-1 verification (adding next variant touches 1 file), (4) cross-reference to source TS file being replaced. Per `feedback_grep_substrate_before_naming_ratification.md`: grep dsl/std/ + docs/audit/ before ratifying new carrier names.

---

## §6. Phase sequencing — parallel critical paths (revised per "replace ASAP")

**Old framing** (rejected): Phase 1 → Phase 3 → Phase 4 sequential. Treats emission as "deferred."

**New framing** (per operator 2026-05-12): Phase 1 + Phase 1.5 + Phase 3 **in parallel**, all critical-path. Per-subsystem cut-over (Phase 4) fires as soon as the three converge for that subsystem.

```
Phase 0 (DONE)
  ├── docs/design-decomposition-algebra.md  (PR #2775)
  ├── docs/r4-ctrl-dag-migration-project-plan.md  (PR #2775)
  └── ctrl audit complete (16 subsystems / ~21,800 TS LOC)
                            │
        ┌───────────────────┼───────────────────┐
        ↓                   ↓                   ↓
    Phase 1            Phase 1.5            Phase 3
  (algebra)         (subsystem            (emission
                     modeling)             targets)
        │                   │                   │
        │ Day 2-5:          │ Day 2-5:          │ Day 3-7:
        │ Substrate Mgr     │ Subsystem-Mgr     │ Emission-Mgr
        │ lands             │ dispatches 8      │ HTTP extdeps
        │ process_algebra   │ parallel workers  │ brief authored
        │                   │ (independent      │
        │                   │ items)            │
        ↓                   ↓                   ↓
   Phase 1 lands       Wave 1 PRs land    HTTP extdeps land
                            │                   │
                            ↓                   ↓
                       Day 6-10:           Day 8-14:
                       Subsystem-Mgr       Emission-Mgr
                       Wave 2 (algebra-    SQL + audit
                       consumers)          extdeps
                            │                   │
                       ─────┴───────────────────┴─────
                                    │
                                    ↓
                          Phase 4: per-subsystem cut-over
                            ↓ (as each subsystem's trio converges)
                          Phase 5: generalize / full replacement
```

**Critical-path** (all three lanes simultaneously):
1. **Algebra substrate** → enables: work_item / messaging / scheduler subsystem modeling
2. **Subsystem modeling** → enables: contracts that emission projects against
3. **Emission targets** → enables: actual TS replacement at runtime

**Convergence trigger for cut-over** (per subsystem): subsystem's `.dag` contract authored ✓ AND relevant emission target landed (HTTP for routes / SQL for schema / etc.) ✓ AND parity test passes ✓ → cut-over PR lands; TS file deleted.

**No "future authority" phase**: Q-G resolved per operator directive — substrate becomes authority THE MOMENT emission proves out for that subsystem. No co-authority window.

---

## §7. First-wave dispatch shape

Once operator spawns the Ctrl-Migration Director:

**Day 1**:
- Director ratifies project plan + companion scope doc
- Director spawns 3 Mgr sessions (Substrate / Subsystem-Modeling / Verification)
  - Emission-Targets Mgr deferred to Day-N when Phase 1 nears landing
- Substrate Mgr authors brief for `dsl/std/process_algebra.dag` Phase 1 substrate

**Day 2-5** (parallel) — first wave of Phase 1.5 (independent items, no Phase 1 dependency):
- Substrate Mgr's worker drafts Phase 1 substrate PR
- Subsystem-Modeling Mgr authors 8 first-wave briefs from §3 catalog rows:
  - Worker A: `dsl/ctrl/inbox.dag` (catalog #3 — **promote existing demo**)
  - Worker B: `dsl/ctrl/session_lifecycle.dag` (catalog #5)
  - Worker C: `dsl/ctrl/pr_digests.dag` (catalog #8 — pure-function-heavy, easy)
  - Worker D: `dsl/ctrl/work_prompts.dag` (catalog #10 — small)
  - Worker E: `dsl/ctrl/analyses.dag` (catalog #11)
  - Worker F: `dsl/ctrl/ci.dag` (catalog #12)
  - Worker G: `dsl/ctrl/api_reviewer.dag` (catalog #14)
  - Worker H: utility-helper consolidation (catalog #16 — fold into consumers)
- 8 workers dispatched in parallel; each lands one PR (bundled by subsystem per §11 Q-H)

**Day 6-10** — second wave (consumes Phase 1 algebra substrate):
- Phase 1 algebra substrate lands (`dsl/std/process_algebra.dag`)
- Dispatch next 6 subsystem briefs:
  - Worker I: `dsl/ctrl/work_item.dag` (catalog #2 — algebra-consumer)
  - Worker J: `dsl/ctrl/messaging.dag` (catalog #4 — control plane)
  - Worker K: `dsl/ctrl/review_pipeline.dag` (catalog #6 — extends `workflows/review.dag`)
  - Worker L: `dsl/ctrl/pools.dag` (catalog #7 — big 7-file group)
  - Worker M: `dsl/ctrl/scheduler.dag` (catalog #9)
  - Worker N: `dsl/ctrl/chatgpt_reviewer.dag` (catalog #13 — partial; contract today, browser-execution deferred)
- Review verdict (catalog #1) parallel-tracked under operator's existing text-parsing fix work; merges in as a co-authority once both stabilize.

**Day 11-14**:
- First + second wave PRs cycle reviews + land
- Verification Mgr framework brief (parity-test scaffolding for Phase 4)
- Emission-Targets Mgr spawned; Phase 3 design briefs author for HTTP/SQL/audit extdeps

**Day 15+**:
- Phase 3 emission target PRs land sequentially (HTTP first per §4 ordering)
- Phase 4 cut-over begins per-subsystem (review verdict first as proven minimum)

---

## §8. Per-subsystem brief shape (template)

Each Phase 1.5 worker brief should declare:

```
# Worker brief — <subsystem> .dag modeling

**Authority**: Ctrl-Migration project plan §3 item N; operator directive 2026-05-12.
**Parent**: Subsystem-Modeling Mgr lane; project Director.
**Closure predicate**: `dsl/ctrl/<subsystem>.dag` authored + 2-distinct-provider APPROVE.

## Output
`dsl/ctrl/<subsystem>.dag` (or `~/ctrl/workflows/<subsystem>.dag` per §11 Q-A)

## Scope
- Type-only modeling; no emission target yet
- Define carriers + closed/open enums + projections
- **Practice 4 dissolution receipts for every enum/sum with ≥2 variants** (including closed sums; per codex inline BLOCKING #5 — modeling-discipline requires 🟢/🟡/🔴 classification for every N≥2 coproduct, not just open enums). See acceptance gate #2 below for receipt-format details.
- Cite the corresponding ctrl/ TS file(s) as "current authority; this PR proposes STAGED substrate, becomes authority on Phase 4 cut-over trio convergence"

## Reference materials
- `docs/r4-ctrl-dag-migration-project-plan.md` §3 item N (this brief's parent)
- `docs/design-decomposition-algebra.md` (algebra substrate)
- Source TS files in ctrl/
- Existing design .md in scripts/session-dashboard/<DESIGN>.md if available

## Acceptance gates
1. Carriers + enums declared
2. **Practice 4 receipts on every enum/sum with ≥2 variants** (NOT just open enums — closed sums need dissolution analysis too, per codex Finding #5 2026-05-12T19:08Z). Each receipt names: (a) classification (🟢 TERMINAL / 🟡 STAGED / 🔴 NEEDS-DISSOLUTION), (b) dissolution pattern if non-terminal (fact-placement / variant-is-data / algebraic-form / dimensional), (c) trigger that fires dissolution.
3. Cross-references to current ctrl/ TS authority files (the consumers being modeled)
4. **Consumer receipt named**: identify the specific consumer (TS file or future emission target) whose parity/cut-over fires the "STAGED → AUTHORITY" trigger for this subsystem
5. Cost-of-change check: adding a new variant/operation touches 1 file
6. Doc-only — no emission code

## STOP / PING criteria
- STOP if substrate-shape question requires re-evaluation of algebra carriers — surface to Substrate Mgr
- STOP if subsystem semantics conflict with companion algebra — surface to project Director
- STOP if a closed sum/enum encountered with no clear dissolution pattern (TERMINAL classification unjustified) — surface to project Director for ratification
- PING project Director on PR-open
```

---

## §9. Risk register

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Phase 1 substrate shape questioned during Phase 1.5 modeling | MEDIUM | HIGH (rework) | Land Phase 1 before bulk Phase 1.5 dispatch; only items not consuming algebra start before |
| Cost-of-change verification fails on first new variant | MEDIUM | MEDIUM (signals modeling gap) | Write the cost-of-change test in Phase 1; treat as ratchet |
| Workflow-types dissolution (companion §4) requires deprecation of in-flight gunbc work | LOW | MEDIUM | Audit `dsl/gunbc/workflow/types.dag` consumers before Phase 1 starts |
| ctrl/ TS keeps evolving during migration (drift) | HIGH | MEDIUM | Verification Mgr maintains parity tests; flag drift as substantive review finding |
| Emission targets (HTTP/SQL/audit) require new gunbc extdeps work that's deeper than estimated | MEDIUM | HIGH (Phase 3 slips) | Audit `dsl/extdeps/` complexity baseline early; consider deferring SQL emission if too deep |
| Director-tier coordination conflict with zesty-bear-812 (R3-close) | LOW | LOW (orthogonal programs) | Both Directors report to operator; cross-tier coordination via operator |
| Practice 4 dissolution discipline drift in subsystem modeling | MEDIUM | MEDIUM | Every brief requires explicit Practice 4 receipt declaration |
| Review-verdict-parser migration validates substrate but doesn't prove broader emission | MEDIUM | LOW (intentional — small first) | Sequence: review-verdict first (proves approach), then decomp-algebra (proves emission stack) |

---

## §10. First-week concrete actions

**Operator (Brian) — Day 0**:
1. Review + ratify this project plan (PR #2775 merge)
2. Confirm remaining §11 Qs (B Director shape, C workflow-types dissolution, E ctrl PRs disposition, F cross-Director coord, H per-subsystem PR cadence)
3. **Spawn Ctrl-Migration Director session via dashboard** — one-time operator action (e.g. `dashboard-ops work-items create "Ctrl-Migration Director"` or analogous spawn-root-session command)

**Ctrl-Migration Director — Day 1**:
1. Ratify scope per operator directive (compositional-modeling discipline; M9 DFS enforcement)
2. Spawn **4 Mgrs simultaneously**: Substrate / Subsystem-Modeling / Emission-Targets / Verification
3. Dispatch Substrate Mgr to author Phase 1 brief AND Emission-Targets Mgr to author HTTP extdeps brief — both critical-path Day 2+

**Substrate Mgr — Day 2-5**:
1. Author Phase 1 brief: `dsl/std/process_algebra.dag` substrate (companion doc §9 skeleton)
2. Spawn worker; PR cycle; coordinate with Subsystem-Modeling Mgr on algebra-consumer briefs

**Subsystem-Modeling Mgr — Day 2-5 (parallel)**:
1. Author **8 first-wave briefs** for independent items (catalog #3, 5, 8, 10, 11, 12, 14, 16)
2. Spawn 8 workers in parallel
3. PR cycle

**Emission-Targets Mgr — Day 3-7 (parallel — NOT deferred)**:
1. Author HTTP extdeps brief: `dsl/extdeps/http/server.dag` carriers (route + handler + middleware shapes)
2. Spawn worker; PR cycle
3. Author SQL extdeps brief (Day 6+); audit-event extdeps brief (Day 8+)

**Verification Mgr — Day 5+ (parallel)**:
1. Author parity-test framework brief
2. Spawn worker to scaffold byte-identity test infrastructure
3. Author first per-subsystem cut-over brief when first subsystem's trio (algebra + modeling + emission) converges (~Day 10-14)

**Wave-2 dispatch (~Day 6-10)**:
- Subsystem-Modeling Mgr: 6 second-wave briefs (catalog #2, 4, 6, 7, 9, 13) for algebra-consuming items
- Emission-Targets Mgr: SQL + audit-event briefs in parallel
- Verification Mgr: parity test infrastructure landing

**Convergence cut-over (~Day 14+)**:
- First subsystem cut-over PR lands (TS file deleted; emission projection becomes authority)
- Per `feedback_parity_script_over_comment_reframe.md`: parity script with fail-close, not process discipline
- Subsequent cut-overs cascade as each subsystem's trio converges

---

## §11. Open questions for operator

**Q-A — RESOLVED (operator 2026-05-12T~18:55Z)**: `.dag` file placement is **gunbc-side**. `dsl/ctrl/*.dag` lives in gunbc as application/tool substrate (parallel to existing `dsl/gunbc/*` application-level types). Universal primitives go to `dsl/std/`. The 3 existing partial `workflows/*.dag` files in ctrl repo eventually migrate to gunbc-side too.

**Q-D — RESOLVED (operator 2026-05-12T~18:55Z)**: review-verdict-parser migration is **already in flight** per operator (text-parsing fix in progress). Phase 1.5 dispatch order is no longer the question; the operator directive is "audit ALL session-dashboard work + migrate real functionality ASAP." See §3 comprehensive catalog (16 subsystems).

---

### Remaining open questions

**Q-B: Director-tier session shape** — single Director or operator-acting-as-director?
- Single Director (proposed): one session owns the program; ratifies + delegates
- Operator-acting (alternative): operator directly spawns Mgrs; saves one tier
- Proposed: single Director given scope (16 subsystems = > 1 week program)

**Q-C: Workflow-types dissolution scope** (also in companion doc §4) — dissolve `dsl/gunbc/workflow/types.dag` into the decomp-algebra, or extend?
- Proposed: dissolve; existing types become structural projections over decomp-algebra
- Operator confirm — has substantial downstream impact on gunbc R3-close work

**Q-E: Existing ctrl PRs #1192-#1197 disposition** — let them land in TS (treat as the "current authority" the migration will eventually replace), or hold pending substrate landing?
- Proposed: let them land in TS; they are the current authority; the migration eventually projects-from-substrate to replace them
- Operator confirm — has implications for ctrl team velocity

**Q-F: Cross-Director coordination** — how do `zesty-bear-812` (gunbc R3-close) and the new Ctrl-Migration Director coordinate when their work touches the same substrate?
- Proposed: cross-tier coordination via operator-relay (operator routes signals between Directors); substrate-shape conflicts get operator ratification
- Operator confirm — sets the inter-program protocol

**Q-G — RESOLVED (operator 2026-05-12T~19:05Z)**: substrate becomes authority **immediately when emission proves out per subsystem**. No "future authority" / "co-authority" phases. Per `feedback_parallel_representation_debt.md`: when canonical source exists, consume it rather than scaffold. Cut-over PR deletes the TS file in the same PR that lands the emission projection.

**Q-H (new): Per-subsystem PR cadence** — bundle related subsystems (e.g., all 4 `findings_*` files → one PR) or one PR per .mjs file?
- Proposed: bundle by subsystem (catalog row) per `feedback_bundle_workstreams_per_pr.md`; ~16 PRs total for Phase 1.5

---

## §12. Cross-references

- `docs/design-decomposition-algebra.md` — companion scope doc; algebra substrate authority
- ctrl PRs `gunb-ai/ctrl#1192`, `gunb-ai/ctrl#1193`, `gunb-ai/ctrl#1195`, `gunb-ai/ctrl#1197` — current TS-side algebra authority
- `gunb-ai/ctrl/workflows/review.dag` — existing partial .dag in ctrl (PR-review workflow)
- `gunb-ai/ctrl/scripts/session-dashboard/DAG_MODELING_PROPOSAL.md` — existing design doc
- `dsl/gunbc/workflow/types.dag` — existing gunbc workflow types (dissolution scope per companion §4)
- `INVARIANTS.md` + `MODELING.md` — governing constraints
- Memory `feedback_lenses_not_passes.md` — substrate authority over heuristic pass

---

— Authored by deep-wolf-155 (PM) 2026-05-12 per operator directive for ctrl-migration project planning. Project Mgr-tier proposal; operator ratifies + spawns Director session for execution.
