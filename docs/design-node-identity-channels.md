# Authority: Node Identity Channels — fields on `Node` and equality participation

> **Status: AUTHORITY DOC (P2 single-authority).** This doc owns two questions that three
> active lanes were about to answer independently: **what fields `Node` carries beyond
> `{ kind, children }`**, and **which of them participate in structural equality / hashing**.
> Any PR that adds, removes, or re-scopes a Node-carried field lands an edit to the table
> below **in the same change**, and reviewers check the PR against it. No other doc — and no
> consumer — declares equality participation for a Node field.
>
> Why this exists: BRAND (#4581 `binding_id`), PROV (`design-provenance-span.md` occurrence
> id), and COMPREP/T-9 (`design-computation-representation.md` callee/BindsTo references)
> each touch the Node shape, with *different* equality treatments. Distributed ownership of
> that policy is exactly the per-consumer equality exception PROV's own design warns against
> — caught in portfolio review (2026-06-10) before any of the three landed.

## The channel table (the policy)

| Channel | Carrier on `Node` | Producer (the one seam) | In structural equality / hash? | Status |
|---|---|---|---|---|
| **Brand identity** (A3/A4 nominal distinctness) | `binding_id` field | #4581 authority-direct stamping at type-env registration / resolve | **YES** — it *is* semantic identity; excluding it would re-open the spelling-collision false-accept | building (#4581 — the linchpin) |
| **Reference** (use→def; callee; the dependency classifier's `BindsTo`) | **no new field** — a substrate fact carried on the `binding_id` channel (the T-9 rider: resolve materializes the use→def relation against the def's `binding_id`) | resolve, same seam as stamping | participates **as structure** (it is an edge/fact, not a field exclusion question) | queued immediately behind #4581 |
| **Provenance** (source anchoring) | opaque `occurrence_id` field (`OccurrenceId` in `v2.std.node`; sole mint `v2.std.occurrence_id.alloc_occurrence_id`) | parse, allocator-issued (PROV wave 1); off-tree `SpanIndex` carries `OriginEvent` (design-provenance-span.md §4.5) | **NO** — bookkeeping, never identity; two occurrences of `1 + 1` stay structurally equal | building (PROV A-pivot Phase 0 model — field on `Node` lands Phase 1+2 atomically with #4604 span revert) |

## Rules

1. **#4581 defines the channel; everyone else rides it.** The allocator pattern + stamping
   seam land once, in #4581. The T-9 reference rider reuses that seam (same PR or
   immediately after). PROV's occurrence-id allocator copies the *pattern* but is a distinct
   id space — provenance ids and binding ids never share an allocator or a field.
2. **One landing order, one field-moment each:** `binding_id` (#4581) → T-9 rider →
   `occurrence_id` (PROV GO 2026-06-10; Phase 0 lands substrate types + allocator; the
   `Node` field lands Phase 1+2 atomically with #4604 span revert).
3. **Equality participation is implemented in exactly two places** — the Rust `Value::eq`
   authority (v2 interpreter; the #4564 single-equality discipline) and the `.dag`
   structural-equality predicate (`exact_structural_equality_zip_fold`) — each citing this
   table. A consumer implementing its own include/skip rule for any Node field is a P2
   violation, full stop.
4. **Hashing follows equality.** Content addressing (`atom_identity_hash` lineage) treats
   fields exactly as the table does: `binding_id` hashes, occurrence id does not.
5. **Determinism obligation (flag to #4581):** because `binding_id` participates in equality
   and hashing, the `BindingIdAllocator` must be **deterministic** (same sources → same ids)
   or `binding_id` must be provably absent from emitted artifacts — otherwise DB-8 and the
   self-host fixed-point compare (`design-self-host-fixed-point.md` §3) inherit
   nondeterminism through the back door. Occurrence ids are per-compile and never serialize
   into artifacts, so they carry no such obligation.

## Consumers (per channel — E-10)

- `binding_id`: A3/A4 brand claims (#4533/#4554 suites), the #4581 adversarial suite.
- Reference/BindsTo: the dependency classifier (`dependency.dag:126` — closes its own T-9
  marker), the `structural_resolution` lens (today fixture-fed), the termination checker's
  call graph (`design-termination-checker.md` §4.3 prerequisite), COMPREP wave 0 callee refs.
- Occurrence id: AFF real-input, WRITE, SYN (the three PROV lanes).

This doc itself carries no code; its consumer is the reviewer of any Node-shape PR — the
check is mechanical (does the PR's field/equality behavior match the table; if not, the PR
updates the table first or is wrong).

## Cross-references (the three lanes now point here, not at each other)

`design-provenance-span.md` §4.1/Q-P2, `design-computation-representation.md` wave 0,
`design-termination-checker.md` §4.3 prerequisite fix. The #4581 design docs (ctrl#4579 +
Amendment 4) predate this doc; their next amendment should cite it as the field-policy home.
