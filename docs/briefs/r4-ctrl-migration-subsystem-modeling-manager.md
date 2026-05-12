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

**Before any Wave-2 dispatch fires, this Mgr requires at least one full trio convergence on a Wave-1 subsystem**: algebra ✓ (Phase 1 substrate landed) **+** Phase 1.5 modeling PR ✓ (this lane) **+** Phase 3 emission target ✓ (Emission-Targets Mgr lane) **+** parity test passing. Trio convergence is a **gating precondition for Phase 4 cut-over dispatch**, NOT itself the `STAGED → AUTHORITY` flip — the authority flip is the single ctrl-side cut-over PR that deletes the TS files (per INVARIANTS P2/P5 single-trigger discipline; see [`dsl/ctrl/README.md`](../../dsl/ctrl/README.md) §"Authority").

**Recommended trio anchor**: catalog #8 (PR digests). Reason: pure-function-heavy, no SQL/HTTP runtime side-effects, the corresponding Phase 3 deliverable is the smallest possible — a **gunbc-owned render projection** over `dsl/std/render.dag` (proposed `dsl/gunbc/digest_render.dag`), consuming GitHub source facts already in `dsl/extdeps/github/pulls.dag`. Per INVARIANTS P1 + `feedback_extdeps_header_discriminator_before_field_placement.md`: extdeps own third-party source facts; rendering/projection is gunbc-owned. The trio's Phase-3 emission is NOT an extdep landing.

If the Wave-1 trio fails to converge by Day 10, this Mgr **pauses Wave-2 dispatch and surfaces to the Director for re-scope** — do not paper over a structural convergence failure by dispatching more staged-debt.

---

## Staged-debt budget (Verification Mgr throttle interface)

