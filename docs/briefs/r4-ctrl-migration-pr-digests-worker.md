# Worker brief — `dsl/ctrl/pr_digests.dag` (catalog #8)

**Status**: `PROPOSAL`. Dispatch fires after PR #2775 merges AND [`docs/briefs/r4-ctrl-migration-subsystem-modeling-manager.md`](r4-ctrl-migration-subsystem-modeling-manager.md) merges (this brief's parent).

**Authority anchor**: project-plan §3 catalog row #8 (PR digests); operator directive 2026-05-12T18:30Z ("make a project plan…migrate as much of ctrl/ onto dag as possible ASAP"); Director ratification of Q1(a) brief-first Mgr shape 2026-05-12 via `msg_c1daa5ae-2dfd-4e0c-a4bd-cdec024e383a`.

**Parent**: Subsystem-Modeling Mgr (session `merry-newt-448`). Project Director: `clever-ant-97`.

**Closure predicate**: `dsl/ctrl/pr_digests.dag` authored + 2-distinct-provider APPROVE + Mgr ratification of Practice-4 receipts.

**Wave-1-trio-anchor status**: this worker is the **proposed trio anchor** per Mgr brief §"Wave-1-trio checkpoint." If this subsystem converges end-to-end (algebra ✓ + this PR ✓ + Phase 3 gunbc-owned render projection over `dsl/std/render.dag` ✓ + named parity-harness gate green ✓) by Day 10, Wave-2 fanout unblocks. If not, Mgr surfaces to Director for re-scope before Wave-2 dispatch. **Phase-3 is gunbc-owned render projection, NOT a new extdep** — per INVARIANTS P1 + Director/Emission ratification 2026-05-12.

---

## Why catalog #8 first

Per Mgr brief §"Wave-1-trio checkpoint" rationale: PR-digest helpers are pure functions over GitHub PR JSON shapes. The corresponding Phase 3 emission target is a **gunbc-owned render projection** over `dsl/std/render.dag` (proposed name `dsl/gunbc/digest_render.dag`), narrow-consuming `dsl/extdeps/github/pulls.dag` for source carriers — **NOT** a new extdep field/operation. If a Markdown-specific wrapper is later needed, it is a **separate source-fact/render placement decision** and must land as its own authority — `dsl/std/render.dag` notes a future `std/markdown_render.dag` format-specific wrapper but no such authority exists on main today (verified 2026-05-12; per Director `msg_96a23421`).

**Placement discipline** (per Emission-Targets Mgr `deep-ibex-326` 2026-05-12 message `msg_c83099ac`): `dsl/extdeps/github/` owns GitHub-API source-of-record facts only (PR / CI / review shapes); rendering/projection is gunbc-owned. This is the `feedback_extdeps_header_discriminator_before_field_placement.md` discipline applied to this lane — gunbc emission/policy facts cannot live on extdeps platform carriers per INVARIANTS P1. The trio anchor therefore does NOT gate on Emission-Targets Mgr's HTTP/SQL extdeps work (PR #2778); it converges via a separate gunbc-owned render landing.

---

## Scope

Type-only modeling of the PR-digest subsystem (~1,200 TS LOC across 5 files in `ctrl/scripts/session-dashboard/`):

- `pr_attached_urls.mjs` — extract URLs from PR bodies/comments
- `pr_ci_digest.mjs` — render CI status into a one-line digest
- `pr_conflict_digest.mjs` — render merge-conflict state
- `pr_merge_ready_digest.mjs` — composite "ready to merge?" verdict
- `pr_rest_fallback.mjs` — REST-API fallback paths when GraphQL incomplete

Each is a pure function modulo GitHub-API I/O. Phase 1.5 models the **input/output carriers + function signatures**; runtime I/O lives in `dsl/extdeps/github/*.dag` (already present).

## Deliverable

`dsl/ctrl/pr_digests.dag` containing:

1. **Module header** with consumer receipt (neutral Phase-3 trigger per Director ratification 2026-05-12 `msg_d1589d17`):
   ```
   // ctrl/pr_digests.dag — PR digest helpers (status / conflicts / merge-readiness / URL extraction).
   //
   // Current authority (consumer-side): ctrl/scripts/session-dashboard/{pr_attached_urls,
   // pr_ci_digest, pr_conflict_digest, pr_merge_ready_digest, pr_rest_fallback}.mjs
   //
   // STAGED → AUTHORITY trigger (single event under INVARIANTS P2/P5): the
   // ctrl PR cut-over deleting the 5 .mjs files. Trio convergence below is
   // the gating precondition that authorizes cut-over dispatch — it is NOT
   // itself the authority flip:
   //   (a) digest source-fact authority (consuming dsl/extdeps/github/pulls.dag PullRequest
   //       as fetched source-of-record; if a narrow GitHub-domain digest-summary record
   //       is needed, lands at dsl/extdeps/github/pull_digest.dag importing pulls.dag —
   //       NOT as new fields on PullRequest itself per Director/Emission-Targets ratification
   //       2026-05-12),
   //   (b) gunbc-owned render projection over dsl/std/render.dag (proposed
   //       name dsl/gunbc/digest_render.dag); any Markdown-specific wrapper
   //       is a separate authority decision, not assumed live,
   //   (c) named parity-harness gate green
   ```

