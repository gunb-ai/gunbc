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

- **Current status**: ✅ compiles, expands, and obligations derive in workflow contract fixture.
- **Primary gap**:
  - exact parity versus legacy `lib/gcp-ops` credential graph shape is still pending.
- **Next increments**:
  - tighten the newly-added credential parity target from deterministic report scaffolding to zero-delta structural parity.
  - align service-call lowering coverage for remaining credential operations across providers.

## S3 — Tool Install Upsert (`tools.bootstrap`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - parity against legacy bootstrap builder shape is still pending.
- **Next increments**:
  - add parity harness coverage for bootstrap workflow shape.

## S4 — Gist Snapshot (`tools.gist`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - parity against legacy gist builder shape is still pending.
- **Next increments**:
  - add gist mode parity harnesses (snapshot/diff/recent) against legacy graph contracts.

## S5 — CI Pipeline (`pipelines.ci`)

- **Current status**: ✅ expands and obligations derive for pipeline module.
- **Primary gap**:
  - phase-4 stage-group/renderer contract work still pending.
- **Next increments**:
  - derive/emit stage-group manifest sections and CI parity assertions.

## S6 — LLM Review (`examples.abstract_services`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - parity and runtime execution coverage for abstract-interface flows is still pending.
- **Next increments**:
  - add parity and execution assertions for interface-backed abstract-service workflows.

## Credential (AWS) (`cloud.aws.credential`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - parity against legacy credential-chain builder shape is still pending.
- **Next increments**:
  - add credential graph parity harness coverage against legacy AWS chain.

## Credential (Azure) (`cloud.azure.credential`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - parity against legacy Azure credential-chain builder shape is still pending.
- **Next increments**:
  - add builder-parity coverage for Azure chain and restore richer federated-flow modeling.

## Clippy (`tools.clippy`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - parity against legacy clippy workflow shape is still pending.
- **Next increments**:
  - add clippy workflow parity harness and execution assertions.

## Deps (`tools.deps`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - parity against legacy deps workflow shape is still pending.
- **Next increments**:
  - add deps workflow parity harness and execution bridge assertions.

## Auth (`services.shell`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - parity versus consumer workflows that invoke shell services is still pending.
- **Next increments**:
  - validate service-only module parity through downstream caller parity/execution tests.

## S8 — Infra Bootstrap (`infra.core`)

- **Current status**: ✅ expands and obligations derive.
- **Primary gap**:
  - interface/resource implementation parity and contract test generation are still pending.
- **Next increments**:
  - implement interface resolution and contract-driven generated tests.

## S9 — Cross-Cloud Deployment (`examples.deployment`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - parity against legacy cross-cloud deployment graph shape is still pending.
- **Next increments**:
  - add deployment parity harness coverage and runtime execution bridge checks.
