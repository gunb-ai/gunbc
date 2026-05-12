# ctrl/ → .dag migration — project plan

**Status**: DRAFT (2026-05-12). Authored per operator directive 2026-05-12T18:30Z: "make a project plan for this... separate tree (beside the director) that models all dependencies out... migrate as much of ctrl/ onto dag as possible ASAP."

**Authority**: this is a Mgr-tier proposal for a NEW parallel program tree (parallel to gunbc R3-close Director `zesty-bear-812`). Operator ratifies + spawns Director session for the program; PM (`deep-wolf-155`) authored as project-tier proposal.

**Companion doc**: `docs/design-decomposition-algebra.md` — algebra scope; consume as substrate authority.

---

## §1. Mission

Migrate `ctrl/` (dashboard / planning / review-pipeline / session-tree / messaging) processes into `.dag` substrate so the .dag model is **single authority** for behavior. ctrl/ TS becomes a projected emission target. Apply the gunbc thesis (lens-not-pass; cost-of-change = 1 file; coproduct dissolution at C-checkpoints) to a real product surface.

**Why now**: today's 7 parser-class operator-tier-bypass surfaces in 8h (3 distinct fingerprints) validate the heuristic-pass-on-unstructured-text cost. The ctrl/ decomposition-algebra series (PRs #1192-#1197) is in flight and the right substrate to consume. ctrl/ already has some .dag (`workflows/review.dag`, `DAG_MODELING_PROPOSAL.md`), so this is an acceleration not a kickoff.

**ASAP discipline**: maximize Phase 1.5 parallel work (type-only modeling of ctrl/ subsystems in .dag, no emission targets required) so the substrate-tier authority lands across subsystems while Phase 3 emission-target work proceeds in parallel.

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

## §3. What can migrate NOW (Phase 1.5 catalog)

**Phase 1.5** = type-only .dag modeling of each ctrl/ subsystem, no emission targets required. Substrate-tier work; each PR is doc-only. Lands in `dsl/ctrl/*.dag` in gunbc OR `~/ctrl/workflows/*.dag` (TBD per §11 Q-A).

**Doable NOW, gated only on operator dispatch**:

1. **Review verdict** — `review_verdict.dag`: typed `ReviewVerdict = Approve | RequestChanges(findings) | Comment`; `ReviewEvent { provider, sha, verdict, timestamp }`; `latest_per_provider_at_HEAD` lens. Replaces today's text-scrape parser. **First migration target** (small + self-validating).

2. **Decomposition algebra (consumer-side)** — `dsl/std/process_algebra.dag` + `dsl/ctrl/work_item.dag`. Phase 1 substrate from companion scope doc §9. Lands `Mode` open enum + `Operation` closed sum + `EventLog<T>` primitive + `canCloseNode` projection. Authoritative for ctrl PRs #1192-#1197 semantics.

3. **DAG API (work-item HTTP layer)** — `work_item_api.dag`: `POST /api/internal-work-items` shape, idempotency-key, parent-binding, mode-flip. Mirrors PR #1193's `dag_writes.mjs` helpers.

4. **Session lifecycle** — `session_lifecycle.dag`: spawn / idle / working / done / archived states; auto-archive grace; manager-grace-elapsed reason; remote-node vs local. Consumes decomposition algebra (a session IS a node in the work-item graph).

5. **Inbox** — `inbox.dag`: message-routing primitive; recipient discrimination; identity-inbox vs session-inbox. Per `feedback_auto_spawn_creates_separate_inbox.md`.

6. **Dashboard messaging** — `messaging.dag`: `Message { sender_session_id, recipient, body, priority, created_at, delivered_at }`; sender-marker discipline (`— sent from <session-id>` footer); HTTP 22 fallback. Per `feedback_sent_from_marker_on_pr_replies.md` + `feedback_dashboard_message_backtick_escape.md`.

7. **Reviews API + posting** — `reviews.dag`: review pipeline; provider/sha/verdict tally; merge_criteria projection. Consumes (1) review_verdict.

8. **PR-review workflow extension** — extend existing `workflows/review.dag` with new substrate; promote pure-helpers to typed projections; add `uses` once compiler supports it (or work around for now per the existing comment).

**Parallelism**: items 1, 3, 4, 5, 6 are independent — can dispatch 5 workers in parallel. Item 2 is the algebra substrate (Phase 1 from companion doc). Item 7 consumes item 1. Item 8 consumes item 2 + item 7.