2. **Carrier types** (DFS via M9 before defining; **MUST grep `dsl/extdeps/github/pulls.dag` before naming any carrier** per substrate-grep-miss correction below):

   **Existing source-fact carriers to import, NOT redeclare** (verified in `pulls.dag` on main 2026-05-12; scope narrowed per Emission Mgr `msg_c5b7d419` "take the smaller first path"):
   - `PullRequest` — fields `number / title / body / state / html_url / user / head / base / created_at / updated_at / merged_at / draft`
   - `PullRequestRef` — `ref / sha / label`
   - `PullReview` — output of `service github.Pulls.ListReviews` operation; fields `id / body / state / commit_id / html_url`
   - `Diff` operation output — `diff: String` (raw unified diff text)
   - `IssueComment` if URL extraction needs comment-body parsing

   **CI/conflict/mergeability are OUT OF SCOPE for this first worker** (per Emission Mgr `msg_c5b7d419` 2026-05-12): they are NOT assumed fields on `PullRequest`, they are NOT added as new gunbc-side carriers, and they are NOT prerequisites for the trio anchor. If parity/receipt work proves CI/conflict/mergeability is load-bearing post-landing, a **follow-up** narrow `dsl/extdeps/github/*.dag` source-fact module gets routed to Emission Mgr (`deep-ibex-326`) — out of this PR's scope.

   **No `pull_digest.dag` prerequisite** (per Emission Mgr `msg_c5b7d419`): do NOT propose a `pull_digest.dag` extdep landing as gate to this worker. If the worker discovers it needs a stable digest-input summary record, model it as a derived record over **existing** `PullRequest` facts inside `dsl/ctrl/pr_digests.dag` (or surface upward), NOT as new provider source authority.

   **Gunbc-side carriers this file owns** (narrowed to smaller-first-path scope; Practice-4 dimensional check applied per operator review codex findings #3 + #4 2026-05-12 commit `a6bd5f56`):
   - `MergeReadinessVerdict` — closed sum:
     - `Ready`
     - `NotReady { first_reason: String, more_reasons: List<String> }` — **structural cardinality**: `NotReady` is uninhabitable with zero reasons (the invariant is encoded in the carrier shape, not in a runtime check on a bare `List<String>`). Reasons derived only from existing PullRequest/PullReview fields (draft / state / merged_at / review states) — NOT from CI/conflict facts (deferred). Practice-4 receipt **required**.
   - `AttachedUrl` — `{ url: Url, container: AttachedUrlContainer, context: AttachedUrlTextContext }` (reuse `Url` from `std.types`). Two **separate dimensional coordinates**, not a single conflated sum:
     - `AttachedUrlContainer` closed sum: `PrBody | IssueCommentBody | PullReviewBody` — *source container* dimension (which GitHub object the URL was found in). Variants are **exactly the source-fact set imported by the service block below** (`PullRequest.body`, `List<IssueComment>` bodies, `List<PullReview>` bodies); no variant exists without a matching imported carrier, per INVARIANTS P2 single-authority discipline. `ReviewCommentBody` (line-level review comments — `dsl/extdeps/github/pulls.dag::ReviewComment`) is deliberately excluded from this first worker and gates on a follow-up Phase 1.5 PR that adds `ListReviewComments` operation output to the import list; until then no `AttachedUrl` value can claim a `ReviewComment` source.
     - `AttachedUrlTextContext` closed sum: `Prose | InlineCode` — *text context* dimension (whether the URL was in a code fence/backticks vs prose). Note: Practice-4 dimensional check made this split visible — original conflated `AttachedUrlSource` was mixing the two coordinates.
     - Both sums get Practice-4 receipts **required**.
   - `RestFallbackReason` — closed sum (`GraphqlIncomplete | GraphqlRateLimited | GraphqlSchemaUnknown`) — Practice-4 receipt **required**

   `CiDigestLine` / `ConflictDigestLine` deferred to follow-up Phase 1.5 PR per scope narrowing.

3. **Service block** narrowed to existing `PullRequest` + `PullReview` + `Diff` output facts (per Emission Mgr `msg_c5b7d419` smaller-first-path ratification 2026-05-12; no transport):
   ```
   service ctrl.PrDigests {
     fn extract_attached_urls(pr: PullRequest, comments: List<IssueComment>, reviews: List<PullReview>) -> List<AttachedUrl>
     fn render_pr_summary_line(pr: PullRequest) -> String         // status/draft/title rendering from existing fields
     fn merge_readiness_verdict(pr: PullRequest, reviews: List<PullReview>) -> MergeReadinessVerdict
                                                                  // verdict from existing fields only: state/draft/merged_at + review-state aggregation
     fn classify_rest_fallback(error: GitHubErrorShape) -> Option<RestFallbackReason>
   }
   ```
   **CI/conflict-rendering signatures deferred** (`render_ci_digest`, `render_conflict_digest`) — those land in a follow-up Phase 1.5 PR once Emission Mgr places the CI/conflict source-fact carriers under `dsl/extdeps/github/`. This worker does NOT model them on the gunbc side.

4. **Practice-4 receipts** for the 3 closed sums above (per Mgr brief §"Per-worker brief template" item 4 — classification + dissolution pattern + trigger).

## Acceptance gates

1. `dsl/ctrl/pr_digests.dag` parses + compiles (whatever validates `.dag` files in this repo — likely `cargo test -p v3-compiler` covers it).
2. Practice-4 receipts on `MergeReadinessVerdict`, `AttachedUrlContainer`, `AttachedUrlTextContext`, `RestFallbackReason` — each names classification + dissolution-pattern + trigger. (Four receipts: dimensional split made `AttachedUrl*` two independent coordinates.)
3. Consumer-receipt header cites all 5 ctrl `.mjs` files + names the **neutral 3-part Phase-3 trigger** (digest source-fact authority + gunbc/std render projection + named parity harness green) per Director ratification `msg_d1589d17`. **DO NOT** name `dsl/extdeps/github/digest_render.dag` or any new render-field on `PullRequest` — both placements were ruled out by Director + Emission Mgr 2026-05-12.
4. Cost-of-change check: adding a new `RestFallbackReason` variant — or a new `AttachedUrlContainer` (e.g. `IssueBody` when issues join the lane) or `AttachedUrlTextContext` variant — touches **only** this file (no parallel registry / no extdep update / no consumer-side fix-up). If it touches more, surface to Mgr — that's a substrate-shape signal.
5. M9 DFS audit comment: for each carrier, name the existing `dsl/std/` or `dsl/extdeps/github/` primitive it attaches to (or document why it must be new — must be a substantive structural reason, not "didn't find one").
6. Doc-only — zero emission code; zero TS-side changes.

## STOP/PING criteria

- **DO NOT STOP for CI/conflict/mergeability source-fact gaps** — those are out-of-scope for this worker per Emission Mgr `msg_c5b7d419` smaller-first-path ratification. Narrow to existing `PullRequest` / `PullReview` / `Diff` fields. If during implementation you find a digest-output cannot be rendered from these alone, surface to Mgr — do NOT add new source-fact carriers gunbc-side and do NOT block this PR on Emission Mgr placement of CI/conflict carriers.
- **STOP** if any rendered digest output requires a field not on existing `PullRequest`/`PullReview`/`Diff` output and the gap looks load-bearing for parity — surface to Mgr; the gap routes to Emission Mgr as a follow-up narrow source-fact PR, NOT a prerequisite for this worker's landing.
- **STOP** if a closed sum has no clear dissolution pattern (TERMINAL unjustified) — surface to project Director for ratification per `feedback_pattern_a_scaffold_sentinel_per_instance_ratification.md`.
- **STOP** if any carrier name collides with `src/v3/SELF_HOSTING.md` or `dsl/std/` (per `feedback_self_hosting_md_authority_audit_before_naming.md`) — rename and re-grep before proceeding.
- **PING** project Director on PR-open with brief reference.

## Reference materials

- [`docs/r4-ctrl-dag-migration-project-plan.md`](../r4-ctrl-dag-migration-project-plan.md) §3 row #8 + §6 parallel-critical-path
- [`docs/briefs/r4-ctrl-migration-subsystem-modeling-manager.md`](r4-ctrl-migration-subsystem-modeling-manager.md) (parent Mgr brief)
- [`dsl/ctrl/README.md`](../../dsl/ctrl/README.md) (path conventions + receipt format)
- [`dsl/extdeps/github/pulls.dag`](../../dsl/extdeps/github/pulls.dag) (existing GitHub carriers — DFS source for M9)
- [`MODELING.md`](../../MODELING.md) M9 (DFS the concept DAG before defining)
- [`INVARIANTS.md`](../../INVARIANTS.md) P2 (declarations alone are staging)
