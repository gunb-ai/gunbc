# Workflow Modeling Preview (Bridge → Phase 2)

This document tracks per-workflow modeling status from the current DSL compiler path.
It answers three questions for each workflow:

1. **What compiles/lowering status do we currently have?**
2. **What model gaps are blocking full parity/execution?**
3. **What should be implemented next to close that gap?**

The source of truth for current status is the workflow fixture contract suite in
`daglang-cli/tests/workflow_contracts.rs`.

---

## S1 — Makegen (`tools.makegen`)

- **Current status**: ✅ expands, obligations derive, compiled-vs-builder parity test passes.
- **Current gaps**:
  - none in bridge scope (this is the proving workflow baseline).
- **Next increments**:
  - keep parity/test gates green while Phase 2+ modeling lands.

## S2 — Credential Chain GCP (`cloud.gcp.credential`)

- **Current status**: ⚠ typecheck blocked in single-module fixture contract.
- **Primary gap**:
  - missing type/import/model integration in the selected compile path for all referenced credential types.
- **Next increments**:
  - close type/import coverage for credential module closure.
  - add builder parity target for credential chain once lowering succeeds.

## S3 — Tool Install Upsert (`tools.bootstrap`)

- **Current status**: ⚠ lower-stage unresolved service call (`shell.Find.ListDirs`).
- **Primary gap**:
  - service call resolution coverage for this module path.
- **Next increments**:
  - complete service endpoint lookup/resolution path for bootstrap service calls.

## S4 — Gist Snapshot (`tools.gist`)

- **Current status**: ⚠ typecheck blocked (domain types unresolved in fixture contract).
- **Primary gap**:
  - unresolved type model wiring for gist-facing domain types.
- **Next increments**:
  - complete type environment coverage for gist module closure.
  - then validate loop/SubDag modeling parity against builder shape.

## S5 — CI Pipeline (`pipelines.ci`)

- **Current status**: ✅ expands and obligations derive for pipeline module.
- **Primary gap**:
  - phase-4 stage-group/renderer contract work still pending.
- **Next increments**:
  - derive/emit stage-group manifest sections and CI parity assertions.

## S6 — LLM Review (`examples.abstract_services`)

- **Current status**: ⚠ typecheck blocked by interface contract mismatches in fixture contract.
- **Primary gap**:
  - interface operation signature alignment and service contract conformance.
- **Next increments**:
  - complete interface/implements matching path for abstract services.

## Credential (AWS) (`cloud.aws.credential`)

- **Current status**: ⚠ typecheck blocked (credential-domain type coverage gap).
- **Primary gap**:
  - unresolved/undefined credential type layer for module closure.
- **Next increments**:
  - complete AWS credential type modeling and contract checks.

## Credential (Azure) (`cloud.azure.credential`)

- **Current status**: ⚠ typecheck blocked (credential-domain type coverage gap).
- **Primary gap**:
  - unresolved/undefined credential type layer for module closure.
- **Next increments**:
  - complete Azure credential type modeling and contract checks.

## Clippy (`tools.clippy`)

- **Current status**: ⚠ lower-stage unresolved service call (`cargo.Build.Clippy`).
- **Primary gap**:
  - service op resolution for cargo service path in this module.
- **Next increments**:
  - map/service-resolve cargo operations in lowering + bridge execution path.

## Deps (`tools.deps`)

- **Current status**: ⚠ typecheck blocked in fixture contract (`FilePath` surface mismatch).
- **Primary gap**:
  - std-type surface integration for this module closure.
- **Next increments**:
  - unify type environment behavior across tool modules.

## Auth (`services.shell`)

- **Current status**: ⚠ lower-stage no-callable/no-pipeline terminal state (service-only module).
- **Primary gap**:
  - no callable entrypoint by design in this file; cannot lower to executable DAG directly.
- **Next increments**:
  - consume this service module from executable workflows and validate through caller parity tests.

## S8 — Infra Bootstrap (`infra.core`)

- **Current status**: ✅ expands and obligations derive.
- **Primary gap**:
  - interface/resource implementation parity and contract test generation are still pending.
- **Next increments**:
  - implement interface resolution and contract-driven generated tests.

## S9 — Cross-Cloud Deployment (`examples.deployment`)

- **Current status**: ⚠ typecheck blocked (cross-provider type/config model gaps).
- **Primary gap**:
  - unresolved provider config/type model integration in this composed workflow.
- **Next increments**:
  - finish provider config type wiring and cross-provider composition validation.