The throttle predicate is **defined by the receipt-trail ledger**, not restated here, to keep the predicate single-sourced (per operator review codex finding #2 2026-05-12 commit `a6bd5f56`). See [`docs/audit/r4-ctrl-phase15-subsystem-receipt-trail.md`](../audit/r4-ctrl-phase15-subsystem-receipt-trail.md) §"Column semantics":

- **`open_receipt_debt`** is the ledger's derived column: `phase15_pr_merged ∧ ¬(phase3_emission_landed ∧ parity_passed)`. It does NOT reference `algebra_landed` (per ledger `N/A` semantics).
- **Dispatch-pause gate** is the ledger's stated condition: `count(rows where open_receipt_debt = true) ≥ 3` ⇒ pause new dispatch.

This Mgr **enforces** the gate by polling the ledger before authoring any new worker brief or spawning any new worker; Verification Mgr (`deep-badger-38`) **owns** the column semantics and the parity-flip evidence. If the predicate needs adjustment, the change lands in the ledger first; this brief defers verbatim.

Operational meaning: the budget is **not a soft signal** — it is the structural answer to "parallel ≠ independent." A Phase 1.5 PR landing without **both** its Phase-3 emission target landed **and** the named parity-harness gate green is acceptable in isolation (single trio converging); a fleet of unmatched stagings (≥3 with `open_receipt_debt = true` per the ledger predicate above, which jointly requires `phase3_emission_landed ∧ parity_passed` to clear the debt) is not.

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

- **2026-05-12**: Mgr session `merry-newt-448` spawned at `node://adhoc-5d3bbf79-ce5`. PR #2775 (project plan + companion algebra doc) OPEN, not yet merged — Mgr operates against the PR diff as authority pending merge. Director-ratified Mgr-PR shape Q1(a) brief-first (no thin-exemplar) via `msg_c1daa5ae-2dfd-4e0c-a4bd-cdec024e383a` 2026-05-12; Q2(b) DRAFT-until-#2775 confirmed; Director also confirmed Wave-1-trio + 3-unmatched-stagings throttle (already encoded in §"Wave-1-trio checkpoint" and §"Staged-debt budget").

- **Artifacts landed in PR #2777 (this PR)**:
  - this Mgr standing brief
  - [`dsl/ctrl/README.md`](../../dsl/ctrl/README.md) — scaffold + path conventions + receipt-format pointer (so worker briefs reference a stable consumer-receipt header shape)
  - [`docs/briefs/r4-ctrl-migration-pr-digests-worker.md`](r4-ctrl-migration-pr-digests-worker.md) — concrete Wave-1 trio-anchor worker brief (catalog #8)

- **Dispatch queue (Wave-1, in dispatch order)**:
  1. **catalog #8 PR digests** — trio anchor; worker brief landed in PR #2777; spawn first post-merge
  2. **catalog #10 work_prompts** — small (~400 LOC), template construction
  3. **catalog #16 utility-helpers** — fold-into-consumers; small surface
  4. **catalog #14 api-reviewer** — backend selection + invocation contract
  5. **catalog #12 ci** — poll + gate decisions pure
  6. **catalog #11 analyses** — sync + table queries
  7. **catalog #3 inbox** — promote existing demo (≈90% there per plan §3)
  8. **catalog #5 session_lifecycle** — spawn/idle/archive emergence
  
  Items 2–8 get worker briefs in follow-up PRs from this session (one per file, per `feedback_one_canonical_subissue_per_workitem.md` — no omnibus brief). Worker dispatch fires post-#2775 merge + Mgr-brief merge.

- **Cross-Mgr coordination open**: Verification Mgr (`deep-badger-38`; throttle interface for staged-debt budget — ping sent `msg_e8525152` 2026-05-12T19:44Z) and Emission-Targets Mgr (`deep-ibex-326`; trio-anchor confirmation — ping sent `msg_fe040236` 2026-05-12T19:44Z).

- **Trio-anchor Phase-3-partner placement correction** (per Emission-Targets Mgr `deep-ibex-326` `msg_c83099ac` + Director `clever-ant-97` `msg_0707a7c8` + Director template-review `msg_d1589d17` 2026-05-12): the catalog #8 trio's Phase-3 trigger is a **neutral 3-part** trio — (a) digest source-fact authority (consuming `dsl/extdeps/github/pulls.dag` `PullRequest`; if a narrow GitHub-domain digest-summary record needed, lands at `dsl/extdeps/github/pull_digest.dag` importing pulls.dag — **NOT as new fields on `PullRequest` itself**), (b) gunbc-owned render projection over `dsl/std/render.dag` (proposed `dsl/gunbc/digest_render.dag`); any Markdown-specific wrapper is a separate authority decision, not assumed live (`dsl/std/markdown_render.dag` is not on main as of 2026-05-12 per Director `msg_96a23421`), (c) named parity-harness gate green. Trio anchor does **not** gate on Emission Mgr's HTTP/SQL extdeps PR #2778 — converges via parallel gunbc-owned render landing. **Future worker briefs from this Mgr MUST name correct placement (extdeps source-fact / gunbc render-projection / std primitive) before dispatch** — standing item on per-worker-brief checklist.

- **Trio-anchor scope narrowed to smaller-first-path** (per Emission Mgr `deep-ibex-326` `msg_c5b7d419` 2026-05-12): catalog #8 worker scope reduced to existing `PullRequest` / `PullReview` / `Diff` operation outputs only. `render_ci_digest` / `render_conflict_digest` deferred to follow-up Phase 1.5 PR; no `pull_digest.dag` prerequisite; no new source-fact carriers gunbc-side. CI/conflict/mergeability gap routes to Emission Mgr as **follow-up** narrow extdep PR only if parity work proves load-bearing post-landing. Removes Emission-side prerequisite from trio anchor critical path.

- **Substrate-grep miss self-memorial** (catch by Emission Mgr `msg_f9d2bfab` 2026-05-12): initial worker brief invented carrier names `GithubPr` / `CiState` / `ConflictState` without grepping `dsl/extdeps/github/pulls.dag` (real names on main: `PullRequest` / `PullRequestRef` / `IssueComment` — no CI / conflict / mergeability carriers exist yet). Direct violation of `feedback_substrate_grep_before_authoring.md` + `feedback_grep_substrate_before_naming_ratification.md`. **Corrective discipline (added to per-worker-brief checklist)**: before any service-block sketch in a worker brief, **grep the cited extdep files on main and quote real type names** in the brief. CI/conflict/mergeability source facts route to Emission-Targets Mgr for placement under `dsl/extdeps/github/*.dag` (narrow `pull_digest.dag` or sibling), NOT invented on the gunbc side and NOT inflated onto `PullRequest`.

- **Staged-debt receipt-trail ledger** (per Verification Mgr `deep-badger-38` ratification `msg_5f8db22f` + `msg_6faaf178` 2026-05-12): single SoT at [`docs/audit/r4-ctrl-phase15-subsystem-receipt-trail.md`](../audit/r4-ctrl-phase15-subsystem-receipt-trail.md). 4-tuple bool columns + derived `open_receipt_debt` flag (`phase15_pr_merged ∧ ¬(phase3_emission_landed ∧ parity_passed)`); dispatch-pause gate at `count ≥ 3`. This Mgr owns row inserts on Phase 1.5 merges; Verification owns column semantics + parity flips; Emission flips `phase3_emission_landed`. Ledger skeleton + first row (catalog #8 placeholder) landed in PR #2777.
