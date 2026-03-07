# Design: Ambient Feedback Model

**Status**: Draft
**Date**: 2026-03-06
**Related tasks**: SDLC `B1`, roadmap `AS2-AS7`
**Related docs**: `docs/design/sdlc/ambient-intellectual-roadmap.md`, `docs/design/modeling/intellectual-pipeline-kernel.md`, `TODO/sdlc.md`

## Motivation

An ambient SDLC system is not trustworthy if comments and reviews are treated as
best-effort text. The system needs a durable way to ingest, classify, rediscover,
act on, and explicitly close human feedback.

The failure mode to avoid is easy to state:

> a human leaves a comment or review finding during execution, and the system
> never turns that into a tracked obligation.

This doc resolves the GitHub-shaped part of that problem.

## Desired Outcome

We want a model where:

1. issue comments, PR comments, review comments, and review summaries are captured
   durably,
2. critique and approval are normalized into typed records,
3. critique becomes an explicit outstanding obligation,
4. webhook loss causes delay, not loss,
5. the system can show whether feedback was seen, acted on, and closed.

## Satisfaction Criteria

This design is good enough only if:

1. every feedback source has a stable identity and a stable revision identity,
2. comment edits or review updates reopen or update the durable record rather than
   disappearing,
3. approval and critique never collapse into the same signal,
4. closure is auditable through linked response or code artifacts,
5. anti-entropy can reconstruct the outstanding feedback set from provider state.

## Failure Mode If Wrong

We should assume this design failed if:

1. feedback identity depends on LLM extraction rather than provider identity,
2. edited comments create duplicate obligations or silently overwrite prior work,
3. approval closes critique accidentally,
4. comments are marked resolved without a linked response artifact,
5. the pipeline cannot recover from missed webhooks.

## Scope / Non-Goals

This doc does not:

1. split one comment into multiple fine-grained findings at ingest time,
2. require perfect semantic understanding of arbitrary human language before
   persistence,
3. define the UI for reports or dashboards,
4. define provider adapters beyond GitHub-shaped surfaces.

This doc does:

1. define the canonical ingestion unit,
2. define the durable feedback and obligation model,
3. define closure and rediscovery rules for ambient SDLC.

## Resolved Decisions

### F1. The canonical ingestion unit is a source object, not an extracted finding

Version 1 tracks feedback at the granularity of the provider source object:

1. issue comment,
2. pull request comment,
3. review comment,
4. review summary.

We do **not** assign durable identity to LLM-extracted sub-findings in v1. That is
too unstable to be the primary key.

### F2. Feedback has stable identity plus revision identity

Every observed feedback item has:

1. `feedback_id` — stable across edits, derived from provider + source kind +
   source object id,
2. `revision_key` — changes when the material content or review state changes.

Rule:

1. `feedback_id` identifies the durable obligation,
2. `revision_key` identifies one observed version of that source object,
3. a new revision reopens or updates the same obligation instead of creating a new
   primary obligation id.

### F3. Capture happens before classification

The system must persist the source record before deciding whether it is critique,
approval, or noise.

That gives a fail-closed ingestion path:

1. capture raw feedback,
2. normalize source references,
3. classify into typed disposition,
4. create approval signal and/or critique obligation,
5. report unclassified records explicitly instead of dropping them.

### F4. Approval and critique are different meanings

They are not opposites and they do not cancel each other implicitly.

Rules:

1. approval may satisfy an approval gate,
2. critique may create or reopen an obligation,
3. approval does not close critique,
4. critique does not count as approval.

### F5. Closure is explicit and auditable

`Addressed` means the pipeline produced a linked response or code/result artifact.
`Closed` means the obligation is no longer active for one explicit reason:

1. source-level resolution exists and is observed,
2. a newer revision superseded the prior one,
3. the enclosing work concluded and the feedback was carried into that conclusion,
4. an operator explicitly overrode closure.

### F6. Anti-entropy owns correctness

Webhooks and review events accelerate ingestion. They are not the only discovery
path.

The feedback loop remains correct by:

1. scanning provider state for unresolved feedback,
2. comparing latest provider revision to stored `revision_key`,
3. emitting missing critique or approval events when gaps are found.

## Canonical Types

```text
FeedbackSourceKind =
  IssueComment
  PullRequestComment
  ReviewComment
  ReviewSummary
```

```text
FeedbackDisposition =
  Critique
  Approval
  Informational
  Administrative
  Unclassified
```

