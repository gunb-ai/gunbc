# Worker brief — `dsl/ctrl/pr_digests.dag` (catalog #8)

**Status**: `PROPOSAL`. Dispatch fires after PR #2775 merges AND [`docs/briefs/r4-ctrl-migration-subsystem-modeling-manager.md`](r4-ctrl-migration-subsystem-modeling-manager.md) merges (this brief's parent).

**Authority anchor**: project-plan §3 catalog row #8 (PR digests); operator directive 2026-05-12T18:30Z ("make a project plan…migrate as much of ctrl/ onto dag as possible ASAP"); Director ratification of Q1(a) brief-first Mgr shape 2026-05-12 via `msg_c1daa5ae-2dfd-4e0c-a4bd-cdec024e383a`.

**Parent**: Subsystem-Modeling Mgr (session `merry-newt-448`). Project Director: `clever-ant-97`.

**Closure predicate**: `dsl/ctrl/pr_digests.dag` authored + 2-distinct-provider APPROVE + Mgr ratification of Practice-4 receipts.

**Wave-1-trio-anchor status**: this worker is the **proposed trio anchor** per Mgr brief §"Wave-1-trio checkpoint." If this subsystem converges end-to-end (algebra ✓ + this PR ✓ + Phase 3 render-helpers extdep ✓ + parity test ✓) by Day 10, Wave-2 fanout unblocks. If not, Mgr surfaces to Director for re-scope before Wave-2 dispatch.

---

## Why catalog #8 first

Per Mgr brief §"Wave-1-trio checkpoint" rationale: PR-digest helpers are pure functions over GitHub PR JSON shapes. The corresponding Phase 3 emission target is a **gunbc-owned render surface** (`dsl/gunbc/digest_render.dag` proposed name), narrow-consuming `dsl/extdeps/github/pulls.dag` for source carriers and `dsl/std/render.dag` + `dsl/std/markdown_render.dag` for rendering primitives — **NOT** a new extdep field/operation.

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

1. **Module header** with consumer receipt** (neutral Phase-3 trigger per Director ratification 2026-05-12 `msg_d1589d17`)**:
   ```
   // ctrl/pr_digests.dag — PR digest helpers (status / conflicts / merge-readiness / URL extraction).
   //
   // Current authority (consumer-side): ctrl/scripts/session-dashboard/{pr_attached_urls,
   // pr_ci_digest, pr_conflict_digest, pr_merge_ready_digest, pr_rest_fallback}.mjs
   //
   // STAGED → AUTHORITY trigger: a Phase-3 trio of
   //   (a) digest source-fact authority (consuming dsl/extdeps/github/pulls.dag PullRequest
   //       as fetched source-of-record; if a narrow GitHub-domain digest-summary record
   //       is needed, lands at dsl/extdeps/github/pull_digest.dag importing pulls.dag —
   //       NOT as new fields on PullRequest itself per Director/Emission-Targets ratification
   //       2026-05-12),
   //   (b) gunbc/std render projection (dsl/gunbc/digest_render.dag composing
   //       dsl/std/render.dag + dsl/std/markdown_render.dag),
   //   (c) named parity-harness gate green
   // PLUS ctrl PR cut-over deleting the 5 .mjs files.
   ```

