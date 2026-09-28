# Planning candidates on Fabric storage

Owner decisions, 2026-09-28: accepting a candidate updates live issue state; both dashboard and
workers read that accepted revision. Use Fabric storage. Repository changes are not the apply
route for these editorial updates.

## Implemented

The candidate review model and ten controls from `d327f501b55` are preserved here. Typed field
identities and preview helpers are carried from Cut 2's status authority; there is no field-name
lookup based on presentation labels.

`gunbc.roadmap_planning_candidate_storage` publishes immutable candidate objects through the
existing `FabricStorageBinding` client and its create-only head CAS. The canonical object body
includes the full base snapshot, author, work request, field/relation proposals and draft children.
A proposal has a separate head from issue state, so publishing it cannot stale its own issue base.
Same-identity/same-body retries recover the committed object; different content under an existing
identity refuses. Conflict comparison uses exact body bytes, not only the structural digest.
An absent/unreachable/corrupt store never becomes an empty successful read or a local fallback.

The adapter is a persistence boundary, not authorization: authenticated proposal admission must
precede it. It currently returns stored canonical bytes; a typed read decoder and HTTP integration
remain to be implemented. No production route calls it yet.

## Dependency and integration

This isolated branch is based on the protected-state sibling at
`ada7daefc28696ce4ba242437cd26daf61bd0bab` (#12465). Its HOLD is unchanged. No sibling checkout,
auth handler, credential, live storage, server or deployment was changed.

Reuse these existing authorities:

- `fabric_state_storage_binding` for the commissioned protected-state endpoint; no literal host.
- `roadmap_event_snapshot_read` for one consistent issue-history snapshot.
- `roadmap_event_append_snapshot` for publication against the observed Fabric head.
- `roadmap_task_projection` for mutable editorial state after its one-time seed.

The current stored editorial task has title/body/state/prerequisites, but lacks the full typed
ticket fields and separate nesting relation required by candidates. Its current read consumers
include the issue page and editorial HTTP route; workers still need the shared accepted-state
projection. Do not claim a dashboard-only override completes the owner's directive.

Remaining implementation:

1. Extend stored editorial state with typed ticket fields and nesting, preserving historical
   event byte identity and keeping executable contract bindings immutable.
2. Decode stored candidates; admit authenticated proposals and bind their identity and author
   to the generic planning work request. Validate complete graph targets/cycles and draft IDs.
3. Atomically apply the accepted candidate plus its operation receipt through the event carrier.
   Return the original result on retry before stale checks, including after later changes.
4. Read one accepted-state projection in dashboard and worker contract/alignment inputs.
5. Add the editor/review UI and authenticated, CSRF-protected POST routes. GET never applies.
6. Execute concurrent acceptance, browser, worker-consumer and served-storage controls. Coordinate
   protected-state commissioning and source-to-store migration before any deployment.

## Validation

Ten review controls and two storage controls passed using binary SHA-256
`e6d8571156538304137139853153e1d30f77f99ba71d8c76ccd3a943b8875e5b`.
The storage inhabitance control executes real file-backed Fabric put, head CAS and closure read
in a temporary directory: absent, publication, readback, retry, changed-payload refusal and unchanged
original readback. The other control verifies unplaced-store refusal.

The earlier shared-branch binary cannot parse this dependency's existing `MachineWidth` declaration;
that run is not counted as validation. The compatible binary passed after correcting one new
refinement cast (`1 as Nat` to the nonnegative literal `1`). No production source was altered to
accommodate the compiler. Byte-identical source closure and its manifest are local under
`target/planning-validation`; retained outputs are in `docs/receipts/planning-candidate-fabric/`.

These results do not establish a running editor, a protected served round trip, accepted issue
application, concurrency qualification, dashboard/worker integration or deployment.
