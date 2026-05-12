# Ctrl-Migration First-Wave Subsystem Briefs

**Status**: READY-FOR-DISPATCH once the Subsystem-Modeling Mgr exists.

**Authority**: Ctrl-Migration project plan §3 from PR #2775. These items are independent of Phase 1 process-algebra substrate and can proceed immediately as staged service contracts.

## Shared Brief Contract

Every worker owns one `dsl/ctrl/<subsystem>.dag` file unless the manager explicitly folds utility helpers into a consumer subsystem.

Acceptance gates:

1. Declare service contracts, carriers, projections, and pure helper signatures for the subsystem.
2. Name every current ctrl TS source file that remains runtime authority.
3. Add Practice 4 receipts for every enum/sum with at least two variants.
4. Mark the file 🟡 STAGED and name the consumer receipt that upgrades it to authority.
5. Avoid new std-like carrier names unless the worker provides an M9 DFS non-reuse proof.

## Wave 1 Items

1. **Inbox Delivery** (`dsl/ctrl/inbox.dag`)
   - Source authority: `scripts/session-dashboard/inbox_policies.mjs`, `inbox_schema.mjs`, and `INBOX_DESIGN.md`.
   - Reuse the existing demo shape from `research/market/viability/demos/agent-ctrl-session-dashboard/inbox_delivery_slice.dag` where applicable.
   - Consumer receipt: generated inbox delivery projection plus parity tests over existing inbox delivery fixtures.

2. **Session Lifecycle** (`dsl/ctrl/session_lifecycle.dag`)
   - Source authority: `sessions_schema.mjs`, `watcher.mjs`, `container_runtime.mjs`, `runtime_tmux.mjs`, `runtime_helpers.mjs`, `SESSION_LIFECYCLE.md`, and `CONTAINER_LIFECYCLE.md`.
   - Model spawn/idle/archive emergence as state projections, not as textual scheduler convention.
   - Consumer receipt: generated lifecycle classifier matches dashboard session rows across open/idle/archived examples.

3. **PR Digests** (`dsl/ctrl/pr_digests.dag`)
   - Source authority: `pr_attached_urls.mjs`, `pr_ci_digest.mjs`, `pr_conflict_digest.mjs`, `pr_merge_ready_digest.mjs`, and `pr_rest_fallback.mjs`.
   - Preserve digest subject identity: repo, PR number, head SHA, and observed artifact source.
   - Consumer receipt: generated digest output matches current digest formatter fixtures.

4. **Work Advancement Prompts** (`dsl/ctrl/work_prompts.dag`)
   - Source authority: `work_advancement_prompts.mjs`.
   - Model prompt components as typed inputs; avoid treating the prompt body as the only authority.
   - Consumer receipt: generated prompt rendering matches existing expected messages.

5. **Analyses Pipeline** (`dsl/ctrl/analyses.dag`)
   - Source authority: `analyses_api.mjs`, `analyses_sync.mjs`, `analyses_sync_targets.mjs`, and `analyses_table.mjs`.
   - Preserve run identity, target identity, sync status, and artifact links as separate axes.
   - Consumer receipt: generated analysis sync table matches existing DB query results.

6. **CI Integration** (`dsl/ctrl/ci.dag`)
   - Source authority: `ci.mjs`.
   - Model check suite state and merge-blocking decision as typed projections.
   - Consumer receipt: generated CI gate result matches current dashboard CI classifier over sample GitHub payloads.

7. **API Reviewer** (`dsl/ctrl/api_reviewer.dag`)
   - Source authority: `scripts/api-reviewer/review-one.mjs`, `review-with-cli.mjs`, and `review-meta-with-cli.mjs`.
   - Model provider selection, OAuth/API-key policy, review invocation, stale-PR exit, and posting contract.
   - Consumer receipt: generated invocation contract matches dry-run CLI behavior for codex/claude/cursor/openai-pro.

8. **Utility Helper Consolidation**
   - Source authority: `disk_pressure.mjs`, `effort_picker.mjs`, `parse_int_env.mjs`, `transcript_excerpt.mjs`, plus helper uses inside the above subsystems.
   - Default disposition: fold helpers into the consuming subsystem file rather than creating a standalone utility substrate.
   - Consumer receipt: helper behavior parity is covered by the consuming subsystem tests.

