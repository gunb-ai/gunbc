# Workflow Modeling Preview (Bridge → Phase 2)

This document tracks per-workflow modeling status from the current DSL compiler path.
It answers three questions for each workflow:

1. **What compiles/lowering status do we currently have?**
2. **What model gaps are blocking full parity/execution?**
3. **What should be implemented next to close that gap?**

The source of truth for current status is the workflow fixture contract suite in
`daglang-cli/tests/workflow_contracts.rs`.

Global status:
- ✅ `daglang obligations dsl --format json` now succeeds end-to-end (full DSL root typechecks), which unblocks root-level obligation auditing and regression testing.

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
  - tighten newly-added deterministic bootstrap parity scaffold to zero-delta shape parity.

## S4 — Gist Snapshot (`tools.gist`)

- **Current status**: ✅ compiles, emits obligations contract successfully, and core gist workflow is compressed to 59 DSL lines.
- **Primary gap**:
  - exact parity against legacy gist builder shape is still pending.
- **Next increments**:
  - tighten newly-added deterministic snapshot/diff/recent parity scaffolds to zero-delta shape parity.
  - gate all three modes on exact structural parity.

## S5 — CI Pipeline (`pipelines.ci`)

- **Current status**: ✅ expands/obligations derive, manifest stage groups are emitted for pipeline nodes, and CI single-file obligations include transitive dependency closure metrics (`pure_node_determinism_targets: 91`, `transport_execution_targets: 24`, `resource_acquire_targets: 4`, `resource_release_targets: 4`).
- **Primary gap**:
  - obligation parity target (133) is not yet reached.
  - exact parity against legacy CI builder shape is still pending.
  - renderer UX still needs collapsible stage-group presentation.
- **Next increments**:
  - tighten newly-added deterministic CI parity scaffold to zero-delta shape parity.
  - wire stage-group manifest output into collapsible progress-renderer sections.

## S6 — LLM Review (`examples.abstract_services`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - parity and runtime execution coverage for abstract-interface flows is still pending.
- **Next increments**:
  - add parity and execution assertions for interface-backed abstract-service workflows.

## Credential (AWS) (`cloud.aws.credential`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - exact parity against legacy credential-chain builder shape is still pending.
- **Next increments**:
  - tighten newly-added deterministic AWS credential parity scaffold to zero-delta shape parity.

## Credential (Azure) (`cloud.azure.credential`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - exact parity against legacy Azure credential-chain builder shape is still pending.
- **Next increments**:
  - tighten newly-added deterministic Azure credential parity scaffold to zero-delta shape parity.
  - restore richer federated-flow modeling once parity baselines are stable.

## Clippy (`tools.clippy`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - exact parity against legacy clippy workflow shape is still pending.
- **Next increments**:
  - tighten newly-added deterministic clippy parity scaffold to zero-delta shape parity.
  - add execution assertions on the resolved clippy path.

## Deps (`tools.deps`)

- **Current status**: ✅ compiles and emits obligations contract successfully.
- **Primary gap**:
  - exact parity against legacy deps workflow shape is still pending.
- **Next increments**:
  - tighten newly-added deterministic deps parity scaffold to zero-delta shape parity.
  - add execution bridge assertions for deps workflow.

## Build (`tools.build`)

- **Current status**: ✅ compiles and deterministic parity scaffold is wired against legacy builder graph.
- **Primary gap**:
  - exact parity against legacy build workflow shape is still pending.
- **Next increments**:
  - tighten deterministic build parity scaffold to zero-delta shape parity.
  - add execution bridge assertions for build workflow.

## Codegen (`tools.codegen`)

- **Current status**: ✅ compiles and deterministic parity scaffold is wired against legacy builder graph.
- **Primary gap**:
  - exact parity against legacy codegen workflow shape is still pending.
- **Next increments**:
  - tighten deterministic codegen parity scaffold to zero-delta shape parity.
  - add execution assertions for codegen artifact lifecycle behavior.

## Pragma (`tools.pragma`)

- **Current status**: ✅ compiles and deterministic parity scaffold is wired against legacy builder graph.
- **Primary gap**:
  - exact parity against legacy pragma workflow shape is still pending.
- **Next increments**:
  - tighten deterministic pragma parity scaffold to zero-delta shape parity.
  - add execution assertions around pragma parsing/application flow.

## Docgen (`tools.docgen`)

- **Current status**: ✅ compiles and deterministic parity scaffold is wired against legacy builder graph.
- **Primary gap**:
  - exact parity against legacy docgen workflow shape is still pending.
- **Next increments**:
  - tighten deterministic docgen parity scaffold to zero-delta shape parity.
  - add execution assertions for docgen read/emit paths.

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

- **Current status**: ✅ compiles and emits obligations contract successfully; lowering regressions now cover provider-hint portability wiring (`GcpConfig`/`AwsConfig`/`AzureConfig`) and cross-provider credential-chain call composition.
- **Primary gap**:
  - parity against legacy cross-cloud deployment graph shape is still pending.
- **Next increments**:
  - add deployment parity harness coverage and runtime execution bridge checks.
