# R2 Manager-Brief Authority Matrix

**Status:** PROPOSAL pre-R1-close. Promotes to ACTIVE on R1 closure → R2 promotion transition. Per openai-pro PAUSE_AND_REGROUP meta-review on [PR #835](https://github.com/gunb-ai/gunbc/pull/835) sha `bfaab66c` — the structural artifact that graduates "authority ambiguity at stage boundaries" out of per-instance review and into a class invariant.

**Authority subject:** the 6 R2 manager briefs at `docs/briefs/r2-{grounding,substrate,modeling,impossible-bugs,pure-bootstrap,release}-manager.md`.

**What the matrix does:** every deliverable / signal / ledger entry owned by an R2 manager belongs to **exactly one** of 5 disjoint artifact categories. The categories carry distinct invariants (ownership rules, artifact format, lifecycle). When a brief lists a deliverable, the deliverable's category is determinate — not authored ambiguously between worker brief / decision brief / signal / ledger / placeholder.

This is the structural fix for the recurring "authority ambiguity" pattern that surfaced as B7 dual-contract on PR #835 sha `bfaab66c` (resolved in `f916fba64`) and as Grounding's unbounded `Pending` line (also resolved `f916fba64`). Wording sweeps closed the instances; this matrix prevents the class.

## The 5 disjoint categories

### Category 1: Worker brief

A dispatchable authoring task that produces a concrete deliverable (worker authors code or doc artifact + lands as a PR).

**Invariants:**
- **Pre-spawn owner:** Director or PM (per inbox #828 split for the R2 spin-up wave; varies by program scope).
- **Post-spawn owner:** the responsible R2 manager, autonomously (no Director authoring per `docs/r2-structure.md` Manager structure).
- **Consumer:** worker who picks up the brief and lands the PR.
- **Artifact format:** `docs/briefs/<scope>-<slug>-worker.md` typically, OR `docs/briefs/<program-name>.md` for program briefs.
- **Lifecycle:** authored → dispatched → worker lands PR → brief marked closed (or refined into follow-on briefs).

**Examples:** `docs/briefs/b4-2-structural-fold-shape-carrier-worker.md` (Substrate Manager → worker dispatch); `docs/briefs/r2-modeling-int-lit-magnitude-worker.md` (Modeling Manager → worker dispatch); `docs/briefs/r2-release-b5-loop-construction-closure-audit-worker.md` (R2 Release Manager → worker dispatch); `docs/briefs/r1c-b-t-p0-fixtures-worker.md` (R1 Closure Manager → worker dispatch).

### Category 2: Decision brief

A scoped design call where the worker's role is to **make + lock** a decision (not implement a pre-decided thing). Has explicit options to choose between, evidence to cite, and a "lock the choice" deliverable.

**Invariants:**
- **Pre-spawn owner:** Director or PM (typically PM for §6a-style design calls; Director for cross-program scope decisions).
- **Post-spawn owner:** responsible R2 manager (commonly R2 Release Manager since most decision briefs are cross-cutting).
- **Consumer:** the worker who makes the decision (with manager + cross-program review per scope).
- **Artifact format:** `docs/briefs/<scope>-pick-worker.md` or `<scope>-decision-worker.md`.
- **Lifecycle:** authored → worker reads options + evidence → picks → lands a "lock the choice" PR (typically updates an upstream design doc to record the decision + minimal demo) → decision-brief is **closed in scope** (subsequent work is post-pick follow-through, distinct category).

**Important distinction from Category 1:** a decision brief's deliverable is the **decision itself plus a minimal demo of the chosen path**. Bulk migration / consumer adoption / etc. is **explicitly out of scope** for the decision brief; that work is the next worker brief (category 1) authored *after* the decision lands. Conflating decision-pick with implementation creates duplicate authority (the §6a anti-pattern).

**Examples:** `docs/briefs/t-permethodmetadata-pick-worker.md` (PR #794, locked Option 3 `MethodContract` + minimal demo). The follow-through worker brief at `docs/briefs/r2-release-6a-follow-through-worker.md` is Category 1 (worker brief), distinct from the pick.

### Category 3: Cross-manager signal

A priority hint, lane-close notification, or scope-change escalation that **routes information across managers** without authoring a code-or-doc deliverable. The signal's "deliverable" is the routing itself.

**Invariants:**
- **Pre-spawn owner:** documenter (PM or Director) authoring the signal-content snapshot for post-spawn relay.
- **Post-spawn owner:** sender manager queues; receiver manager acks.
- **Consumer:** receiver manager.
- **Artifact format:** documented either (a) inline in sender's manager brief under "Cross-program dependencies → Produces" / "Consumes", OR (b) as a standalone signal-doc at `docs/briefs/<sender>-<purpose>-relay-to-<receiver>.md` for content snapshots requiring more elaboration.
- **Lifecycle:** documented → spawn fires → sender queues signal → receiver acks + adjusts behavior → signal-doc marks RESOLVED (signal delivered + consumed); follow-up cleanup retires the doc.
- **Channel:** GitHub session-inbox issue comment for human-target signals; cross-manager queue (per R1 `Cross-manager notifications queued` brief pattern) for inter-manager signals (per `docs/r2-structure.md §"Manager structure"` escalation signal channel).

**NOT a worker brief.** The conflation of signal with worker brief is the B7 anti-pattern (resolved in PR #835 `f916fba64`). A signal does NOT author a deliverable; it routes a priority/scope statement.

**Examples:** `docs/briefs/r2-release-b7-priority-hint-relay-to-pure-bootstrap.md` (R2 Release Manager → R2 Pure Bootstrap Manager); lane-close signals from each lane manager → R2 Release Manager (closure ledger).

### Category 4: Standing reporting duty

A continuous-state ledger / monitor / aggregated report owned by a manager throughout the manager's lifecycle. Not authored once and closed; refreshed at cadence.

**Invariants:**
- **Pre-spawn owner:** N/A (these activate on spawn; pre-spawn placeholder = "fill on spawn").
- **Post-spawn owner:** the manager whose role includes the standing duty (typically R2 Release Manager).
- **Consumer:** Director / user / cross-manager.
- **Artifact format:** ledger entries / cadence reports inline in manager brief's "Working state" section, OR sibling ledger doc.
- **Lifecycle:** activates on spawn → refreshes per cadence (integration-reflection pass / weekly health check / etc.) → dissolves on manager's dissolution.

**Examples:** R2 closure ledger (R2 Release Manager); velocity-tripwire ratio report (R2 Release Manager surfacing to Director per `INVARIANTS.md#p5-progress-is-dissolution`); Substrate-Manager bottleneck watch (R2 Release Manager, per `docs/r2-structure.md` §"R2 Release Manager"); R1 Closure Manager's lane status table.

### Category 5: Pre-spawn placeholder

A skeleton brief authored before R2 spawn that becomes operational on spawn. **Not a permanent category** — placeholders graduate into one of categories 1-4 at spawn, OR dissolve if their predicates change.

**Invariants:**
- **Pre-spawn owner:** PM (manager-brief skeletons per inbox #828 PM portion) OR Director (worker-brief skeletons per inbox #828 Director portion).
- **Post-spawn owner:** transitions per the brief's spawn-time graduation rule.
- **Consumer:** cross-program scope-readiness coordinator at spawn time.
- **Artifact format:** the brief itself, with explicit `Pre-spawn vs post-spawn authority` section bounding the placeholder phase.
- **Lifecycle:** authored pre-spawn → R2 spawn → graduates into category 1/2/3/4 OR dissolves.

**Examples:** the 6 R2 manager briefs at `docs/briefs/r2-{grounding,substrate,modeling,impossible-bugs,pure-bootstrap,release}-manager.md` (each is a Category 5 placeholder that graduates on R2 spawn into the manager's operational state). Each names its post-spawn owner explicitly in the `Pre-spawn vs post-spawn authority` section.

## Per-manager deliverable inventory

Each deliverable owned by an R2 manager belongs to exactly one of categories 1-5. This inventory tags each deliverable; the manager briefs cite this matrix as authority and stop self-categorizing.

### R2 Grounding Manager (`docs/briefs/r2-grounding-manager.md`)

| Deliverable | Category | Notes |
|---|---|---|
| The manager brief itself | 5 (placeholder) | Graduates on R2 spawn |
| T-Ground-Pilot worker | 1 (worker brief) | Landed PR #765 |
| T-Ground-Engine Phase 1 typestructure | 1 (worker brief) | Landed PR #788 |
| T-Ground-Rust full implementation | 1 (worker brief) | Pending |
| T-Ground-Engine Phase 2 implementation | 1 (worker brief) | Pending; gated on substrate |
| T-Ground-Tests | 1 (worker brief) | Pending |
| T-Ground-Dissolve (Track-13 cleanup) | 1 (worker brief) | Pending; final critical-path step |
| T-Ground-Python | 1 (worker brief) | Pending; fill-queue parallel |
| T-Ground-Go | 1 (worker brief) | Pending; fill-queue parallel |
| Lane-close signals → R2 Release Manager | 3 (signal) | Cross-manager queue at lane close |

### R2 Substrate Manager (`docs/briefs/r2-substrate-manager.md`)

| Deliverable | Category | Notes |
|---|---|---|
| The manager brief itself | 5 (placeholder) | Graduates on R2 spawn |
| T-Substrate cardinality-for-int-lit sub-lane | 1 (worker brief) | Authored on PR #836 + PR #806 |
| T-Substrate nominal-opaque-for-Secret sub-lane | 1 (worker brief) | Authored on PR #836 |
| T-Substrate parametric-algebra-for-Dimensions sub-lane | 1 (worker brief) | Authored on PR #836 (closed by audit per Director receipt) |
| T-Substrate ValueBody-list/sum + std.unicode | 1 (worker brief) | PR #790 |
| B4 program brief | 1 (worker brief; program-level) | `b4-identity-carrier-substrate-pass.md` |
| B4.1 DeclarationRef consumer migration | 1 (worker brief) | Landed PR #819 |
| B4.1a DeclarationRef runner migration | 1 (worker brief) | Landed PR #819 |
| B4.2 fold-shape carrier | 1 (worker brief) | Authored on PR #836 |
| B4.3 emit-helper carrier | 1 (worker brief) | Authored on PR #836 |
| B4.4 extdeps-fixture-set carrier | 1 (worker brief) | Authored on PR #836 |
| B4.5–B4.12 Phase 2 site dissolutions | 1 (worker brief; 8 sub-briefs) | Phase 2 queue authored on PR #836 |
| Cardinality-for-int-lit carrier-readiness signal → Modeling | 3 (signal) | Cross-manager queue |
| Nominal-opaque-for-Secret carrier-readiness signal → Modeling | 3 (signal) | Cross-manager queue |
| Parametric-algebra-for-Dimensions carrier-readiness signal → Modeling | 3 (signal) | Cross-manager queue (already closed by audit) |
| ValueBody-list/sum carrier-readiness signal → Modeling + Grounding | 3 (signal) | Cross-manager queue |
| Sub-lane / Phase-close signals → R2 Release Manager | 3 (signal) | Cross-manager queue |

### R2 Modeling Manager (`docs/briefs/r2-modeling-manager.md`)

| Deliverable | Category | Notes |
|---|---|---|
| The manager brief itself | 5 (placeholder) | Graduates on R2 spawn |
| Int-lit magnitude consumer worker | 1 (worker brief) | Authored on PR #836; gated on Substrate carrier |
| Secret<T> graduation worker | 1 (worker brief) | Authored on PR #836; gated on Substrate carrier |
| Dimensions phantom-parameter consumer worker | 1 (worker brief) | Authored on PR #836; ungated (audit closed Substrate-side) |
| Tokenizer charclass phase-2 consumer worker | 1 (worker brief) | Authored on PR #836; gated on T-Substrate ValueBody-list/sum |
| Item-close signals → R2 Release Manager | 3 (signal) | Cross-manager queue |

### R2 Impossible-Bugs Manager (`docs/briefs/r2-impossible-bugs-manager.md`)

| Deliverable | Category | Notes |
|---|---|---|
| The manager brief itself | 5 (placeholder) | Graduates on R2 spawn |
| Nested-optional flatten implementation worker | 1 (worker brief) | Authored on PR #836; gated on cardinality refinement |
| Unhandled-diagnostic-paths implementation worker | 1 (worker brief) | Authored on PR #836; per design-doc Director-actionable totality-by-omission path |
| Unenumerated-effects implementation worker | 1 (worker brief) | Authored on PR #836; closed-system per #808 |
| Substrate-gap escalation → Substrate Manager | 3 (signal) | Cross-manager queue when class surfaces substrate gap |
| Class-close signals → R2 Release Manager | 3 (signal) | Cross-manager queue |

### R2 Pure Bootstrap Manager (`docs/briefs/r2-pure-bootstrap-manager.md`)

| Deliverable | Category | Notes |
|---|---|---|
| The manager brief itself | 5 (placeholder) | Graduates on R2 spawn |
| Tier 3 mirror dissolutions (termination/computation/induction/effect-carrier) | 1 (worker brief; 4+ sub-briefs) | Authored in `docs/briefs/r2-pb-tier3-mirror-dissolution-workers.md`; mirror-by-mirror dispatch |
| Tier 2 patch_lower_helpers_* retirement | 1 (worker brief) | **Closed / green (PB lower-helper slice only):** #1014 (native `refinement` in generated `lower_helpers`); **#1192** ratchet `bridge_lower_helpers_patch_zero_residual_test.rs` — zero contiguous `patch_lower`+`_helpers` in `src/v3/compiler` `.rs` + `build.rs` per SG-0 + closure ledger. **Not** umbrella exact-string patching retirement — other classes keep their own triggers (`docs/r3-structure.md`, `docs/r2-closure-ledger.md`). |
| `kernel_algebra_profile` mirror dissolution | 1 (worker brief) | Pending; gated on `ValueBody::Map` substrate |
| Post-R1 emergent dissolutions | 1 (worker brief; ad-hoc) | Pending; emerge during R2 |
| Lane-close signals → R2 Release Manager | 3 (signal) | Cross-manager queue |

### R2 Release Manager (`docs/briefs/r2-release-manager.md`)

| Deliverable | Category | Notes |
|---|---|---|
| The manager brief itself | 5 (placeholder) | Graduates on R2 spawn |
| §6a per-method-metadata follow-through worker | 1 (worker brief) | Authored on PR #847; **NOT** a decision brief (§6a pick is closed by `t-permethodmetadata-pick-worker.md` PR #794) |
| B5 Loop construction-closure audit worker | 1 (worker brief) | Authored on PR #847; audit-first |
| B6 file-preference rank checklist worker | 1 (worker brief) | Authored on PR #847; XS-trivial |
| B7 priority-hint relay to Pure Bootstrap | 3 (signal) | Authored on PR #847; signal-doc, NOT worker brief |
| Thesis-claim coverage mapping table | 1 (worker brief; documents) | Authored on PR #847 as `docs/thesis-claim-coverage.md` |
| R2 closure ledger | 4 (standing duty) | Activates on R2 spawn |
| Velocity-tripwire ratio reporting | 4 (standing duty) | Per integration-reflection cadence |
| Substrate-Manager bottleneck watch | 4 (standing duty) | Per `docs/r2-structure.md` §"R2 Release Manager" |
| R2 demo coordination | 4 (standing duty) | Per Demo discipline; "it runs" artifact at each lane close |
| B-wave Tier 0 through-merge (B1/B2/B3) | 4 (standing duty) | Coordinate worker iteration on already-in-flight PRs |
| v2 retirement coordination | 4 (standing duty) | Post-R2 operational; tracked but not gated |

## Sweep verification — current state of the 6 R2 manager briefs

Verifying that each manager brief on PR #835 sha `3260d710` categorizes its deliverables consistently with this matrix:

- ✅ **Grounding Manager** — All deliverables in the brief's `Owned deliverables` table fit Category 1 (worker briefs); Cross-program signals to Release fit Category 3. `Pending` section uses bounded pre-spawn/post-spawn language (fixed in `f916fba64`). No category conflicts.
- ✅ **Substrate Manager** — All sub-lanes + B4.1-B4.12 fit Category 1; produces Category 3 signals to Modeling + Grounding; signals lane-close to Release as Category 3. B4.1 status no longer carries stale BLOCKING (fixed in `74b679b8a`). No category conflicts.
- ✅ **Modeling Manager** — All 4 items fit Category 1 (worker briefs gated on Substrate carriers); item-close signals fit Category 3. No category conflicts.
- ✅ **Impossible-Bugs Manager** — All 3 classes fit Category 1; substrate-gap escalations + class-close signals fit Category 3. Filenames use canonical `-design.md` suffix (fixed `70df547e6`). No category conflicts.
- ✅ **Pure Bootstrap Manager** — All deliverables fit Category 1; lane-close signals fit Category 3. No category conflicts.
- ✅ **Release Manager** — Owned deliverables split correctly: §6a follow-through / B5 / B6 / thesis-claim mapping fit Category 1; B7 fits Category 3 (signal); closure ledger / velocity tripwire / bottleneck watch / demo coordination fit Category 4. The B7 dual-contract was fixed in `f916fba64`; the §6a stale framing fixed in `74b679b8a` + `3260d7100`. No category conflicts.

**Sweep result: all 6 R2 manager briefs are consistent with the matrix at PR #835 sha `3260d710`. No additional fix-pushes needed.**

## Local review checklist (per meta-review recommendation #3)

When authoring or reviewing an R2 manager brief, every `Owned deliverables`, `Pre-spawn vs post-spawn authority`, `Autonomous dispatch authority`, and `Sub-briefs (authored / pending)` section must agree on:
- **Owner** (pre-spawn AND post-spawn) per the deliverable's category in this matrix.
- **Artifact type** (which of categories 1-5).
- **Status.** A single deliverable cannot be both `DISPATCHED` / `AUTHORED` in the deliverables table AND `Pending` / `NOT YET AUTHORED` in the Sub-briefs section. If a lane is partially landed (e.g., Pilot done; full implementation pending), the table status must scope the partial state explicitly (e.g., `PARTIAL — Pilot PR #X done; full implementation pending`), not generic `DISPATCHED` with a parenthetical that contradicts the Sub-briefs Pending list. The §"Sub-briefs (authored / pending)" section is the single authority for which sub-briefs are authored vs pending; the deliverables table cites that authority and does not duplicate it ambiguously. Surfaced by openai-pro APPROVE_WITH_COMMENTS finding on PR #835 sha `3260d710` (T-Ground-Rust dual-status: row said `DISPATCHED`, Pending list said `T-Ground-Rust full implementation`); fixed in `3ef1509dc`.

A deliverable that doesn't cleanly fit one of the 5 categories is **a category bug**, not a sixth category — surface for matrix amendment rather than authoring an ambiguous brief.

### Pre-author verification invariant

Before authoring a brief that references substrate state, gate condition, existing brief, or upstream design-doc disposition: **grep the source-of-truth before slicing**. Specifically:
- `src/v3/std/`, `src/v3/spec/`, `src/v3/compiler/src/` for substrate / runtime / compiler state cited in the brief.
- `docs/briefs/` for existing briefs that may already cover the scope (canonical authority + scope-closure clauses).
- The cited design doc's §Director-actionable / §Q-recommendation / §Decision sections **in full**, not just the section title.

Cite specific `file:line` / brief filename / `§ref` in the brief's `Read first` section. State the audit receipt before slicing. The 7-reframe pattern on PR #836 (`feedback_verify_thesis_claims` violations: nested-optional gating, unhandled-diagnostic predicate-entailment default, unenumerated-effects 8-req elision, parametric-algebra producer redundancy, cardinality-for-int-lit redundancy, nominal-opaque 7th-connective option, int-lit consumer scope mismatch with `wise-pike-578`) demonstrates the failure mode this invariant prevents. This is the operationalization of `feedback_verify_thesis_claims` for the brief-authoring family.

These do not need new top-level INVARIANTS.md P-rules; `INVARIANTS.md#p2-boundary-discipline` (single-authority), `INVARIANTS.md#p5-progress-is-dissolution` (dispatch-discipline), and the existing `feedback_verify_thesis_claims` discipline already cover them. These are the **local invariants** for the manager-brief family — collected here so brief authoring can cite this matrix as a single review checkpoint.

## Refresh discipline

- **When to refresh:** at every R2 manager brief authoring round (sweep new deliverables for category fit); at R2 spawn (verify Category 5 placeholders graduate cleanly into 1-4); at every release transition.
- **Refresh process:** sweep new deliverables; tag each with category; flag any that don't fit; surface as matrix-amendment proposal if a sixth-category candidate emerges.
- **Authority of this doc:** **descriptive over the union, prescriptive over the categorization.** Manager briefs are authoritative on their own deliverables' content; this matrix is authoritative on which category each deliverable belongs to.

## Cross-refs

- Originating meta-review: openai-pro on [PR #835](https://github.com/gunb-ai/pull/835) sha `bfaab66c` (PAUSE_AND_REGROUP verdict).
- R2 manager briefs (authority subjects): `docs/briefs/r2-{grounding,substrate,modeling,impossible-bugs,pure-bootstrap,release}-manager.md` (lands on PR #835 merge).
- R2 worker briefs (Category 1 examples authored on PR #836): the 14 Director-portion briefs.
- R2 follow-through worker briefs (Category 1 examples authored on PR #847): §6a / B5 / B6 / thesis-claim-mapping.
- Cross-manager signal example (Category 3 on PR #847): B7 priority-hint relay.
- Standing duties (Category 4) inline in `r2-release-manager.md` Owned deliverables section.
- Manager structure authority: [`docs/r2-structure.md` §"Manager structure"](../r2-structure.md).
- Discipline framework: [`INVARIANTS.md#p5-progress-is-dissolution` "Dispatch-Discipline Mechanisms"](../../INVARIANTS.md).
- Escalation discipline: [`docs/escalation-paths.md`](../escalation-paths.md).
