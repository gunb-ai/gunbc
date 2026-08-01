# Sole modeled publisher — model + shadow design

Status: DRAFT (slice 1 — model + shadow only; 2026-08-01 recut on #7591).

**Honest framing:** ordinary dashboard sessions do **not** yet lose public-write capability in this slice. Credentials and push machinery are unchanged. Live credential removal + transport rebinding is a separate live slice; that slice's acceptance act is when an ordinary session's `git push --no-verify` finds no credential/transport.

Origin: priority-assessment directive section 8 — publication safety outranks reveal UX. The per-file placement gate/roster is deleted (#7591); publication enforcement returns through derived projection policy, not pathname census.

## Retained architecture (PR B)

| artifact | role |
|---|---|
| `extdeps.git.publication_transport` | Modeled `GitPushRequest` / `GitCommitRequest`; argv is one transport handler |
| `gunbc.publication_policy` | Subtree prefix → disposition; default `WithholdPrivate`; most-specific wins |
| `gunbc.public_projection` | `PublicProjectionPlan` derived from policy over authored subjects |
| `gunbc.publication_decision` | `PublicationDecisionReceipt` binds content identity, not pathnames |
| `gunbc.publication_admission_delta` | Change-local admission (current vs candidate projection) |
| `tools.publication_publisher` | Sole publisher identity + typed refusals over admitted projection |
| `tools.publication_push_shadow` | Read-only replay ledger; reports, does not green |

## Authority model

**WHO:** exactly one `sole_modeled_publisher_session` (`gunbc-publication-publisher`) may consume an admitted `PublicProjectionPlan`.

**WHAT:** `PublicationPolicy` derives inclusion from subtree rules — never `public_file_publish_grants` and never the unrestricted authored tree.

**Receipt:** exceptional decisions use `PublicationDecisionReceipt { subject_identity, content_identity, audience, policy_version, authority }`. Changed sensitive content requires a fresh receipt.

**Admission grain:** change-local only — newly public / newly withheld / audience widenings / private-to-public references / policy changes / digest mismatch. Whole-projection audit is a separate centrally-owned main-health job.

## Deferred to live slice

- ordinary session credential/transport removal
- content absent from public repo **object database** before admission (not merely absent from `main`)
- live receipt binding `{ source_revision, policy_version, projection_digest, destination, pushed_ref }`
- repeating an identical policy rule idempotent or refused at construction (policy carrier lands the construction wall; live transport rebind is separate)

## Staged migration

### Stage 0 — model + shadow (this PR)

Model write operations, policy/projection/decision carriers, publisher authority, shadow replay ledger.

**Trigger to Stage 1:** shadow receipt shows stable refusal taxonomy on live specimens; authorization kernel (#7586) landed.

### Stage 1 — staging remote

Sessions push to private staging only; publisher mirrors admitted projections.

### Stage 2 — enforcement gate

Bind `git.PublicationTransport` dispatch to `PublicationAuthority`.

### Stage 3 — downstream mirror

Public GitHub repo is generated projection only.

## Coordination

- **#7591 (vivid-newt-418):** deletes placement gate/roster — base branch for this recut
- **#7586 (vivid-crab-386):** authorization kernel — PR A, not this lane
- **#7552 (proud-swift-104):** belt producer wiring — out of scope for PR B