**Effort estimate per item**: 1-2 days of worker time for the modeling pass (no implementation; doc + types + projections). Roughly 1 PR per item.

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

## §5. Parallel program tree

**Proposed org**:

```
operator (Brian)
└── Ctrl-Migration Director  ← NEW root, parallel to zesty-bear-812 (gunbc R3-close Director)
    ├── Substrate Mgr
    │   └── workers: Phase 1 algebra substrate, EventLog primitive, Lens type, Witness
    ├── Subsystem-Modeling Mgr
    │   └── workers: Phase 1.5 catalog items (review_verdict, work_item, session_lifecycle, inbox, messaging, reviews, ...)
    ├── Emission-Targets Mgr
    │   └── workers: HTTP extdeps, SQL extdeps, audit-event extdeps (Phase 3)
    └── Verification Mgr
        └── workers: parity tests + byte-identity gates (Phase 4)
```

**Decisions per ladder**:
- Director: scopes phases, ratifies cross-Mgr signals, surfaces to operator
- Substrate Mgr: own Phase 1 substrate landing (single substrate file + practice 4 receipts)
- Subsystem-Modeling Mgr: own Phase 1.5 — dispatches 5-8 parallel workers; each lands one subsystem .dag modeling PR
- Emission-Targets Mgr: own Phase 3 — HTTP/SQL/audit extdeps (sequential or parallel TBD)
- Verification Mgr: own Phase 4 — byte-identity tests; consumes emission outputs

