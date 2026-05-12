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

1. **Module header** with consumer receipt:
   ```
   // ctrl/pr_digests.dag — PR digest helpers (status / conflicts / merge-readiness / URL extraction).
   //
   // Current authority (consumer-side): ctrl/scripts/session-dashboard/{pr_attached_urls,
   // pr_ci_digest, pr_conflict_digest, pr_merge_ready_digest, pr_rest_fallback}.mjs
   // STAGED → AUTHORITY trigger: dsl/gunbc/digest_render.dag (Phase 3 gunbc-owned render
   // surface, NOT an extdep) lands + ctrl PR cut-over deletes the 5 .mjs files + parity
   // test passes. GitHub source carriers imported from dsl/extdeps/github/pulls.dag;
   // rendering primitives from dsl/std/render.dag + dsl/std/markdown_render.dag.
   ```

2. **Carrier types** (DFS via M9 before defining; reuse `dsl/extdeps/github/pulls.dag` carriers where they exist):
   - `PrDigestInputs` — composite of CI-state, conflict-state, branch-state needed for verdict
   - `CiDigestLine` — single-line CI rendering
   - `ConflictDigestLine` — single-line conflict rendering
   - `MergeReadinessVerdict` — closed sum: `Ready | NotReady(reasons: List<String>)` — Practice-4 receipt **required**
   - `AttachedUrl` — `{ url: String, source: AttachedUrlSource }` where `AttachedUrlSource` is closed sum (`PrBody | CommentThread | ReviewComment | InlineCode`) — Practice-4 receipt **required**
   - `RestFallbackReason` — closed sum classifying when REST fallback triggers (`GraphqlIncomplete | GraphqlRateLimited | GraphqlSchemaUnknown`) — Practice-4 receipt **required**

3. **Service block** with function signatures (no transport):
   ```
   service ctrl.PrDigests {
     fn extract_attached_urls(pr: GithubPr) -> List<AttachedUrl>
     fn render_ci_digest(state: CiState) -> CiDigestLine
     fn render_conflict_digest(state: ConflictState) -> ConflictDigestLine
     fn merge_readiness_verdict(inputs: PrDigestInputs) -> MergeReadinessVerdict
     fn classify_rest_fallback(error: GithubError) -> Option<RestFallbackReason>
   }
   ```
   (Final signature shape DFS-derives from existing `dsl/extdeps/github/pulls.dag` `GithubPr` / `CiState` / `ConflictState` carriers if present; if absent, surface to Mgr — those are extdep-side substrate, not subsystem-side.)

4. **Practice-4 receipts** for the 3 closed sums above (per Mgr brief §"Per-worker brief template" item 4 — classification + dissolution pattern + trigger).

## Acceptance gates

1. `dsl/ctrl/pr_digests.dag` parses + compiles (whatever validates `.dag` files in this repo — likely `cargo test -p v3-compiler` covers it).
2. Practice-4 receipts on `MergeReadinessVerdict`, `AttachedUrlSource`, `RestFallbackReason` — each names classification + dissolution-pattern + trigger.
3. Consumer-receipt header cites all 5 ctrl `.mjs` files + names `dsl/gunbc/digest_render.dag` (gunbc-owned render surface) as the Phase 3 trigger. **DO NOT** name `dsl/extdeps/github/digest_render.dag` — that placement was corrected pre-dispatch by Emission-Targets Mgr (see §"Why catalog #8 first" placement-discipline note).
4. Cost-of-change check: adding a new `RestFallbackReason` variant touches **only** this file (no parallel registry / no extdep update / no consumer-side fix-up). If it touches more, surface to Mgr — that's a substrate-shape signal.
5. M9 DFS audit comment: for each carrier, name the existing `dsl/std/` or `dsl/extdeps/github/` primitive it attaches to (or document why it must be new — must be a substantive structural reason, not "didn't find one").
6. Doc-only — zero emission code; zero TS-side changes.

## STOP/PING criteria

- **STOP** if `dsl/extdeps/github/pulls.dag` lacks `GithubPr` / `CiState` / `ConflictState` carriers — that's an extdep-side substrate gap; surface to Mgr who routes to Substrate Mgr or Emission-Targets Mgr (`deep-ibex-326`). Per Emission Mgr placement-discipline 2026-05-12: any new GitHub-source-of-record carriers MUST land in `dsl/extdeps/github/` (provider facts), never in this gunbc-side subsystem file.
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
