# Planning candidates on Fabric storage

Owner decisions: acceptance updates live issue state immediately; dashboard and workers read
accepted state. Fabric storage owns persistence. Repository merge is not the editorial apply path.

## Implemented flow

The issue detail links to a typed planning editor. An authenticated human can propose ticket
field changes, independently edit parent and blockers, and add draft children. Saving publishes
an immutable candidate containing its base snapshot, author and work-request identity. Reviewing
is read-only; acceptance is an explicit CSRF-protected POST by the current accountable Google
principal. Legacy email-only identity does not confer acceptance authority.

Candidate publication uses a separate create-only Fabric head, so saving does not stale its own
issue base. Same-identity/same-body retries recover the existing proposal; changed content refuses.
Acceptance validates the current base, full resulting relation graph and draft identities, then
publishes children, their accountable assignments, the parent edit and acceptance receipt with one
issue-head CAS. Concurrent different candidates cannot both win. A committed candidate retry
returns its original result even after later issue events; a lost CAS never silently rebases.

Stored editorial state retains typed ticket fields and an independent nesting relation. Historical
records without these optional extensions retain their canonical bytes; their existing title/body
normalize into headline/brief with unrecorded fields empty. Executable bindings and verification
metadata cannot be edited through this interface. Draft children remain non-dispatchable until an
execution contract is separately bound.

Dashboard observations and worker dispatch consume the same live document projection. Accepted
brief and plan reach worker prompts. Children derive from membership; blocking derives from
dependency edges. Missing/corrupt/unreadable state refuses instead of reverting to source text.
Acceptance refreshes the current instance's pre-rendered dashboard. A refresh failure reports that
accepted state is durable and refresh is pending; retrying acceptance retries refresh. Other
instances observe through their normal belt refresh cadence.

## Scope and dependency

This branch is stacked on protected-state PR #12465 at
`ada7daefc28696ce4ba242437cd26daf61bd0bab`. Its HOLD remains in force. Runtime binding uses
`fabric_state_storage_binding`, without a literal host or fallback store. No credentials, real
protected storage, live dashboard, DNS, deployment or merge were changed.

This implements the human editor vertical. It does not implement a new Ask Fabric planning-worker
dispatch workflow. The preserved broader work contract remains in `planning-work-contract.md`.
Production commissioning and source-to-store migration belong to the protected-state dependency.

## Validation and remaining gates

See `docs/receipts/planning-candidate-fabric/editor-validation.md` for evidence and exact limits.
Targeted controls cover review, canonical storage, actual acceptance, authenticated HTTP admission,
legacy event identity, task projection and the shared worker consumer. Browser and independent
process race controls use real temporary Fabric file storage with fixture authentication.

The full production serve compile and broad imported integration run exceeded the existing 6 GiB
validation cap (exit 137). They are unresolved integration gates, not passing checks. A protected
production cookie/session round trip, commissioned storage, and deployment have not been tested.
The change should remain draft until integrated qualification and dependency landing are complete.
