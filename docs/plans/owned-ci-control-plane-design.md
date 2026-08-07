# Owned CI control plane v0

**Status:** v0 implementation lane (operator-directed 2026-08-06).

## Why now

GitHub Actions webhook delivery is not a completeness authority: failed deliveries are not automatically redelivered, and platform incidents can throttle push/PR triggers for hours while PRs accumulate approvals with checks absent.

## Responsibility split

| GitHub keeps | gunbc owns |
|---|---|
| Source hosting | Trigger reconciliation |
| Pull requests and reviews | Durable queue |
| Branch protection | Scheduling |
| Visible check result surface | Execution |
| | Artifacts, receipts, logs, run UI |
| | Recovery |

Poll-first: a webhook may remain a latency optimization; it may never be the completeness authority.

## v0 shape

- **Subject discovery:** local bare mirror; periodically fetch `main` and PR refs; compare against a durable subject ledger.
- **Queue:** filesystem run records claimed by atomic rename plus an expiring lease.
- **Executor:** one process, one exact-SHA worktree, sequential `build → regen → floor`.
- **Receipts:** append-only local run/stage receipts and logs under `.gunbc/owned-ci/`.
- **UI:** `GET /ci` and `GET /ci/run/{id}` on the gunbc-served surface.
- **Projection:** one required Check Run (`gunbc-ci`) updated `queued → in_progress → completed`.

Auto-heal mutation is out of scope for v0: owned CI may detect and report generated-artifact drift; pushing repairs stays a separate trusted action.

## Medium-neutral execution plan

`gunbc.ci_spec` owns gates, witness entries, discovery, diff policy, and deploy stages. `gunbc.ci_workflow` is the GitHub Actions projection only.

```
CiExecutionPlan { subject, stages, check_name, details_url, budgets }
  → GitHub Actions YAML (temporary sibling)
  → local owned executor (new primary)
```

If you find yourself parsing `ci.yml`, you have taken the wrong branch.

Subject identity reuses the merge-admission grain: exact head SHA, base-tree identity, gate-roster identity, conclusion.

## Check publication honesty

A local run that finishes while GitHub's API is unavailable is:

`LocalVerdictPersisted + CheckPublicationPending`

not "failed", and emphatically not "successfully published". A publication reconciler attaches the result when GitHub recovers.

## Scope for v0

- Same-repository PR → main, push → main, explicit operator rerun.
- Fork PRs: refuse or sandbox without secrets.
- Merge queue stays on the old path until the single-executor owned path is stable.

## Sequence

design → poll/queue → one local executor → Checks projection → shadow comparison against Actions → branch-protection cutover.

Shadow before cutover is not optional.

## Acceptance (v0)

- **Consumer:** `ci_control_plane` bin on a host with a git mirror, `GITHUB_TOKEN` with `checks:write`, and `gunbc serve` for `/ci` UI.
- **Green:** poll discovers head SHA; run record claimed by atomic rename under expiring lease; one executor runs `build → regen → floor` in one exact-SHA worktree; append-only receipts; `/ci/run/{id}` renders; one Check Run transitions `queued → in_progress → completed`.
- **RED controls:** expired lease is reclaimed (not stranded); GitHub API failure at publication yields `LocalVerdictPersisted + CheckPublicationPending` (never "failed", never "published"); poll without GitHub read refuses rather than inferring no PR subjects changed; fork PRs refuse.
- **Receipt location:** `.gunbc/owned-ci/runs/{id}/`, `receipts/{id}/{stage}.json`, `target/owned-ci-run-receipt.txt`.
