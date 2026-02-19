# H10 Design: Compute Stack Service Interfaces

## Problem

Compute stack provisioning needs service-level interfaces, but scope is too large without a phased model.

## Decision

Deliver MVP scope first: Cloud Run + GCS + Load Balancer interfaces. Defer GCE integration to phase 2.

## Service Interfaces (MVP)

- `RunService`:
  - deploy revision
  - set traffic split
  - fetch service status
- `StorageService`:
  - ensure bucket
  - set lifecycle/policy
  - upload artifact pointer
- `LoadBalancerService`:
  - ensure backend + URL map + HTTPS policy
  - attach service backends
  - expose endpoint status

## Non-MVP (Phase 2)

- `ComputeEngineService` (VM templates, MIG, instance lifecycle)

## Invariants

- All service operations are idempotent.
- Plan/apply separation is explicit.
- Provider-specific request/response mapped to stable IR types.

## Migration Plan

1. Define provider-neutral service traits.
2. Implement GCP adapters for Run/GCS/LB.
3. Integrate into infra plan/apply DAG.
4. Add drift/status probes and rollback-safe apply behavior.

## Follow-up Implementation Tasks

- `H10.1` Define neutral service interfaces and typed models.
- `H10.2` Implement GCP Cloud Run adapter.
- `H10.3` Implement GCP GCS adapter.
- `H10.4` Implement GCP LB adapter.
- `H10.5` Wire adapters into infra plan/apply DAG.
- `H10.6` Add conformance tests + idempotency checks.
- `H10.7` Phase 2: add GCE interface and adapter.
