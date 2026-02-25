# SDLC Implementation Roadmap

Status: Active
Date: 2026-02-25
Parent: [mega-modeling-design.md](mega-modeling-design.md) (MD0-D)
Sibling: [e2e-gap-analysis.md](e2e-gap-analysis.md), [domain-modeling-comprehensive.md](domain-modeling-comprehensive.md)
Cleanup dependency: [foundation-cleanup.md](../v4/foundation-cleanup.md)

## 1. Goal

SDLC pipeline running holistically in `.dag` — issue intake through close,
claim-based workers, multi-provider adapters, all modeled and executed from DSL.
No Rust-side SDLC logic. No extern bridges.

## 2. Current State

### 2.1 DSL Inventory (3,616 lines across 20 files)

| Category | Files | Lines | Status |
|----------|-------|-------|--------|
| Pipeline | `pipelines/sdlc.dag` | 554 | Complete — 11 stages |
| Stage handlers | `funcs/sdlc_stages.dag` | 756 | 7/8 (testing→done missing) |
| Worker dispatch | `funcs/sdlc_worker.dag` | 393 | Complete |
| Dispatch runtime | `funcs/sdlc_dispatch_runtime.dag` | 104 | **Stub** |
| Validation runtime | `funcs/sdlc_validation_runtime.dag` | 59 | **Stub** |
| Interfaces (7) | `interfaces/*.dag` | 373 | Complete |
| Providers (9) | `services/sdlc/providers/*.dag` | 1,235 | Complete |
| Profile binding | `profiles/sdlc.dag` | 109 | Complete |
| Workflow entry | `workflows/sdlc.dag` | 33 | Stub |
| Deploy | `infra/sdlc/deploy.dag` | 141 | Scaffold |

### 2.2 What's Blocking E2E Execution

| Blocker | Category | Unblocked by |
|---------|----------|-------------|
| Pipeline not in workflow catalog | Wiring | SDLC-1 |
| Dispatch runtime is stub | Logic | SDLC-2 |
| Validation runtime is stub | Logic | SDLC-3 |
| testing→done handler missing | Logic | SDLC-4 |
| Signal + Artifact stores: stubs only | Providers | SDLC-5, SDLC-6 |
| 9 extern bridges in Rust | Foundation | [foundation-cleanup.md](../v4/foundation-cleanup.md) |

## 3. Dependency Graph

```
foundation-cleanup.md                    this document
┌─────────────────────┐                  ┌──────────────────────┐
│                     │                  │                      │
│  FC-CL (dead code)  │                  │  SDLC-1 (catalog)    │
│         │           │                  │    │                  │
│  FC-NF7 (lowerer)──────────────────────│──▶ SDLC-2 (dispatch) │
│         │           │                  │    │                  │
│  FC-P6 (policy) ────│─ (unblocks       │  SDLC-3 (validation) │
│         │           │   pure-DSL       │    │                  │
│  FC-P7 (registry) ──│─  rendering)     │  SDLC-4 (testing)    │
│         │           │                  │    │                  │
│  FC-CF (compiler) ──│─ (unblocks       │  SDLC-5 (signal)     │
│         │           │   tree/snapshot)  │  SDLC-6 (artifact)   │
│  FC-P8 (anemic) ────│─ (last externs)  │    │                  │
│                     │                  │  SDLC-7 (verify)     │
└─────────────────────┘                  │    │                  │
                                         │  SDLC-8 (local e2e)  │
                                         │    │                  │
                                         │  SDLC-CD (cloud)     │
                                         └──────────────────────┘
```

**Parallelism**: SDLC-1 through SDLC-6 can start immediately (no foundation
dependency). FC-NF7 unblocks extern bridge elimination but not SDLC activation.
Cloud deployment (SDLC-CD) depends on SDLC-8.

## 4. SDLC Activation Tasks

These tasks bring the pipeline from "compiles" to "runs e2e on local profile."