**Why parallel to zesty-bear-812 not under it**: gunbc R3-close is its own program (substrate-prereq + Cost-Dim + affected-set lens + workflow-as-data). Ctrl migration is orthogonal (consumes gunbc substrate but doesn't gate R3-close gates). Two parallel Director-tier programs with operator at root.

---

## §6. Phase sequencing + dependency graph

```
Phase 0 (DONE today)
  - docs/design-decomposition-algebra.md (PR #2775)
  - this project plan
  - ctrl audit complete
       ↓
Phase 1 (~1 week)
  - Substrate Mgr lands dsl/std/process_algebra.dag
  - Phase 1.5 starts in parallel
       ↓
Phase 1.5 (~1-3 weeks parallel; depends on Phase 1 for items consuming algebra)
  - 5-8 subsystem modeling PRs, doc-only
  - Items 1, 3, 4, 5, 6 from §3 can start NOW (don't need Phase 1)
  - Items 2, 7, 8 wait for Phase 1
       ↓
Phase 2 (~1-2 weeks, parallel with Phase 3 if Mgrs available)
  - CLI projection to Rust binary
       ↓
Phase 3 (multi-week)
  - HTTP / SQL / audit-event extdeps
       ↓
Phase 4 (multi-week)
  - Ctrl/ cut-over per subsystem
       ↓
Phase 5 (open-ended)
  - Generalize to additional subsystems; eventual full ctrl/ in .dag
```

**Critical-path**: Phase 1 → Phase 3 → Phase 4. Phase 1.5 is parallel-fast-track. Phase 2 is parallel.

---

## §7. First-wave dispatch shape

Once operator spawns the Ctrl-Migration Director:

**Day 1**:
- Director ratifies project plan + companion scope doc
- Director spawns 3 Mgr sessions (Substrate / Subsystem-Modeling / Verification)
  - Emission-Targets Mgr deferred to Day-N when Phase 1 nears landing
- Substrate Mgr authors brief for `dsl/std/process_algebra.dag` Phase 1 substrate

**Day 2-5** (parallel):
- Substrate Mgr's worker drafts Phase 1 substrate PR
- Subsystem-Modeling Mgr authors first-wave briefs:
  - Worker A: `review_verdict.dag` (item 1 from §3 — first migration target)
  - Worker B: `work_item_api.dag` (item 3)
  - Worker C: `session_lifecycle.dag` (item 4)
  - Worker D: `inbox.dag` (item 5)
  - Worker E: `messaging.dag` (item 6)
- 5 workers dispatched in parallel; each lands one PR

**Day 6-14**:
- First-wave PRs cycle reviews + land
- Phase 1 substrate lands
- Second-wave items (2, 7, 8 from §3) dispatch
- Emission-Targets Mgr spawned; Phase 3 design briefs author

**Day 15+**:
- Phase 3 emission target PRs land sequentially
- Phase 4 cut-over begins per-subsystem

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
- Practice 4 dissolution receipts for any open enum
- Cite the corresponding ctrl/ TS file(s) as "current authority; this PR proposes substrate authority post-Phase 4"

## Reference materials
- `docs/r4-ctrl-dag-migration-project-plan.md` §3 item N (this brief's parent)
- `docs/design-decomposition-algebra.md` (algebra substrate)
- Source TS files in ctrl/
- Existing design .md in scripts/session-dashboard/<DESIGN>.md if available

## Acceptance gates
1. Carriers + enums declared
2. Practice 4 receipts on any open enum
3. Cross-references to current ctrl/ TS authority
4. Cost-of-change check: adding a new variant/operation touches 1 file
5. Doc-only — no emission code

## STOP / PING criteria
- STOP if substrate-shape question requires re-evaluation of algebra carriers — surface to Substrate Mgr
- STOP if subsystem semantics conflict with companion algebra — surface to project Director
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
1. Review + ratify this project plan
2. Decide §11 open Qs (especially A: where do `dsl/ctrl/*.dag` files live — gunbc or `~/ctrl`)
3. Spawn Ctrl-Migration Director session via dashboard (one-time operator action)

**Ctrl-Migration Director — Day 1**:
1. Ratify scope per operator directive
2. Spawn 3 Mgrs: Substrate / Subsystem-Modeling / Verification
3. Dispatch Substrate Mgr to author Phase 1 brief

**Substrate Mgr — Day 2-3**:
1. Author Phase 1 brief: `dsl/std/process_algebra.dag` substrate (companion doc §9 skeleton)
2. Spawn worker; PR cycle

**Subsystem-Modeling Mgr — Day 2-5 (parallel)**:
1. Author 5 first-wave briefs (review_verdict, work_item_api, session_lifecycle, inbox, messaging)
2. Spawn 5 workers in parallel
3. PR cycle

**Verification Mgr — Day 4+ (when first PRs land)**:
1. Author parity-test framework brief
2. Set up byte-identity tests for emission targets (when Phase 3 begins)

---

## §11. Open questions for operator

**Q-A: `.dag` file placement for ctrl-domain models** — gunbc-side (`dsl/ctrl/*.dag` in gunbc repo) or ctrl-side (`~/ctrl/workflows/*.dag` in ctrl repo)?
- Pros gunbc-side: single substrate authority in one place; gunbc compiler emits ctrl/ artifacts; centralizes the .dag toolchain
- Pros ctrl-side: keeps ctrl/ self-contained; respects existing `workflows/*.dag` precedent; ctrl team owns their substrate
- Proposed: gunbc-side for universal primitives (`dsl/std/process_algebra.dag`); ctrl-side for application-specific (`~/ctrl/workflows/<subsystem>.dag`), extending existing `workflows/review.dag` precedent

**Q-B: Director-tier session shape** — single Director or operator-acting-as-director?
- Single Director (proposed): one session owns the program; ratifies + delegates
- Operator-acting (alternative): operator directly spawns Mgrs; saves one tier
- Proposed: single Director if program runs > 1 week; operator-acting if scope tightens

**Q-C: Workflow-types dissolution scope** (also in companion doc §4 + §11.1) — dissolve `dsl/gunbc/workflow/types.dag` into the decomp-algebra, or extend?
- Proposed: dissolve; existing types become structural projections over decomp-algebra
- Operator confirm — has substantial downstream impact

**Q-D: First migration target** — review-verdict-parser first (small, today's pain) or decomposition-algebra-itself first (foundational, in-flight)?
- Proposed: review-verdict-parser first (proves the approach), decomp-algebra second (replaces foundational TS)
- Operator confirm — affects worker dispatch order

**Q-E: Existing ctrl PRs #1192-#1197 disposition** — let them land in TS (treat as the "current authority" the migration will eventually replace), or hold pending substrate landing?
- Proposed: let them land in TS; they are the current authority; the migration eventually projects-from-substrate to replace them
- Operator confirm — has implications for ctrl team velocity

**Q-F: Cross-Director coordination** — how do `zesty-bear-812` (gunbc R3-close) and the new Ctrl-Migration Director coordinate when their work touches the same substrate?
- Proposed: cross-tier coordination via operator-relay (operator routes signals between Directors); substrate-shape conflicts get operator ratification
- Operator confirm — sets the inter-program protocol

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