2. **Carrier types** (DFS via M9 before defining; **MUST grep `dsl/extdeps/github/pulls.dag` before naming any carrier** per substrate-grep-miss correction below):

   **Existing source-fact carriers to import, NOT redeclare** (verified in `pulls.dag` on main 2026-05-12 per Emission Mgr `msg_f9d2bfab`): `PullRequest` (with fields `number / title / body / state / html_url / user / head / base / created_at / updated_at / merged_at / draft`), `PullRequestRef`, `IssueComment`. Plus diff/review operation outputs already in `pulls.dag`.

   **GitHub-source-fact gaps** (CI state, conflict state, mergeability): these are **NOT** assumed fields on `PullRequest`. They are **digest input facts** from existing or future GitHub source-fact carriers. If the worker needs them and they're absent on main, **route a source-fact gap to Emission-Targets Mgr** (`deep-ibex-326`) for placement under `dsl/extdeps/github/*.dag` (narrow `pull_digest.dag` or sibling) — **DO NOT inflate `PullRequest`** and **DO NOT define them on the gunbc side**.

   **Gunbc-side carriers this file owns** (after source-fact carriers resolve):
   - `CiDigestLine` — single-line CI rendering output (gunbc projection)
   - `ConflictDigestLine` — single-line conflict rendering output (gunbc projection)
   - `MergeReadinessVerdict` — closed sum `Ready | NotReady(reasons: List<String>)` — Practice-4 receipt **required**
   - `AttachedUrl` — `{ url: Url, source: AttachedUrlSource }` (reuse `Url` from `std.types`); `AttachedUrlSource` closed sum (`PrBody | CommentThread | ReviewComment | InlineCode`) — Practice-4 receipt **required**
   - `RestFallbackReason` — closed sum (`GraphqlIncomplete | GraphqlRateLimited | GraphqlSchemaUnknown`) — Practice-4 receipt **required**

3. **Service block** with function signatures (no transport; signatures use REAL imported `PullRequest` carrier + neutral source-fact placeholders for CI/conflict input facts pending Emission-Mgr placement):
   ```
   service ctrl.PrDigests {
     fn extract_attached_urls(pr: PullRequest) -> List<AttachedUrl>
     fn render_ci_digest(facts: <ci-source-facts>) -> CiDigestLine
     fn render_conflict_digest(facts: <conflict-source-facts>) -> ConflictDigestLine
     fn merge_readiness_verdict(pr: PullRequest, ci: <ci-source-facts>, conflict: <conflict-source-facts>) -> MergeReadinessVerdict
     fn classify_rest_fallback(error: GitHubErrorShape) -> Option<RestFallbackReason>
   }
   ```
   `<ci-source-facts>` / `<conflict-source-facts>` resolve to typed carriers imported from `dsl/extdeps/github/*.dag` (existing or new under Emission-Mgr placement) — worker MUST NOT invent these on the gunbc side.

4. **Practice-4 receipts** for the 3 closed sums above (per Mgr brief §"Per-worker brief template" item 4 — classification + dissolution pattern + trigger).

## Acceptance gates

1. `dsl/ctrl/pr_digests.dag` parses + compiles (whatever validates `.dag` files in this repo — likely `cargo test -p v3-compiler` covers it).
2. Practice-4 receipts on `MergeReadinessVerdict`, `AttachedUrlSource`, `RestFallbackReason` — each names classification + dissolution-pattern + trigger.
3. Consumer-receipt header cites all 5 ctrl `.mjs` files + names the **neutral 3-part Phase-3 trigger** (digest source-fact authority + gunbc/std render projection + named parity harness green) per Director ratification `msg_d1589d17`. **DO NOT** name `dsl/extdeps/github/digest_render.dag` or any new render-field on `PullRequest` — both placements were ruled out by Director + Emission Mgr 2026-05-12.
4. Cost-of-change check: adding a new `RestFallbackReason` variant touches **only** this file (no parallel registry / no extdep update / no consumer-side fix-up). If it touches more, surface to Mgr — that's a substrate-shape signal.
5. M9 DFS audit comment: for each carrier, name the existing `dsl/std/` or `dsl/extdeps/github/` primitive it attaches to (or document why it must be new — must be a substantive structural reason, not "didn't find one").
6. Doc-only — zero emission code; zero TS-side changes.

## STOP/PING criteria

- **STOP** if required GitHub source facts (CI state, conflict state, mergeability inputs) are absent on main — **route an emission/source-fact gap to Emission-Targets Mgr** (`deep-ibex-326`) for placement under `dsl/extdeps/github/*.dag` (narrow `pull_digest.dag` or sibling). **DO NOT inflate `PullRequest` with render/digest fields**; **DO NOT** define source facts on the gunbc side. Per Director ratification `msg_d1589d17`: the worker either narrows to existing `PullRequest` fields (number/title/body/state/html_url/user/head/base/created_at/updated_at/merged_at/draft + diff/review op outputs) OR waits for source-fact placement.
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