| ID | Task | Size | Deps | Design ref |
|----|------|------|------|------------|
| SDLC-1 | Register SDLC pipeline in workflow catalog. Add `sdlc` to `dsl_registry.rs`. Wire `WorkspaceBinary::Sdlc` dispatch. | M | — | mega-modeling §5.3 |
| SDLC-2 | Fill dispatch runtime: real stage transition logic in `sdlc_dispatch_runtime.dag`. Determine next stage from labels, validate via state machine from `std.state_machines`. | M | SDLC-1 | domain-modeling §3, mega-modeling §A.4 |
| SDLC-3 | Fill validation runtime: `review_gate` (approval label + reviewer comment check), `ci_gate` (passing CI status). | M | SDLC-2 | mega-modeling §6.4 (approval yield) |
| SDLC-4 | Complete testing→done handler: cargo test + clippy invocation, conditional merge on all-pass, label transitions. | M | SDLC-1 | mega-modeling §A.5, §A.9 |
| SDLC-5 | Local SignalStore provider: file-based signal bus. Emit writes JSON, consume reads + filters by type, ack marks consumed. Must satisfy `interfaces/signal_store.dag` contracts. | M | — | mega-modeling §6.6 |
| SDLC-6 | Local ArtifactStore provider: file-based storage. Content-hash keyed paths, two-phase commit markers (provisional → canonical). Must satisfy `interfaces/artifact_store.dag` contracts. | M | — | mega-modeling §6.3 |
| SDLC-7 | Profile binding verification: compile all 3 profiles, fix wiring gaps. Run unit_test profile hermetic e2e (all mocked). | M | SDLC-1:6 | — |
| SDLC-8 | Local profile e2e: real GitHub repo integration test. Validate idea → design → review flow. | L | SDLC-7 | mega-modeling §2.1.3 |

### Deliverable

`gunbc sdlc --profile local --repo owner/name` runs the full pipeline.
unit_test profile passes hermetic e2e. Local profile processes real GitHub
issues through idea → design → review stages.

## 5. Cloud Deployment Tasks

Bring cloud_run profile to production after local e2e works.

| ID | Task | Size | Deps | Design ref |
|----|------|------|------|------------|
| SDLC-CD1 | GCS SignalStore: PubSub-backed signal bus with at-least-once delivery. | M | SDLC-8 | mega-modeling §6.6 |
| SDLC-CD2 | GCS ArtifactStore: GCS-backed, content-hash paths, generation CAS markers. | M | SDLC-8 | mega-modeling §6.3 |
| SDLC-CD3 | GCP credential chaining: WIF OIDC exchange → metadata server → scoped token. | L | SDLC-8 | — |
| SDLC-CD4 | Cloud Run deployment DAG: wire `infra/sdlc/deploy.dag` to real lifecycle. | L | SDLC-CD1:3 | mega-modeling §2.1.1 |
| SDLC-CD5 | Multi-worker CAS stress test: 3 workers, exactly-once stage execution. | M | SDLC-CD4 | mega-modeling §6.2 |
| SDLC-CD6 | CI integration: hermetic test + cloud_run smoke test in staging. | M | SDLC-CD5 | mega-modeling §7 |

### Deliverable

SDLC on Cloud Run with GCS-backed ledgers, PubSub signals, multi-worker
claim contention, and CI coverage.

## 6. Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| SDLC e2e reveals new DSL lowerer gaps | Delays SDLC-7/8 | unit_test profile (hermetic) validates logic without transport. Gaps surface early. |
| Cloud Run infra not available | Delays SDLC-CD | Local profile is fully functional. Cloud is scoped last. |
| Agent provider (Codex) not ready | Blocks implementation stage | accepted→implementing can be deferred; other stages work independently. |
| Profile binding has compile errors | Blocks SDLC-7 | Compile each profile individually during development, not just at integration. |

## 7. Endstate

**DSL owns all SDLC logic.** Issue intake → design → review → implementation →
code review → testing → close. Three deployment profiles. Multi-worker claim
contention. Replay-skip idempotency. LLM-driven design and review. Agent-spawned
implementation.

The Rust substrate compiles and executes DAGs. It has no knowledge of SDLC
stages, claim stores, issue providers, or stage transition rules.
