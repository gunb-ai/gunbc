# SDLC Redundancy Ledger

Historical snapshot of known duplication and overlap in the SDLC DSL subsystem.

Status note (2026-03-05): profile-related entries in this file reflect an older
profile-era design. The current branch only retains a temporary
`dsl/profiles/sdlc.dag` compatibility path while concrete binding/link cleanup lands.

## Active Redundancies

### Stub provider definitions (unit_test.dag vs stub_providers.dag)

- **Files**: `profiles/unit_test.dag` lines 39–179, `services/sdlc/providers/stub_providers.dag`
- **What**: Both define identical stub service implementations (StubIssueProvider, InMemoryClaimStore, etc.) with same operation signatures
- **Resolution**: Remove inline definitions from `unit_test.dag`; bind to `stub_providers.dag` providers instead
- **Status**: Deferred — requires profile bind syntax to reference cross-module services

## Intentional Duplication

### Pipeline vs Workflow (different abstraction layers)

- **Files**: `pipelines/sdlc.dag` (479 lines), `workflows/sdlc.dag` (59 lines)
- **Why**: Pipeline is a reference design document showing full stage logic. Workflow is the operational entry point calling `dispatch_sdlc()` from `funcs/sdlc_worker.dag`. Pipeline is demoted to reference-only (CL-7).

### Historical profile duplication note

- **Files**: historical `profiles/{unit_test,local,cloud_run}.dag` files plus the aggregate `profiles/sdlc.dag`; the current branch only retains temporary `dsl/profiles/sdlc.dag`
- **Why**: this was part of the older profile-based binding design. Treat it as historical context, not an active duplication target.

## No Redundancy

| Area | Files | Status |
|------|-------|--------|
| Interfaces | 7 files | Each unique; CapabilityBehaviorContract/CapabilityFailureContract extracted to std/behavioral.dag |
| Concrete providers | 17 files | Each has distinct storage backend (file, GCS, pubsub, GitHub) |
| Worker / Stages | 2 files | Clean separation: worker=dispatch loop, stages=per-stage handlers |

## Counts

- Active redundancies: **1** (stub provider defs)
- Intentional duplicates / historical notes: **2** (pipeline/workflow layer, profile-era profile references)
- Dead code: **0** (sdlc_dispatch_runtime.dag deleted in CL-DELETE)
