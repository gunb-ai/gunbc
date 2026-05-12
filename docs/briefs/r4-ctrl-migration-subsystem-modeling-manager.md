# R4 Ctrl-Migration — Subsystem-Modeling Manager Brief

**Status**: `PROPOSAL` (formal) → `ACTIVE` (Director-discretionary, pending [PR #2775](https://github.com/gunb-ai/gunbc/pull/2775) merge).

This manager owns **Phase 1.5** of the `ctrl/` → `.dag` migration program — the parallel-critical-path subsystem modeling lane (16 subsystems, ~21,800 TS LOC under audit). The program tree, audit catalog, and phase sequencing are authored at [`docs/r4-ctrl-dag-migration-project-plan.md`](../r4-ctrl-dag-migration-project-plan.md) (landing via PR #2775); this brief operationalizes §3 / §6 / §7 / §8 of that plan as the Mgr-tier standing program.

**Parent**: Ctrl-Migration Director (session `clever-ant-97`, codex-provider). Inter-program coordination (with gunbc R3-close Director `zesty-bear-812`) routes through PM/CEO (`deep-wolf-155`).

**Companion authority**: [`docs/design-decomposition-algebra.md`](../design-decomposition-algebra.md) — algebra substrate; primitive-mapping table; 4 modeling gaps. Phase 1 (Substrate Mgr lane, sibling to this one) lands `dsl/std/process_algebra.dag` consuming this companion.

---

## Orient before reading

- **Project plan authority**: [`docs/r4-ctrl-dag-migration-project-plan.md`](../r4-ctrl-dag-migration-project-plan.md), especially §3 (catalog of 16 subsystems with priority/dependency table), §6 (parallel-critical-path sequencing with staged-debt throttle), §7 (Wave-1 / Wave-2 dispatch shape), §8 (per-worker brief template), §9 (risk register). All landing via PR #2775; treat the PR diff as authority until merge.
- **Companion algebra scope**: [`docs/design-decomposition-algebra.md`](../design-decomposition-algebra.md) — Phase 1 substrate this lane's algebra-consumer subsystems block on.
- **Existing precedents in this repo**: `dsl/extdeps/*.dag` for service-contract shape (e.g. [`dsl/extdeps/cron.dag`](../../dsl/extdeps/cron.dag), [`dsl/extdeps/github/pulls.dag`](../../dsl/extdeps/github/pulls.dag)); `dsl/std/*.dag` for pure-type modeling without transport.
- **Governance**: [`INVARIANTS.md`](../../INVARIANTS.md) (especially **P2**: declarations alone are staging, not landed authority — the `🟡 STAGED` vs `🟢 AUTHORITY` distinction is load-bearing for every PR this lane lands), [`MODELING.md`](../../MODELING.md) **M9** (DFS the concept DAG before defining new types).
- **Cross-role discipline carry-overs from MEMORY.md** that apply to every dispatch this Mgr fires (paste into worker briefs verbatim where applicable):
  - [feedback_self_hosting_md_authority_audit_before_naming.md] — grep `src/v3/SELF_HOSTING.md` (and ctrl's authority docs) before declaring sum/struct names; collisions force costly rename cascades.
  - [feedback_extdeps_header_discriminator_before_field_placement.md] — gunbc emission/policy facts must not live on extdeps platform carriers (INVARIANTS P1).
  - [feedback_pattern_a_scaffold_sentinel_per_instance_ratification.md] — every scaffold-with-sentinel landing needs Director ratification; Mgr does not batch-ratify.
  - [feedback_grep_substrate_before_naming_ratification.md] — grep `dsl/std/` + `docs/audit/` before ratifying any new carrier name in a worker brief.
  - [feedback_canvas_two_axis_verification.md] — every canvas finding needs BOTH substrate-precedent AND consumer-side grep before Mgr ratification.
  - [feedback_one_canonical_subissue_per_workitem.md] — one canonical sub-issue per subsystem; no omnibus + per-instance dual-authoring (the 16-subsystem catalog must NOT spawn an omnibus parent).
  - [feedback_substrate_plumbing_receipt_naming.md] — when a worker lands a placeholder/non-canonical fixture or receipt, the test name itself must encode the scope (not just a doc-string).

---

## Slice

This manager owns Phase 1.5 of the program — 16 subsystem `.dag` modeling PRs across two dispatch waves, parallel-critical-path with Substrate Mgr (Phase 1) and Emission-Targets Mgr (Phase 3).

**Wave 1 (Day 2–5, ~8 parallel workers, no Phase 1 dependency)** — catalog rows that don't consume the algebra substrate:

| # | Subsystem | Worker brief | Notes |
|---|---|---|---|
| 3 | Inbox delivery | `r4-ctrl-migration-inbox-worker.md` | **Promote existing demo** at `research/.../inbox_delivery_slice.dag` (≈90% there per plan §3) |
| 5 | Session lifecycle | `r4-ctrl-migration-session-lifecycle-worker.md` | spawn/idle/archive emergence; consumes `SESSION_LIFECYCLE.md` + `CONTAINER_LIFECYCLE.md` |
| 8 | PR digests | `r4-ctrl-migration-pr-digests-worker.md` | Pure-function-heavy; smallest first-wave item; recommended Wave-1 trio anchor |
| 10 | Work-advancement prompts | `r4-ctrl-migration-work-prompts-worker.md` | Template-construction pure functions; ~400 LOC |
| 11 | Analyses pipeline | `r4-ctrl-migration-analyses-worker.md` | Sync + table queries |
| 12 | CI integration | `r4-ctrl-migration-ci-worker.md` | Poll + gate decisions pure |
| 14 | api-reviewer (CLI backends) | `r4-ctrl-migration-api-reviewer-worker.md` | Backend selection + invocation contract |
| 16 | Utility helpers | `r4-ctrl-migration-utility-fold-worker.md` | Fold into consuming subsystems; no standalone file |

**Wave 2 (Day 6–10, ~6 workers, algebra-consumers — gated on Phase 1)**:

| # | Subsystem | Notes |
|---|---|---|
| 2 | Decomposition algebra (work-item) | First algebra-consumer; mirrors ctrl PRs #1192–#1197 |
| 4 | Control-plane messages | Dashboard-message routing + sender-marker discipline |
| 6 | Review pipeline (extended) | Promotes existing `ctrl/workflows/review.dag` |
| 7 | Pools (7-file group) | Largest single subsystem |
| 9 | Scheduler | Decision contract pure; trigger execution gated |
| 13 | chatgpt-reviewer (browser) | PARTIAL — contract today, execution deferred to Phase 3 browser extdeps |

**Excluded from this lane**:
- Catalog #1 (review verdict) — parallel-tracked under operator's existing text-parsing fix work per plan §7 Day-6–10 note.
- Catalog #15 (server / HTTP routes) — route table doable as a Wave-2-late item; handler bodies gate on Phase 3 HTTP emission target (Emission-Targets Mgr lane).

---

## Wave-1-trio checkpoint (load-bearing throttle)

Per plan §6 / §7 (incorporating claude review #10327 exploratory observation 2026-05-12T19:22Z):

**Before any Wave-2 dispatch fires, this Mgr requires at least one full trio convergence on a Wave-1 subsystem**: algebra ✓ (Phase 1 substrate landed) **+** Phase 1.5 modeling PR ✓ (this lane) **+** Phase 3 emission target ✓ (Emission-Targets Mgr lane) **+** parity test passing.

**Recommended trio anchor**: catalog #8 (PR digests). Reason: pure-function-heavy, no SQL/HTTP runtime side-effects, the corresponding emission target is the smallest possible Phase 3 deliverable (a render-helpers extdep, essentially zero new external authority).

If the Wave-1 trio fails to converge by Day 10, this Mgr **pauses Wave-2 dispatch and surfaces to the Director for re-scope** — do not paper over a structural convergence failure by dispatching more staged-debt.

---

## Staged-debt budget (Verification Mgr throttle interface)

Per plan §6 staged-debt-throttle: if **3 or more Phase 1.5 PRs have merged with no matching Phase 3 emission target landed**, this Mgr pauses new dispatch (Wave-2 entries onward) and routes the saturated subsystems to the Emission-Targets Mgr for prioritization. Verification Mgr tracks the receipt-trail; this Mgr enforces the dispatch gate.

The budget is **not a soft signal** — it is the structural answer to "parallel ≠ independent." Wave-1 PRs landing without their Phase-3 partner is acceptable (single trio converging); a fleet of 8 unmatched stagings is not.

---

## Per-worker brief template (operationalizes plan §8)

Every Wave-1 / Wave-2 worker brief this Mgr authors MUST declare:

1. **Authority anchor**: cite project plan §3 row, operator directive 2026-05-12, and any companion design-doc in `ctrl/scripts/session-dashboard/*.md` (these are the consumer-side references — workers won't be able to read `ctrl/` source from the gunbc worktree; design docs are the proxy).
2. **Closure predicate**: `dsl/ctrl/<subsystem>.dag` authored + 2-distinct-provider APPROVE.
3. **Output path discipline**: subsystem `.dag` files land at `dsl/ctrl/<subsystem>.dag` (gunbc-internal; resolves Q-A pending operator ratification — if operator instead routes them to `~/ctrl/workflows/`, brief updates fire from this Mgr, not from workers).
4. **Scope (type-only)**:
   - Carriers + closed/open enums + projections.
   - Practice-4 dissolution receipts for **every** enum/sum with ≥2 variants (not just open enums — per codex review #10331 finding #5: closed sums require classification too). Each receipt names: (a) `🟢 TERMINAL` / `🟡 STAGED` / `🔴 NEEDS-DISSOLUTION` classification, (b) dissolution pattern if non-terminal (fact-placement / variant-is-data / algebraic-form / dimensional), (c) trigger that fires dissolution.
   - Cross-references to ctrl/ TS authority files being modeled.
   - **Consumer receipt named**: the specific consumer (TS file or future emission target) whose parity / cut-over fires the `🟡 STAGED → 🟢 AUTHORITY` trigger.
   - Cost-of-change check: adding a new variant/operation touches 1 file.
   - Doc-only — no emission code in Phase 1.5 worker PRs.
5. **STOP/PING criteria** (verbatim from plan §8):
   - STOP if substrate-shape question requires re-evaluation of algebra carriers → Substrate Mgr.
   - STOP if subsystem semantics conflict with companion algebra → project Director.
   - STOP if a closed sum/enum encountered with no clear dissolution pattern (TERMINAL unjustified) → project Director ratification.
   - PING project Director on PR-open.

---

## Working state

- **2026-05-12**: Mgr session `merry-newt-448` spawned at `node://adhoc-5d3bbf79-ce5`. PR #2775 (project plan + companion algebra doc) OPEN, not yet merged — Mgr operates against the PR diff as authority pending merge. No Wave-1 worker dispatched yet; this brief is the first Mgr-tier deliverable. Wave-1 trio anchor proposed: catalog #8 (PR digests) per "Wave-1-trio checkpoint" §.

- **Open coordination**: PM-tier ack sent to parent (`clever-ant-97`) 2026-05-12T19:39Z requesting (Q1) Mgr-PR deliverable shape and (Q2) wait-on-#2775 disposition. Mgr default if no reply ~30min: brief-authoring + scaffold (this file); workers dispatched post-#2775 merge. Worker briefs for Wave-1 catalog rows will land in follow-up PRs from this session before child-worker dispatch.