```text
FeedbackStatus =
  Seen
  InProgress
  Addressed
  Closed
```

```text
FeedbackCloseReason =
  ResolvedAtSource
  SupersededByNewRevision
  ClosedByConclusion
  OperatorOverride
```

```text
FeedbackSourceRef {
  provider: NonEmptyStr
  owner: NonEmptyStr
  repo: NonEmptyStr
  issue_id: NonEmptyStr?
  pull_request_id: NonEmptyStr?
  review_id: NonEmptyStr?
  comment_id: NonEmptyStr?
  thread_id: NonEmptyStr?
  commit_sha: NonEmptyStr?
  path: String?
  line: Int?
  url: Url?
}
```

```text
FeedbackRecord {
  feedback_id: NonEmptyStr
  revision_key: NonEmptyStr
  source_kind: FeedbackSourceKind
  source_ref: FeedbackSourceRef
  disposition: FeedbackDisposition
  author: NonEmptyStr?
  body: String
  review_state: String?
  observed_at: Timestamp
  updated_at: Timestamp
  raw_payload_hash: NonEmptyStr
}
```

```text
FeedbackObligation {
  obligation_id: NonEmptyStr        // equal to feedback_id in v1
  feedback_id: NonEmptyStr
  latest_revision_key: NonEmptyStr
  status: FeedbackStatus
  close_reason: FeedbackCloseReason?
  linked_run_key: NonEmptyStr?
  linked_artifact_ids: List<NonEmptyStr>
  reopened_count: Int
  created_at: Timestamp
  updated_at: Timestamp
}
```

## Normalization Rules

### Identity

```text
feedback_id =
  hash(provider, source_kind, repo_scope, stable_source_object_id)
```

```text
revision_key =
  hash(
    feedback_id,
    normalized_body,
    review_state,
    thread_anchor,
    updated_at_or_submitted_at
  )
```

### Review-state mapping

Provider review state maps as follows:

1. `APPROVED` -> `Approval`
2. `CHANGES_REQUESTED` -> `Critique`
3. `COMMENTED` -> `Critique` if text requests action, otherwise `Informational`
4. unknown or parser failure -> `Unclassified`

Bodyless state transitions still produce records. The state itself is meaningful.

### Edit behavior

If the same source object changes materially:

1. persist the new `FeedbackRecord`,
2. update `latest_revision_key`,
3. reopen the obligation to `Seen` unless it is already active,
4. increment `reopened_count` if it had reached `Addressed` or `Closed`.

## Obligation Lifecycle

```text
Seen -> InProgress -> Addressed -> Closed
```

Reopen rule:

```text
Addressed -> Seen     (new revision)
Closed -> Seen        (new revision)
```

Minimum evidence for each state:

1. `Seen`: captured and classified, but no linked response yet
2. `InProgress`: worker/agent run linked to the feedback
3. `Addressed`: linked response artifact, code change result, or explicit response
4. `Closed`: one `FeedbackCloseReason` recorded

## Response and Closure Rules

An obligation may move to `Addressed` only when at least one linked artifact exists:

1. response comment posted by the pipeline,
2. design revision artifact,
3. code change / commit / PR result,
4. test or evidence artifact that directly answers the critique.

`Closed` requires one explicit close reason:

1. `ResolvedAtSource` — review thread resolved or equivalent provider signal
2. `SupersededByNewRevision` — the source object changed and the old revision is no
   longer the active target
3. `ClosedByConclusion` — the enclosing work concluded with the feedback linked in
   the conclusion artifact
4. `OperatorOverride` — human operator closed it manually

For source surfaces without native resolution semantics, `Addressed` may be the
normal steady state until either a conclusion artifact or operator action closes it.

## Anti-Entropy Rules

Rediscovery must scan:

1. open issue comments since the last watermark,
2. open PR comments and review comments,
3. review summaries and review state changes,
4. unresolved review threads when the provider exposes them.

For each discovered source object:

1. compute `feedback_id`,
2. compute current `revision_key`,
3. compare against stored `latest_revision_key`,
4. emit missing critique or approval work when the revision is new or absent.

Lost webhooks therefore cause latency, not semantic loss.

## Day 1 for Ambient Trust

The minimum trustworthy version is:

1. one durable record per source object,
2. one durable obligation per feedback source object,
3. explicit `Addressed` vs `Closed`,
4. linked response artifacts,
5. anti-entropy scan proving rediscovery.

Sub-finding extraction can be layered on later if it adds value, but it must not
replace the provider-shaped identity model defined here.
