# Allocation demo state migration: implementation and validation

Integration tree: `codex/allocation-demo`, base `e66a6a65ae5`. No live history has been migrated and no serving checkout has been changed by this lane.

## Runtime carrier

`roadmap_event_carrier` stores original event bytes as immutable fabric objects, with one CAS event-set manifest head (`roadmap-events-v1`). Current authenticated events use v4; v1/v2 legacy identity bytes and v3 authenticated IDs remain readable. Editorial create/revise, comments, claims and progress share each node's existing event chain. Mutable editorial state does not replace executable code-contract identity or completion receipts.

Production uses `RoadmapMigratedService`. An absent initialization marker refuses reads and writes; it cannot appear as an empty tracker. `RoadmapImportTarget` is an explicit migration/test layout, not a production fallback.

## Restartable migration sequence

1. Read a pinned Git object via `roadmap_events_import_pinned`; uncommitted files do not participate. Decode every path and event ID, preserve original bytes, publish through CAS, and compare readback.
2. The importer indexes its immutable provenance receipt at `roadmap-events-import-receipt-v1`. `roadmap_import_recover` checks that receipt against the pinned source and verifies all original bytes remain in the current manifest. This works after seed events or later edits; do not rerun the exact-history raw import after seeds.
3. `roadmap_tasks_seed_authored` adds normal editorial creation events and verifies every authored node's original identity/code-contract binding. Existing editorial fields are not overwritten on retry.
4. `roadmap_tasks_finish_migration` rechecks the indexed import receipt and seed identities, captures the actual current manifest as the baseline, publishes `roadmap-events-initialized-v1` linking both receipt and baseline, then reads through the production gate.
5. Subsequent production reads require the current observed head to retain every baseline object. Missing, corrupt or unreadable migration evidence refuses.

The protected fabric transport is being reconciled with the existing restricted local socket work by the integration owner. Neither these tests nor an initialization marker constitute installation/cutover authorization.

## Validation receipts

All tests below used copied, byte-identical import closures, bounded user scopes, and `MemorySwapMax=0`.

- Request/model: 4 pure witnesses passed (2 GiB); `/tmp/allocation-request-closure-validation.log`, `/tmp/allocation-request-remaining.log`.
- Editorial model: 5 original pure witnesses passed (1 GiB); `/tmp/editorial-five-validation.log`. Two additional full-graph tests passed in `/tmp/editorial-graph-validation.log`, using the existing graph authority for unknown-reference/cycle admission.
- Carrier and migration gate: 7 pure witnesses passed (1 GiB, 155 modules); `/tmp/migration-gate-validation.log`.
- Carrier effects: 4 wet witnesses passed (1 GiB, 156 modules), including real pinned temporary Git import, original-byte replay, competing writers/idempotency, and stale editorial revision refusal; `/tmp/carrier-four-wet-validation.log`.
- Crash-phase recovery and gate publication: 1 wet witness passed (1 GiB, 157 modules); `/tmp/migration-recovery-wet-validation.log`.
- Separate-process acceptance: initialize, first append, stale contender, second append, idempotent replay and two-event readback passed against one temporary store across fresh interpreters; `/tmp/carrier-process-validation.log` and `/tmp/carrier-process-early-stop-validation.log`.
- Shared production append graph refusal passed in `/tmp/editorial-append-graph-validation.log`. Completion rejects unresolved seed references; that negative control and positive recovery/edit controls passed in `/tmp/roadmap-graph-completion-wet.log`.
- Shared closure budget regression: a two-object durable closure refuses bound 1 and returns both objects under bound 100000 (1 GiB, 45 modules); `/tmp/closure-budget-validation.log`.

The earlier 1/2 GiB stale-writer runs were killed without verdict. Investigation found `fabric_storage_file_closure` eagerly constructed and folded all 100000 configured budget steps even after the closure ended. The shared implementation now stops when pending work is empty or refused, preserving its step ceiling. The same stale-writer test then passed under 1 GiB. Import factory extraction reduced the carrier closure from 555 modules to 155 without source substitutions.

## Remaining allocation lane validation

`workspace_offer_producer`, `workspace_tool_observer`, `workspace_preparation`, and the `workspace_prepare_convergence` bridge passed source resolution/typecheck in the explicitly imported 846-module runtime validation (all five runtime claims passed, clean exit 0; peak 5,531,768 KiB, zero swap). Approved commissioning/commercial evidence production and the CLI missing-plan binding remain pending. The bridge derives real offers through existing route admission, persists the selected plan, then returns the existing sealed convergence basis. The new tool observer reads the actual controller release binaries and host/time; it does not accept an authored readiness flag. Their entry is `test.manual.workspace_preparation_typecheck`; it is a typecheck entry, not proof of fleet availability. No slot is currently asserted commissioned. No real VM, SSH connection, release or expiry loop has been demonstrated by this lane.


## Read-only live-source preflight

The live checkout's local `roadmap-events` and origin-tracking refs both named `6f4f6dceb06c1f93d3c1f9c3c8a4ee0ad02fcd51` when inspected. That object contains two v1 event files on two nodes. `test.manual.roadmap_import_audit` read that exact live-repository object and validated all paths, canonical event IDs, bytes and the expected count through the production importer decoder; exit 0 under 1 GiB. Its entry accepts no target binding and cannot publish state. Receipt: `/tmp/roadmap-live-source-audit.log`.

This was a local-ref inventory, with no fetch or write freeze. Actual cutover must quiesce the old writer through the existing deployment/convergence boundary, pin/recheck its final head, and only then finish initialization and switch readers. A baseline receipt does not cover writes appended to the old carrier after its pinned commit.
