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
  - parity versus legacy `lib/gcp-ops` credential graph shape is still pending.
- **Next increments**:
  - add builder parity target for credential chain.
  - align service-call lowering coverage for remaining credential operations across providers.

## S3 — Tool Install Upsert (`tools.bootstrap`)

- **Current status**: ⚠ lower-stage unresolved service call (`shell.Find.ListDirs`).
- **Primary gap**:
  - service call resolution coverage for this module path.
- **Next increments**:
  - complete service endpoint lookup/resolution path for bootstrap service calls.

## S4 — Gist Snapshot (`tools.gist`)

- **Current status**: ⚠ typecheck blocked by pattern output-shape mismatches (`upsert.outputs` / `transaction.outputs` field contracts).
- **Primary gap**:
  - gist pattern composition currently diverges from expected pattern output contracts in strict single-file checks.
- **Next increments**:
  - align gist pattern output field contracts with current `std.patterns` shape.
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

- **Current status**: ⚠ lower-stage unresolved service call (`cargo.Build.Clippy`).
- **Primary gap**:
  - service op resolution for cargo service path in this module.
- **Next increments**:
  - map/service-resolve cargo operations in lowering + bridge execution path.

## Deps (`tools.deps`)

- **Current status**: ⚠ typecheck blocked by shared pattern output-shape mismatches (`upsert.outputs`/`transaction.outputs` field contracts).
- **Primary gap**:
  - deps workflow still depends on stale pattern field assumptions in strict checks.
- **Next increments**:
  - migrate deps workflow call-sites to current `std.patterns` output contracts.

## Auth (`services.shell`)

- **Current status**: ✅ service-only module now lowers transport triplets and emits obligations.
- **Primary gap**:
  - still lacks a direct executable callable entrypoint by design.
- **Next increments**:
  - validate caller workflows against this service module for parity and runtime bridge coverage.

## S8 — Infra Bootstrap (`infra.core`)

- **Current status**: ✅ expands and obligations derive.
- **Primary gap**:
  - interface/resource implementation parity and contract test generation are still pending.
- **Next increments**:
  - implement interface resolution and contract-driven generated tests.

## S9 — Cross-Cloud Deployment (`examples.deployment`)

- **Current status**: ⚠ typecheck blocked by record-shape mismatches in provider configuration helpers.
- **Primary gap**:
  - provider config helper return shapes still diverge from strict expected records.
- **Next increments**:
  - align provider config helper return contracts and rerun full cross-provider lowering.
